use crate::config::Config;

pub fn run(_config: &Config) -> anyhow::Result<()> {
    println!("Scanning PATH...");
    Ok(())
}
