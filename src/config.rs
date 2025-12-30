use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub config: GlobalConfig,
    #[serde(default)]
    pub tools: HashMap<String, Tool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GlobalConfig {
    #[serde(default = "default_bin_dir")]
    pub bin_dir: PathBuf,
    #[serde(default = "default_shell_rc")]
    pub shell_rc: PathBuf,
}

fn default_bin_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".kit")
        .join("bin")
}

fn default_shell_rc() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".zshrc")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "source", rename_all = "lowercase")]
pub enum Tool {
    Brew {
        #[serde(default)]
        aliases: Vec<String>,
    },
    Mise {
        version: Option<String>,
        #[serde(default)]
        aliases: Vec<String>,
    },
    Curl {
        install_url: String,
        binary: String,
        #[serde(default)]
        aliases: Vec<String>,
    },
}

impl Tool {
    pub fn aliases(&self) -> &[String] {
        match self {
            Tool::Brew { aliases } => aliases,
            Tool::Mise { aliases, .. } => aliases,
            Tool::Curl { aliases, .. } => aliases,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let toml_str = r#"
[tools.ripgrep]
source = "brew"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.tools.contains_key("ripgrep"));
        assert!(matches!(config.tools["ripgrep"], Tool::Brew { .. }));
    }

    #[test]
    fn test_parse_full_config() {
        let toml_str = r#"
[config]
bin_dir = "~/.kit/bin"
shell_rc = "~/.zshrc"

[tools.ripgrep]
source = "brew"
aliases = ["rg"]

[tools.python]
source = "mise"
version = "3.12"

[tools.starship]
source = "curl"
install_url = "https://starship.rs/install.sh"
binary = "starship"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.tools.len(), 3);

        match &config.tools["ripgrep"] {
            Tool::Brew { aliases } => assert_eq!(aliases, &["rg"]),
            _ => panic!("Expected Brew tool"),
        }

        match &config.tools["python"] {
            Tool::Mise { version, .. } => assert_eq!(version, &Some("3.12".to_string())),
            _ => panic!("Expected Mise tool"),
        }

        match &config.tools["starship"] {
            Tool::Curl { install_url, binary, .. } => {
                assert_eq!(install_url, "https://starship.rs/install.sh");
                assert_eq!(binary, "starship");
            }
            _ => panic!("Expected Curl tool"),
        }
    }
}
