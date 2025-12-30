use crate::config::Config;

pub fn run(config: &Config) -> anyhow::Result<()> {
    println!("Configured tools:");
    for (name, tool) in &config.tools {
        println!("  {} ({:?})", name, tool);
    }
    Ok(())
}
