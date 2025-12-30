use anyhow::{Context, Result};
use std::process::Command;

pub struct Brew;

impl Brew {
    pub fn install(name: &str) -> Result<()> {
        let status = Command::new("brew")
            .args(["install", name])
            .status()
            .context("Failed to run brew")?;

        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("brew install {} failed", name)
        }
    }

    pub fn is_installed(name: &str) -> bool {
        Command::new("brew")
            .args(["list", name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
