mod commands;
mod config;
mod scanner;
mod shell;
mod sources;

use clap::{builder::styling, Parser, Subcommand};
use colored::*;
use config::Config;

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Cyan.on_default().bold())
    .placeholder(styling::AnsiColor::Yellow.on_default());

#[derive(Parser)]
#[command(name = "kit")]
#[command(version)]
#[command(about = "CLI Tool Manager - track, install, and manage your command-line tools")]
#[command(long_about = None)]
#[command(styles = STYLES)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initial setup: check dependencies, inject shell rc
    Setup,
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
    /// Regenerate aliases and symlinks
    Regen,
    /// Output kit.toml to stdout
    Export,
    /// Detect source, add to config interactively
    Add {
        /// Tool name to add
        tool: String,
    },
    /// Update kit to the latest version
    Update,
}

fn print_banner() {
    println!(
        "{}",
        r#"  _    _ _
 | | _(_) |_
 | |/ / | __|
 |   <| | |_
 |_|\_\_|\__|"#
            .green()
            .bold()
    );
    println!();
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        for cause in e.chain().skip(1) {
            eprintln!("  Caused by: {}", cause);
        }
        std::process::exit(1);
    }
}

fn check_for_updates() {
    if let Ok(Some(version)) = commands::update::check_for_update() {
        eprintln!(
            "\n\x1b[33m📦 New version available: {}\x1b[0m",
            version
        );
        eprintln!("   Run \x1b[1mkit update\x1b[0m to upgrade\n");
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load_or_default();

    match cli.command {
        None => {
            print_banner();
            let tool_count = config.tools.len();
            let installed = config
                .tools
                .keys()
                .filter(|name| which::which(name).is_ok())
                .count();
            println!(
                "  {} {}",
                "Tools:".white().bold(),
                format!("{}/{} installed", installed, tool_count).cyan()
            );
            println!(
                "  {} {}",
                "Config:".white().bold(),
                Config::config_path().display().to_string().cyan()
            );
            println!(
                "  {} {}",
                "Version:".white().bold(),
                env!("CARGO_PKG_VERSION").cyan()
            );
            println!();
            println!(
                "  Run {} for available commands.",
                "kit --help".cyan().bold()
            );
            println!();
            check_for_updates();
            Ok(())
        }
        Some(cmd) => {
            match &cmd {
                Commands::Setup | Commands::List => print_banner(),
                _ => {}
            }
            match cmd {
                Commands::Setup => commands::setup::run(&config),
                Commands::List => commands::list::run(&config),
                Commands::Install { tool, all } => commands::install::run(&config, tool, all),
                Commands::Regen => commands::regen::run(&config),
                Commands::Export => commands::export::run(&config),
                Commands::Add { tool } => commands::add::run(&config, tool),
                Commands::Update => commands::update::run(),
            }
        }
    }
}
