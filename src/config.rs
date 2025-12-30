use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("Config file not found at {0}")]
    NotFound(PathBuf),
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("kit")
            .join("kit.toml")
    }

    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        Self::load_from(&path)
    }

    pub fn load_from(path: &PathBuf) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::NotFound(path.clone()));
        }
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config: GlobalConfig::default(),
            tools: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"
[tools.ripgrep]
source = "brew"
aliases = ["rg"]
"#).unwrap();

        let config = Config::load_from(&file.path().to_path_buf()).unwrap();
        assert!(config.tools.contains_key("ripgrep"));
    }

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
