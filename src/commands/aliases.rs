use crate::config::Config;

pub fn run(_config: &Config) -> anyhow::Result<()> {
    println!("Regenerating aliases...");
    Ok(())
}
