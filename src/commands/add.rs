use crate::commands::regen;
use crate::config::{Config, Tool};
use crate::scanner::{DetectedSource, Scanner};
use anyhow::{bail, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};

pub fn run(config: &Config, tool: String) -> Result<()> {
    // Check if already configured
    if config.tools.contains_key(&tool) {
        bail!("Tool '{}' is already in your config", tool);
    }

    // Try to find the binary
    let scanner = Scanner::new();
    let binaries = scanner.scan_path();
    let found = binaries.iter().find(|b| b.name == tool);

    let detected_source = found.map(|b| &b.source);

    // Present source options
    let sources = ["brew", "mise", "curl"];
    let default = match detected_source {
        Some(DetectedSource::Brew) => 0,
        Some(DetectedSource::Mise) => 1,
        _ => 0,
    };

    let selection = Select::new()
        .with_prompt(format!("Select source for '{}'", tool.cyan()))
        .items(&sources)
        .default(default)
        .interact()?;

    let new_tool = match sources[selection] {
        "brew" => {
            let aliases: String = Input::new()
                .with_prompt("Aliases (comma-separated, or empty)")
                .allow_empty(true)
                .interact_text()?;

            let aliases: Vec<String> = aliases
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            Tool::Brew { aliases }
        }
        "mise" => {
            let version: String = Input::new()
                .with_prompt("Version (or empty for latest)")
                .allow_empty(true)
                .interact_text()?;

            let version = if version.is_empty() {
                None
            } else {
                Some(version)
            };

            let aliases: String = Input::new()
                .with_prompt("Aliases (comma-separated, or empty)")
                .allow_empty(true)
                .interact_text()?;

            let aliases: Vec<String> = aliases
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            Tool::Mise { version, aliases }
        }
        "curl" => {
            let install_url: String = Input::new()
                .with_prompt("Install URL")
                .interact_text()?;

            let binary: String = Input::new()
                .with_prompt("Binary name (to verify installation)")
                .default(tool.clone())
                .interact_text()?;

            let aliases: String = Input::new()
                .with_prompt("Aliases (comma-separated, or empty)")
                .allow_empty(true)
                .interact_text()?;

            let aliases: Vec<String> = aliases
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            Tool::Curl {
                install_url,
                binary,
                aliases,
            }
        }
        _ => unreachable!(),
    };

    // Confirm and save
    println!("\n{}", "Will add to kit.toml:".bold());
    println!("  {}", format!("[tools.{}]", tool).cyan());
    println!("  {:?}", new_tool);

    if Confirm::new()
        .with_prompt("Add this tool?")
        .default(true)
        .interact()?
    {
        let mut config = config.clone();
        config.tools.insert(tool.clone(), new_tool);
        config.save()?;
        regen::regenerate(&config)?;
        println!(
            "\n{} Added '{}' to kit.toml",
            "\u{2713}".green(),
            tool.green().bold()
        );
        println!(
            "Run {} to install it",
            format!("kit install {}", tool).cyan()
        );
    } else {
        println!("{}", "Cancelled".yellow());
    }

    Ok(())
}
