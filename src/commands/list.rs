use crate::config::{Config, Tool};
use crate::sources::{brew::Brew, curl::Curl, mise::Mise};

pub fn run(config: &Config) -> anyhow::Result<()> {
    if config.tools.is_empty() {
        println!("No tools configured in kit.toml");
        println!("Run `kit add <tool>` to add tools or create ~/.config/kit/kit.toml");
        return Ok(());
    }

    println!("Configured tools:\n");

    for (name, tool) in &config.tools {
        let (source, installed) = match tool {
            Tool::Brew { aliases } => {
                let installed = Brew::is_installed(name);
                let aliases_str = if aliases.is_empty() {
                    String::new()
                } else {
                    format!(" [aliases: {}]", aliases.join(", "))
                };
                (format!("brew{}", aliases_str), installed)
            }
            Tool::Mise { version, aliases } => {
                let installed = Mise::is_installed(name, version.as_deref());
                let ver_str = version.as_deref().unwrap_or("latest");
                let aliases_str = if aliases.is_empty() {
                    String::new()
                } else {
                    format!(" [aliases: {}]", aliases.join(", "))
                };
                (format!("mise@{}{}", ver_str, aliases_str), installed)
            }
            Tool::Curl { binary, aliases, .. } => {
                let installed = Curl::is_installed(binary);
                let aliases_str = if aliases.is_empty() {
                    String::new()
                } else {
                    format!(" [aliases: {}]", aliases.join(", "))
                };
                (format!("curl ({}){}", binary, aliases_str), installed)
            }
        };

        let status = if installed { "\u{2713}" } else { "\u{2717}" };
        println!("  {} {} ({})", status, name, source);
    }

    Ok(())
}
