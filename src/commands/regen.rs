use crate::config::Config;
use crate::shell::Shell;
use anyhow::Result;
use colored::Colorize;

/// Regenerate aliases and symlinks for all configured tools
pub fn regenerate(config: &Config) -> Result<()> {
    let shell = Shell::new(
        config.config.bin_dir.parent().unwrap().to_path_buf(),
        config.config.shell_rc.clone(),
    );

    // Collect all aliases
    let mut aliases: Vec<(String, String)> = Vec::new();
    for (name, tool) in &config.tools {
        for alias in tool.aliases() {
            aliases.push((alias.clone(), name.clone()));
        }
    }

    // Write aliases file
    shell.write_aliases(&aliases)?;

    // Create symlinks
    for (name, tool) in &config.tools {
        if let Ok(path) = which::which(name) {
            // Create symlink for main binary
            shell.create_symlink(name, &path)?;

            // Create symlinks for aliases
            for alias in tool.aliases() {
                shell.create_symlink(alias, &path)?;
            }
        }
    }

    Ok(())
}

pub fn run(config: &Config) -> Result<()> {
    regenerate(config)?;
    println!(
        "{} Regenerated aliases and symlinks.",
        "\u{2713}".green()
    );
    println!(
        "Run {} if PATH is not configured.",
        "kit setup".cyan()
    );
    Ok(())
}
