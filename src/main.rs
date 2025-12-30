mod config;
mod commands;
mod scanner;
mod sources;

use clap::{Parser, Subcommand};
use config::Config;

#[derive(Parser)]
#[command(name = "kit")]
#[command(about = "CLI Tool Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Discover PATH, diff against config
    Scan,
    /// Show configured tools + install status
    List,
    /// Install using source from config
    Install {
        /// Tool name to install
        tool: Option<String>,
        /// Install all tools from config
        #[arg(long)]
        all: bool,
    },
    /// Regenerate aliases + re-inject into shell rc
    Aliases,
    /// Output kit.toml to stdout
    Export,
    /// Detect source, add to config interactively
    Add {
        /// Tool name to add
        tool: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load_or_default();

    match cli.command {
        Commands::Scan => commands::scan::run(&config),
        Commands::List => commands::list::run(&config),
        Commands::Install { tool, all } => commands::install::run(&config, tool, all),
        Commands::Aliases => commands::aliases::run(&config),
        Commands::Export => commands::export::run(&config),
        Commands::Add { tool } => commands::add::run(&config, tool),
    }
}
