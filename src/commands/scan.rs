use crate::config::{Config, Tool};
use crate::scanner::{DetectedSource, Scanner};
use std::collections::HashSet;

pub fn run(config: &Config) -> anyhow::Result<()> {
    let scanner = Scanner::new();
    let binaries = scanner.scan_path();

    let configured_names: HashSet<_> = config.tools.keys().cloned().collect();
    let found_names: HashSet<_> = binaries.iter().map(|b| b.name.clone()).collect();

    // Configured & installed
    let mut installed = Vec::new();
    let mut missing = Vec::new();

    for (name, tool) in &config.tools {
        if found_names.contains(name) {
            installed.push((name.clone(), source_name(tool)));
        } else {
            missing.push((name.clone(), source_name(tool)));
        }
    }

    // Found but not configured
    let unconfigured: Vec<_> = binaries
        .iter()
        .filter(|b| !configured_names.contains(&b.name))
        .collect();

    // Print results
    if !installed.is_empty() {
        println!("Configured & installed:");
        for (name, source) in &installed {
            println!("  \u{2713} {} ({})", name, source);
        }
        println!();
    }

    if !missing.is_empty() {
        println!("Configured but missing:");
        for (name, source) in &missing {
            println!("  \u{2717} {} ({}) - run `kit install {}`", name, source, name);
        }
        println!();
    }

    if !unconfigured.is_empty() {
        println!("Found but not in config:");
        for binary in unconfigured.iter().take(20) {
            let source_str = match binary.source {
                DetectedSource::Brew => "brew",
                DetectedSource::Mise => "mise",
                DetectedSource::Unknown => "unknown",
            };
            println!(
                "  ? {} ({}) - run `kit add {}`",
                binary.name,
                binary.path.display(),
                binary.name
            );
        }
        if unconfigured.len() > 20 {
            println!("  ... and {} more", unconfigured.len() - 20);
        }
    }

    Ok(())
}

fn source_name(tool: &Tool) -> &'static str {
    match tool {
        Tool::Brew { .. } => "brew",
        Tool::Mise { .. } => "mise",
        Tool::Curl { .. } => "curl",
    }
}
