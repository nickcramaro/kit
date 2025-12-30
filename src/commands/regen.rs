use crate::config::Config;
use crate::shell::Shell;
use anyhow::Result;

pub fn run(config: &Config) -> Result<()> {
    let shell = Shell::new(
        config.config.bin_dir.parent().unwrap().to_path_buf(),
        config.config.shell_rc.clone(),
    );

    // Collect all aliases
    let mut aliases: Vec<(String, String)> = Vec::new();
    for (name, tool) in &config.tools {
        for alias in tool.aliases() {
            aliases.push((alias.clone(), name.clone()));
        }
    }

    // Write aliases file
    shell.write_aliases(&aliases)?;
    println!("Wrote {} aliases to {:?}", aliases.len(), shell.aliases_path());

    // Create symlinks
    for (name, tool) in &config.tools {
        if let Ok(path) = which::which(name) {
            // Create symlink for main binary
            shell.create_symlink(name, &path)?;

            // Create symlinks for aliases
            for alias in tool.aliases() {
                shell.create_symlink(alias, &path)?;
            }
        }
    }

    println!("Done. Run `kit setup` if PATH is not configured.");

    Ok(())
}
