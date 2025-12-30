use anyhow::{Context, Result};
use std::process::Command;

pub struct Mise;

impl Mise {
    pub fn install(name: &str, version: Option<&str>) -> Result<()> {
        let tool_spec = match version {
            Some(v) => format!("{}@{}", name, v),
            None => name.to_string(),
        };

        let status = Command::new("mise")
            .args(["install", &tool_spec])
            .status()
            .context("Failed to run mise")?;

        if status.success() {
            // Also set it as active
            Command::new("mise")
                .args(["use", "--global", &tool_spec])
                .status()
                .ok();
            Ok(())
        } else {
            anyhow::bail!("mise install {} failed", tool_spec)
        }
    }

    pub fn is_installed(name: &str, version: Option<&str>) -> bool {
        let output = Command::new("mise")
            .args(["ls", "--current", "--json"])
            .output()
            .ok();

        if let Some(output) = output {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(obj) = json.as_object() {
                    if obj.contains_key(name) {
                        // If version specified, check it matches
                        if let Some(v) = version {
                            if let Some(arr) = obj.get(name).and_then(|v| v.as_array()) {
                                return arr.iter().any(|entry| {
                                    entry.get("version")
                                        .and_then(|v| v.as_str())
                                        .map(|ver| ver.starts_with(v))
                                        .unwrap_or(false)
                                });
                            }
                        }
                        return true;
                    }
                }
            }
        }
        false
    }
}
