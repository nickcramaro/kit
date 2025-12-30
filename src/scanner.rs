use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct FoundBinary {
    pub name: String,
    pub path: PathBuf,
    pub source: DetectedSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DetectedSource {
    Brew,
    Mise,
    Unknown,
}

pub struct Scanner {
    brew_formulas: HashSet<String>,
    mise_tools: HashSet<String>,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            brew_formulas: Self::get_brew_formulas(),
            mise_tools: Self::get_mise_tools(),
        }
    }

    fn get_brew_formulas() -> HashSet<String> {
        std::process::Command::new("brew")
            .args(["list", "--formula", "-1"])
            .output()
            .ok()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_mise_tools() -> HashSet<String> {
        std::process::Command::new("mise")
            .args(["ls", "--current", "--json"])
            .output()
            .ok()
            .and_then(|output| {
                serde_json::from_slice::<HashMap<String, serde_json::Value>>(&output.stdout).ok()
            })
            .map(|map| map.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn scan_path(&self) -> Vec<FoundBinary> {
        let path_var = env::var("PATH").unwrap_or_default();
        let mut seen: HashSet<String> = HashSet::new();
        let mut binaries = Vec::new();

        for dir in path_var.split(':') {
            let dir_path = PathBuf::from(dir);
            if !dir_path.is_dir() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }

                    // Check if executable
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(meta) = path.metadata() {
                            if meta.permissions().mode() & 0o111 == 0 {
                                continue;
                            }
                        }
                    }

                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let name = name.to_string();
                        if seen.contains(&name) {
                            continue;
                        }
                        seen.insert(name.clone());

                        let source = self.detect_source(&name, &path);
                        binaries.push(FoundBinary { name, path, source });
                    }
                }
            }
        }

        binaries
    }

    fn detect_source(&self, name: &str, path: &PathBuf) -> DetectedSource {
        // Check if it's a brew formula
        if self.brew_formulas.contains(name) {
            return DetectedSource::Brew;
        }

        // Check if path contains mise
        if path.to_string_lossy().contains("mise") {
            return DetectedSource::Mise;
        }

        // Check for common mise tool names
        if self.mise_tools.contains(name) {
            return DetectedSource::Mise;
        }

        DetectedSource::Unknown
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_finds_binaries() {
        let scanner = Scanner::new();
        let binaries = scanner.scan_path();
        // Should find at least some common binaries
        assert!(!binaries.is_empty());
    }

    #[test]
    fn test_detected_source_display() {
        assert_eq!(format!("{:?}", DetectedSource::Brew), "Brew");
        assert_eq!(format!("{:?}", DetectedSource::Mise), "Mise");
        assert_eq!(format!("{:?}", DetectedSource::Unknown), "Unknown");
    }
}
