use crate::config::Config;

pub fn run(config: &Config) -> anyhow::Result<()> {
    let toml_str = toml::to_string_pretty(config)?;
    println!("{}", toml_str);
    Ok(())
}
