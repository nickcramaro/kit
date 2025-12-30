use crate::config::{Config, Tool};
use crate::shell::Shell;
use anyhow::{bail, Result};
use std::collections::HashSet;

pub fn run(config: &Config) -> Result<()> {
    let config_path = Config::config_path();

    // Check if config exists
    if !config_path.exists() {
        println!("No config found at {:?}", config_path);
        println!("Run `kit add <tool>` to add your first tool.\n");
    } else {
        println!("Config: {:?}", config_path);
        println!("Tools configured: {}\n", config.tools.len());
    }

    // Collect required sources from configured tools
    let mut required_sources: HashSet<&str> = HashSet::new();
    for tool in config.tools.values() {
        match tool {
            Tool::Brew { .. } => { required_sources.insert("brew"); }
            Tool::Mise { .. } => { required_sources.insert("mise"); }
            Tool::Curl { .. } => { required_sources.insert("curl"); }
        }
    }

    // Check each required source is installed
    let mut missing: Vec<&str> = Vec::new();
    for source in &required_sources {
        if which::which(source).is_err() {
            missing.push(source);
        }
    }

    if !missing.is_empty() {
        println!("Missing dependencies:");
        for dep in &missing {
            let install_hint = match *dep {
                "brew" => "Install from https://brew.sh",
                "mise" => "Install from https://mise.jdx.dev",
                "curl" => "Install via your system package manager",
                _ => "",
            };
            println!("  {} - {}", dep, install_hint);
        }
        println!();
        bail!("Install missing dependencies and run `kit setup` again");
    }

    if !required_sources.is_empty() {
        println!("Dependencies OK: {}", required_sources.into_iter().collect::<Vec<_>>().join(", "));
    }

    // Inject into shell rc
    let shell = Shell::new(
        config.config.bin_dir.parent().unwrap().to_path_buf(),
        config.config.shell_rc.clone(),
    );

    shell.inject_rc()?;
    println!("Updated {:?}", config.config.shell_rc);

    println!("\nSetup complete! Run `source {:?}` or start a new shell.", config.config.shell_rc);

    Ok(())
}
