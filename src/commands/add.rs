use crate::config::Config;

pub fn run(_config: &Config, tool: String) -> anyhow::Result<()> {
    println!("Adding {}...", tool);
    Ok(())
}
