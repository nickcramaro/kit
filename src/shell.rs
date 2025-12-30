use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const MARKER_START: &str = "# >>> kit >>>";
const MARKER_END: &str = "# <<< kit <<<";

pub struct Shell {
    pub kit_dir: PathBuf,
    pub shell_rc: PathBuf,
}

impl Shell {
    pub fn new(kit_dir: PathBuf, shell_rc: PathBuf) -> Self {
        Self { kit_dir, shell_rc }
    }

    pub fn aliases_path(&self) -> PathBuf {
        self.kit_dir.join("aliases.zsh")
    }

    pub fn bin_dir(&self) -> PathBuf {
        self.kit_dir.join("bin")
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.kit_dir)?;
        fs::create_dir_all(self.bin_dir())?;
        Ok(())
    }

    pub fn write_aliases(&self, aliases: &[(String, String)]) -> Result<()> {
        self.ensure_dirs()?;

        let mut content = String::from("# ~/.kit/aliases.zsh - managed by kit, do not edit\n");
        for (alias, target) in aliases {
            content.push_str(&format!("alias {}=\"{}\"\n", alias, target));
        }

        fs::write(self.aliases_path(), content)?;
        Ok(())
    }

    pub fn inject_rc(&self) -> Result<()> {
        let home = dirs::home_dir().unwrap_or_default();
        let kit_dir_str = self.kit_dir.to_string_lossy().replace(
            &home.to_string_lossy().to_string(),
            "$HOME",
        );

        let injection = format!(
            r#"{marker_start}
export PATH="{kit_dir}/bin:$PATH"
source "{kit_dir}/aliases.zsh"
{marker_end}"#,
            marker_start = MARKER_START,
            marker_end = MARKER_END,
            kit_dir = kit_dir_str,
        );

        // Read existing rc file
        let existing = fs::read_to_string(&self.shell_rc).unwrap_or_default();

        // Check if already injected
        if existing.contains(MARKER_START) {
            // Replace existing injection
            let before = existing
                .split(MARKER_START)
                .next()
                .unwrap_or("");
            let after = existing
                .split(MARKER_END)
                .nth(1)
                .unwrap_or("");

            let new_content = format!("{}{}{}", before, injection, after);
            fs::write(&self.shell_rc, new_content)?;
        } else {
            // Append new injection
            let new_content = format!("{}\n{}\n", existing.trim_end(), injection);
            fs::write(&self.shell_rc, new_content)?;
        }

        Ok(())
    }

    pub fn create_symlink(&self, name: &str, target: &PathBuf) -> Result<()> {
        self.ensure_dirs()?;
        let link_path = self.bin_dir().join(name);

        // Remove existing symlink if present
        if link_path.exists() || link_path.is_symlink() {
            fs::remove_file(&link_path).ok();
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(target, &link_path)
            .context(format!("Failed to create symlink for {}", name))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_aliases() {
        let tmp = TempDir::new().unwrap();
        let shell = Shell::new(
            tmp.path().join(".kit"),
            tmp.path().join(".zshrc"),
        );

        let aliases = vec![
            ("rg".to_string(), "ripgrep".to_string()),
            ("cat".to_string(), "bat".to_string()),
        ];

        shell.write_aliases(&aliases).unwrap();

        let content = fs::read_to_string(shell.aliases_path()).unwrap();
        assert!(content.contains("alias rg=\"ripgrep\""));
        assert!(content.contains("alias cat=\"bat\""));
    }

    #[test]
    fn test_inject_rc_new() {
        let tmp = TempDir::new().unwrap();
        let rc_path = tmp.path().join(".zshrc");
        fs::write(&rc_path, "# existing content\n").unwrap();

        let shell = Shell::new(tmp.path().join(".kit"), rc_path.clone());
        shell.inject_rc().unwrap();

        let content = fs::read_to_string(&rc_path).unwrap();
        assert!(content.contains("# >>> kit >>>"));
        assert!(content.contains("# <<< kit <<<"));
        assert!(content.contains("# existing content"));
    }

    #[test]
    fn test_inject_rc_replace() {
        let tmp = TempDir::new().unwrap();
        let rc_path = tmp.path().join(".zshrc");
        fs::write(
            &rc_path,
            "before\n# >>> kit >>>\nold stuff\n# <<< kit <<<\nafter\n",
        )
        .unwrap();

        let shell = Shell::new(tmp.path().join(".kit"), rc_path.clone());
        shell.inject_rc().unwrap();

        let content = fs::read_to_string(&rc_path).unwrap();
        assert!(content.contains("before"));
        assert!(content.contains("after"));
        assert!(!content.contains("old stuff"));
        assert_eq!(content.matches("# >>> kit >>>").count(), 1);
    }
}
