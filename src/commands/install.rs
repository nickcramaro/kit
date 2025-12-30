use crate::config::Config;

pub fn run(_config: &Config, tool: Option<String>, all: bool) -> anyhow::Result<()> {
    if all {
        println!("Installing all tools...");
    } else if let Some(name) = tool {
        println!("Installing {}...", name);
    }
    Ok(())
}
