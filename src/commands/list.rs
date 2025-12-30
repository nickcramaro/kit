use crate::config::{Config, Tool};
use crate::sources::{brew::Brew, curl::Curl, mise::Mise};
use colored::Colorize;

pub fn run(config: &Config) -> anyhow::Result<()> {
    if config.tools.is_empty() {
        println!("{}", "No tools configured in kit.toml".yellow());
        println!(
            "Run {} to add tools or create {}",
            "kit add <tool>".cyan(),
            "~/.config/kit/kit.toml".cyan()
        );
        return Ok(());
    }

    println!("{}\n", "Configured tools:".bold());

    for (name, tool) in &config.tools {
        let (source, installed) = match tool {
            Tool::Brew { aliases } => {
                let installed = Brew::is_installed(name);
                let aliases_str = if aliases.is_empty() {
                    String::new()
                } else {
                    format!(" [aliases: {}]", aliases.join(", ").dimmed())
                };
                (format!("{}{}", "brew".blue(), aliases_str), installed)
            }
            Tool::Mise { version, aliases } => {
                let installed = Mise::is_installed(name, version.as_deref());
                let ver_str = version.as_deref().unwrap_or("latest");
                let aliases_str = if aliases.is_empty() {
                    String::new()
                } else {
                    format!(" [aliases: {}]", aliases.join(", ").dimmed())
                };
                (format!("{}@{}{}", "mise".magenta(), ver_str, aliases_str), installed)
            }
            Tool::Curl { binary, aliases, .. } => {
                let installed = Curl::is_installed(binary);
                let aliases_str = if aliases.is_empty() {
                    String::new()
                } else {
                    format!(" [aliases: {}]", aliases.join(", ").dimmed())
                };
                (format!("{} ({}){}", "curl".yellow(), binary, aliases_str), installed)
            }
        };

        let (status, name_colored) = if installed {
            ("\u{2713}".green(), name.green())
        } else {
            ("\u{2717}".red(), name.red())
        };
        println!("  {} {} ({})", status, name_colored, source);
    }

    Ok(())
}
