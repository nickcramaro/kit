use anyhow::{Context, Result};
use std::process::Command;

pub struct Curl;

impl Curl {
    pub fn install(install_url: &str, binary: &str) -> Result<()> {
        // Download and execute install script
        let curl_output = Command::new("curl")
            .args(["-fsSL", install_url])
            .output()
            .context("Failed to download install script")?;

        if !curl_output.status.success() {
            anyhow::bail!("Failed to download {}", install_url);
        }

        let status = Command::new("sh")
            .arg("-s")
            .arg("--")
            .arg("-y") // Common flag for non-interactive install
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(&curl_output.stdout)?;
                }
                child.wait()
            })
            .context("Failed to run install script")?;

        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("Install script for {} failed", binary)
        }
    }

    pub fn is_installed(binary: &str) -> bool {
        which::which(binary).is_ok()
    }
}
