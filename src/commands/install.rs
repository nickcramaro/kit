use crate::config::{Config, Tool};
use crate::sources::{brew::Brew, curl::Curl, mise::Mise};
use anyhow::{bail, Result};

pub fn run(config: &Config, tool: Option<String>, all: bool) -> Result<()> {
    if all {
        install_all(config)
    } else if let Some(name) = tool {
        install_one(config, &name)
    } else {
        bail!("Specify a tool name or use --all")
    }
}

fn install_all(config: &Config) -> Result<()> {
    let mut success = 0;
    let mut failed = 0;

    for (name, tool) in &config.tools {
        print!("Installing {}... ", name);
        match install_tool(name, tool) {
            Ok(()) => {
                println!("\u{2713}");
                success += 1;
            }
            Err(e) => {
                println!("\u{2717} {}", e);
                failed += 1;
            }
        }
    }

    println!("\nInstalled: {}, Failed: {}", success, failed);
    Ok(())
}

fn install_one(config: &Config, name: &str) -> Result<()> {
    let tool = config
        .tools
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("Tool '{}' not found in config", name))?;

    install_tool(name, tool)?;
    println!("\u{2713} {} installed successfully", name);
    Ok(())
}

fn install_tool(name: &str, tool: &Tool) -> Result<()> {
    match tool {
        Tool::Brew { .. } => {
            if Brew::is_installed(name) {
                return Ok(());
            }
            Brew::install(name)
        }
        Tool::Mise { version, .. } => {
            if Mise::is_installed(name, version.as_deref()) {
                return Ok(());
            }
            Mise::install(name, version.as_deref())
        }
        Tool::Curl {
            install_url,
            binary,
            ..
        } => {
            if Curl::is_installed(binary) {
                return Ok(());
            }
            Curl::install(install_url, binary)
        }
    }
}
