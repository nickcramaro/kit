# Kit CLI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust CLI tool for tracking, installing, and managing command-line tools across brew, mise, and curl sources.

**Architecture:** Clap-based CLI with subcommands. TOML config at `~/.config/kit/kit.toml`. Source-specific modules handle detection and installation. Shell integration via generated alias files and managed zshrc injection.

**Tech Stack:** Rust, clap (CLI), serde + toml (config), walkdir (PATH scanning), which (binary resolution), dialoguer (interactive prompts)

---

## Phase 1: Project Setup & Config

### Task 1: Initialize Rust Project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

**Step 1: Initialize cargo project**

Run:
```bash
cargo init
```

**Step 2: Update Cargo.toml with dependencies**

Replace `Cargo.toml`:
```toml
[package]
name = "kit"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
walkdir = "2"
which = "6"
dialoguer = "0.11"
dirs = "5"
thiserror = "1"
```

**Step 3: Verify dependencies resolve**

Run:
```bash
cargo check
```
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs
git commit -m "chore: initialize Rust project with dependencies"
```

---

### Task 2: Define Config Types

**Files:**
- Create: `src/config.rs`
- Modify: `src/main.rs`

**Step 1: Write the failing test for config parsing**

Create `src/config.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub config: GlobalConfig,
    #[serde(default)]
    pub tools: HashMap<String, Tool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GlobalConfig {
    #[serde(default = "default_bin_dir")]
    pub bin_dir: PathBuf,
    #[serde(default = "default_shell_rc")]
    pub shell_rc: PathBuf,
}

fn default_bin_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".kit")
        .join("bin")
}

fn default_shell_rc() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".zshrc")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "source", rename_all = "lowercase")]
pub enum Tool {
    Brew {
        #[serde(default)]
        aliases: Vec<String>,
    },
    Mise {
        version: Option<String>,
        #[serde(default)]
        aliases: Vec<String>,
    },
    Curl {
        install_url: String,
        binary: String,
        #[serde(default)]
        aliases: Vec<String>,
    },
}

impl Tool {
    pub fn aliases(&self) -> &[String] {
        match self {
            Tool::Brew { aliases } => aliases,
            Tool::Mise { aliases, .. } => aliases,
            Tool::Curl { aliases, .. } => aliases,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let toml_str = r#"
[tools.ripgrep]
source = "brew"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.tools.contains_key("ripgrep"));
        assert!(matches!(config.tools["ripgrep"], Tool::Brew { .. }));
    }

    #[test]
    fn test_parse_full_config() {
        let toml_str = r#"
[config]
bin_dir = "~/.kit/bin"
shell_rc = "~/.zshrc"

[tools.ripgrep]
source = "brew"
aliases = ["rg"]

[tools.python]
source = "mise"
version = "3.12"

[tools.starship]
source = "curl"
install_url = "https://starship.rs/install.sh"
binary = "starship"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.tools.len(), 3);

        match &config.tools["ripgrep"] {
            Tool::Brew { aliases } => assert_eq!(aliases, &["rg"]),
            _ => panic!("Expected Brew tool"),
        }

        match &config.tools["python"] {
            Tool::Mise { version, .. } => assert_eq!(version, &Some("3.12".to_string())),
            _ => panic!("Expected Mise tool"),
        }

        match &config.tools["starship"] {
            Tool::Curl { install_url, binary, .. } => {
                assert_eq!(install_url, "https://starship.rs/install.sh");
                assert_eq!(binary, "starship");
            }
            _ => panic!("Expected Curl tool"),
        }
    }
}
```

**Step 2: Update main.rs to include module**

Replace `src/main.rs`:
```rust
mod config;

fn main() {
    println!("kit - CLI Tool Manager");
}
```

**Step 3: Run tests to verify they pass**

Run:
```bash
cargo test
```
Expected: 2 tests pass

**Step 4: Commit**

```bash
git add src/config.rs src/main.rs
git commit -m "feat: add config types with TOML parsing"
```

---

### Task 3: Add Config Loading from File

**Files:**
- Modify: `src/config.rs`

**Step 1: Add config loading function and test**

Add to `src/config.rs` after the `Tool` impl block:
```rust
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("Config file not found at {0}")]
    NotFound(PathBuf),
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("kit")
            .join("kit.toml")
    }

    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        Self::load_from(&path)
    }

    pub fn load_from(path: &PathBuf) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::NotFound(path.clone()));
        }
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config: GlobalConfig::default(),
            tools: HashMap::new(),
        }
    }
}
```

**Step 2: Add test for load_from**

Add to the tests module in `src/config.rs`:
```rust
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"
[tools.ripgrep]
source = "brew"
aliases = ["rg"]
"#).unwrap();

        let config = Config::load_from(&file.path().to_path_buf()).unwrap();
        assert!(config.tools.contains_key("ripgrep"));
    }
```

**Step 3: Add tempfile dev dependency**

Add to `Cargo.toml` at the end:
```toml
[dev-dependencies]
tempfile = "3"
```

**Step 4: Run tests**

Run:
```bash
cargo test
```
Expected: 3 tests pass

**Step 5: Commit**

```bash
git add src/config.rs Cargo.toml Cargo.lock
git commit -m "feat: add config file loading"
```

---

## Phase 2: CLI Framework

### Task 4: Set Up Clap CLI Structure

**Files:**
- Modify: `src/main.rs`
- Create: `src/commands/mod.rs`

**Step 1: Create commands module**

Create `src/commands/mod.rs`:
```rust
pub mod list;
pub mod scan;
pub mod install;
pub mod aliases;
pub mod export;
pub mod add;
```

**Step 2: Create placeholder command modules**

Create `src/commands/list.rs`:
```rust
use crate::config::Config;

pub fn run(config: &Config) -> anyhow::Result<()> {
    println!("Configured tools:");
    for (name, tool) in &config.tools {
        println!("  {} ({:?})", name, tool);
    }
    Ok(())
}
```

Create `src/commands/scan.rs`:
```rust
use crate::config::Config;

pub fn run(_config: &Config) -> anyhow::Result<()> {
    println!("Scanning PATH...");
    Ok(())
}
```

Create `src/commands/install.rs`:
```rust
use crate::config::Config;

pub fn run(_config: &Config, tool: Option<String>, all: bool) -> anyhow::Result<()> {
    if all {
        println!("Installing all tools...");
    } else if let Some(name) = tool {
        println!("Installing {}...", name);
    }
    Ok(())
}
```

Create `src/commands/aliases.rs`:
```rust
use crate::config::Config;

pub fn run(_config: &Config) -> anyhow::Result<()> {
    println!("Regenerating aliases...");
    Ok(())
}
```

Create `src/commands/export.rs`:
```rust
use crate::config::Config;

pub fn run(config: &Config) -> anyhow::Result<()> {
    let toml_str = toml::to_string_pretty(config)?;
    println!("{}", toml_str);
    Ok(())
}
```

Create `src/commands/add.rs`:
```rust
use crate::config::Config;

pub fn run(_config: &Config, tool: String) -> anyhow::Result<()> {
    println!("Adding {}...", tool);
    Ok(())
}
```

**Step 3: Update main.rs with clap CLI**

Replace `src/main.rs`:
```rust
mod config;
mod commands;

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
```

**Step 4: Add anyhow dependency**

Add to `[dependencies]` in `Cargo.toml`:
```toml
anyhow = "1"
```

**Step 5: Verify it compiles and runs**

Run:
```bash
cargo build
./target/debug/kit --help
./target/debug/kit list
```
Expected: Help shows all commands, list runs without error

**Step 6: Commit**

```bash
git add src/ Cargo.toml Cargo.lock
git commit -m "feat: add clap CLI with subcommand structure"
```

---

## Phase 3: Scanner Module

### Task 5: Implement PATH Scanner

**Files:**
- Create: `src/scanner.rs`
- Modify: `src/main.rs`

**Step 1: Create scanner with tests**

Create `src/scanner.rs`:
```rust
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct FoundBinary {
    pub name: String,
    pub path: PathBuf,
    pub source: DetectedSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DetectedSource {
    Brew,
    Mise,
    Unknown,
}

pub struct Scanner {
    brew_formulas: HashSet<String>,
    mise_tools: HashSet<String>,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            brew_formulas: Self::get_brew_formulas(),
            mise_tools: Self::get_mise_tools(),
        }
    }

    fn get_brew_formulas() -> HashSet<String> {
        std::process::Command::new("brew")
            .args(["list", "--formula", "-1"])
            .output()
            .ok()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_mise_tools() -> HashSet<String> {
        std::process::Command::new("mise")
            .args(["ls", "--current", "--json"])
            .output()
            .ok()
            .and_then(|output| {
                serde_json::from_slice::<HashMap<String, serde_json::Value>>(&output.stdout).ok()
            })
            .map(|map| map.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn scan_path(&self) -> Vec<FoundBinary> {
        let path_var = env::var("PATH").unwrap_or_default();
        let mut seen: HashSet<String> = HashSet::new();
        let mut binaries = Vec::new();

        for dir in path_var.split(':') {
            let dir_path = PathBuf::from(dir);
            if !dir_path.is_dir() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }

                    // Check if executable
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(meta) = path.metadata() {
                            if meta.permissions().mode() & 0o111 == 0 {
                                continue;
                            }
                        }
                    }

                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let name = name.to_string();
                        if seen.contains(&name) {
                            continue;
                        }
                        seen.insert(name.clone());

                        let source = self.detect_source(&name, &path);
                        binaries.push(FoundBinary { name, path, source });
                    }
                }
            }
        }

        binaries
    }

    fn detect_source(&self, name: &str, path: &PathBuf) -> DetectedSource {
        // Check if it's a brew formula
        if self.brew_formulas.contains(name) {
            return DetectedSource::Brew;
        }

        // Check if path contains mise
        if path.to_string_lossy().contains("mise") {
            return DetectedSource::Mise;
        }

        // Check for common mise tool names
        if self.mise_tools.contains(name) {
            return DetectedSource::Mise;
        }

        DetectedSource::Unknown
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_finds_binaries() {
        let scanner = Scanner::new();
        let binaries = scanner.scan_path();
        // Should find at least some common binaries
        assert!(!binaries.is_empty());
    }

    #[test]
    fn test_detected_source_display() {
        assert_eq!(format!("{:?}", DetectedSource::Brew), "Brew");
        assert_eq!(format!("{:?}", DetectedSource::Mise), "Mise");
        assert_eq!(format!("{:?}", DetectedSource::Unknown), "Unknown");
    }
}
```

**Step 2: Add serde_json dependency**

Add to `[dependencies]` in `Cargo.toml`:
```toml
serde_json = "1"
```

**Step 3: Add module to main.rs**

Add after `mod commands;` in `src/main.rs`:
```rust
mod scanner;
```

**Step 4: Run tests**

Run:
```bash
cargo test
```
Expected: All tests pass

**Step 5: Commit**

```bash
git add src/scanner.rs src/main.rs Cargo.toml Cargo.lock
git commit -m "feat: add PATH scanner with source detection"
```

---

### Task 6: Implement Scan Command

**Files:**
- Modify: `src/commands/scan.rs`

**Step 1: Implement full scan command**

Replace `src/commands/scan.rs`:
```rust
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
```

**Step 2: Test scan command**

Run:
```bash
cargo run -- scan
```
Expected: Shows binaries found in PATH

**Step 3: Commit**

```bash
git add src/commands/scan.rs
git commit -m "feat: implement scan command with diff output"
```

---

## Phase 4: Install & Sources

### Task 7: Implement Source Modules

**Files:**
- Create: `src/sources/mod.rs`
- Create: `src/sources/brew.rs`
- Create: `src/sources/mise.rs`
- Create: `src/sources/curl.rs`
- Modify: `src/main.rs`

**Step 1: Create sources module**

Create `src/sources/mod.rs`:
```rust
pub mod brew;
pub mod curl;
pub mod mise;

use anyhow::Result;

pub trait Source {
    fn install(&self, name: &str) -> Result<()>;
    fn is_installed(&self, name: &str) -> bool;
}
```

**Step 2: Create brew source**

Create `src/sources/brew.rs`:
```rust
use anyhow::{Context, Result};
use std::process::Command;

pub struct Brew;

impl Brew {
    pub fn install(name: &str) -> Result<()> {
        let status = Command::new("brew")
            .args(["install", name])
            .status()
            .context("Failed to run brew")?;

        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("brew install {} failed", name)
        }
    }

    pub fn is_installed(name: &str) -> bool {
        Command::new("brew")
            .args(["list", name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
```

**Step 3: Create mise source**

Create `src/sources/mise.rs`:
```rust
use anyhow::{Context, Result};
use std::process::Command;

pub struct Mise;

impl Mise {
    pub fn install(name: &str, version: Option<&str>) -> Result<()> {
        let tool_spec = match version {
            Some(v) => format!("{}@{}", name, v),
            None => name.to_string(),
        };

        let status = Command::new("mise")
            .args(["install", &tool_spec])
            .status()
            .context("Failed to run mise")?;

        if status.success() {
            // Also set it as active
            Command::new("mise")
                .args(["use", "--global", &tool_spec])
                .status()
                .ok();
            Ok(())
        } else {
            anyhow::bail!("mise install {} failed", tool_spec)
        }
    }

    pub fn is_installed(name: &str, version: Option<&str>) -> bool {
        let output = Command::new("mise")
            .args(["ls", "--current", "--json"])
            .output()
            .ok();

        if let Some(output) = output {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(obj) = json.as_object() {
                    if obj.contains_key(name) {
                        // If version specified, check it matches
                        if let Some(v) = version {
                            if let Some(arr) = obj.get(name).and_then(|v| v.as_array()) {
                                return arr.iter().any(|entry| {
                                    entry.get("version")
                                        .and_then(|v| v.as_str())
                                        .map(|ver| ver.starts_with(v))
                                        .unwrap_or(false)
                                });
                            }
                        }
                        return true;
                    }
                }
            }
        }
        false
    }
}
```

**Step 4: Create curl source**

Create `src/sources/curl.rs`:
```rust
use anyhow::{Context, Result};
use std::process::Command;

pub struct Curl;

impl Curl {
    pub fn install(install_url: &str, binary: &str) -> Result<()> {
        // Download and execute install script
        let curl_output = Command::new("curl")
            .args(["-fsSL", install_url])
            .output()
            .context("Failed to download install script")?;

        if !curl_output.status.success() {
            anyhow::bail!("Failed to download {}", install_url);
        }

        let status = Command::new("sh")
            .arg("-s")
            .arg("--")
            .arg("-y") // Common flag for non-interactive install
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(&curl_output.stdout)?;
                }
                child.wait()
            })
            .context("Failed to run install script")?;

        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("Install script for {} failed", binary)
        }
    }

    pub fn is_installed(binary: &str) -> bool {
        which::which(binary).is_ok()
    }
}
```

**Step 5: Add module to main.rs**

Add after `mod scanner;` in `src/main.rs`:
```rust
mod sources;
```

**Step 6: Verify it compiles**

Run:
```bash
cargo check
```
Expected: Compiles successfully

**Step 7: Commit**

```bash
git add src/sources/ src/main.rs
git commit -m "feat: add source modules for brew, mise, curl"
```

---

### Task 8: Implement Install Command

**Files:**
- Modify: `src/commands/install.rs`

**Step 1: Implement install command**

Replace `src/commands/install.rs`:
```rust
use crate::config::{Config, Tool};
use crate::sources::{brew::Brew, curl::Curl, mise::Mise};
use anyhow::{bail, Result};

pub fn run(config: &Config, tool: Option<String>, all: bool) -> Result<()> {
    if all {
        install_all(config)
    } else if let Some(name) = tool {
        install_one(config, &name)
    } else {
        bail!("Specify a tool name or use --all")
    }
}

fn install_all(config: &Config) -> Result<()> {
    let mut success = 0;
    let mut failed = 0;

    for (name, tool) in &config.tools {
        print!("Installing {}... ", name);
        match install_tool(name, tool) {
            Ok(()) => {
                println!("\u{2713}");
                success += 1;
            }
            Err(e) => {
                println!("\u{2717} {}", e);
                failed += 1;
            }
        }
    }

    println!("\nInstalled: {}, Failed: {}", success, failed);
    Ok(())
}

fn install_one(config: &Config, name: &str) -> Result<()> {
    let tool = config
        .tools
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("Tool '{}' not found in config", name))?;

    install_tool(name, tool)?;
    println!("\u{2713} {} installed successfully", name);
    Ok(())
}

fn install_tool(name: &str, tool: &Tool) -> Result<()> {
    match tool {
        Tool::Brew { .. } => {
            if Brew::is_installed(name) {
                return Ok(());
            }
            Brew::install(name)
        }
        Tool::Mise { version, .. } => {
            if Mise::is_installed(name, version.as_deref()) {
                return Ok(());
            }
            Mise::install(name, version.as_deref())
        }
        Tool::Curl {
            install_url,
            binary,
            ..
        } => {
            if Curl::is_installed(binary) {
                return Ok(());
            }
            Curl::install(install_url, binary)
        }
    }
}
```

**Step 2: Test install command (dry run)**

Run:
```bash
cargo run -- install --help
```
Expected: Shows install usage

**Step 3: Commit**

```bash
git add src/commands/install.rs
git commit -m "feat: implement install command with source dispatch"
```

---

## Phase 5: Shell Integration

### Task 9: Implement Shell Module

**Files:**
- Create: `src/shell.rs`
- Modify: `src/main.rs`

**Step 1: Create shell module with tests**

Create `src/shell.rs`:
```rust
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const MARKER_START: &str = "# >>> kit >>>";
const MARKER_END: &str = "# <<< kit <<<";

pub struct Shell {
    pub kit_dir: PathBuf,
    pub shell_rc: PathBuf,
}

impl Shell {
    pub fn new(kit_dir: PathBuf, shell_rc: PathBuf) -> Self {
        Self { kit_dir, shell_rc }
    }

    pub fn aliases_path(&self) -> PathBuf {
        self.kit_dir.join("aliases.zsh")
    }

    pub fn bin_dir(&self) -> PathBuf {
        self.kit_dir.join("bin")
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.kit_dir)?;
        fs::create_dir_all(self.bin_dir())?;
        Ok(())
    }

    pub fn write_aliases(&self, aliases: &[(String, String)]) -> Result<()> {
        self.ensure_dirs()?;

        let mut content = String::from("# ~/.kit/aliases.zsh - managed by kit, do not edit\n");
        for (alias, target) in aliases {
            content.push_str(&format!("alias {}=\"{}\"\n", alias, target));
        }

        fs::write(self.aliases_path(), content)?;
        Ok(())
    }

    pub fn inject_rc(&self) -> Result<()> {
        let home = dirs::home_dir().unwrap_or_default();
        let kit_dir_str = self.kit_dir.to_string_lossy().replace(
            &home.to_string_lossy().to_string(),
            "$HOME",
        );

        let injection = format!(
            r#"{marker_start}
export PATH="{kit_dir}/bin:$PATH"
source "{kit_dir}/aliases.zsh"
{marker_end}"#,
            marker_start = MARKER_START,
            marker_end = MARKER_END,
            kit_dir = kit_dir_str,
        );

        // Read existing rc file
        let existing = fs::read_to_string(&self.shell_rc).unwrap_or_default();

        // Check if already injected
        if existing.contains(MARKER_START) {
            // Replace existing injection
            let before = existing
                .split(MARKER_START)
                .next()
                .unwrap_or("");
            let after = existing
                .split(MARKER_END)
                .nth(1)
                .unwrap_or("");

            let new_content = format!("{}{}{}", before, injection, after);
            fs::write(&self.shell_rc, new_content)?;
        } else {
            // Append new injection
            let new_content = format!("{}\n{}\n", existing.trim_end(), injection);
            fs::write(&self.shell_rc, new_content)?;
        }

        Ok(())
    }

    pub fn create_symlink(&self, name: &str, target: &PathBuf) -> Result<()> {
        self.ensure_dirs()?;
        let link_path = self.bin_dir().join(name);

        // Remove existing symlink if present
        if link_path.exists() || link_path.is_symlink() {
            fs::remove_file(&link_path).ok();
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(target, &link_path)
            .context(format!("Failed to create symlink for {}", name))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_aliases() {
        let tmp = TempDir::new().unwrap();
        let shell = Shell::new(
            tmp.path().join(".kit"),
            tmp.path().join(".zshrc"),
        );

        let aliases = vec![
            ("rg".to_string(), "ripgrep".to_string()),
            ("cat".to_string(), "bat".to_string()),
        ];

        shell.write_aliases(&aliases).unwrap();

        let content = fs::read_to_string(shell.aliases_path()).unwrap();
        assert!(content.contains("alias rg=\"ripgrep\""));
        assert!(content.contains("alias cat=\"bat\""));
    }

    #[test]
    fn test_inject_rc_new() {
        let tmp = TempDir::new().unwrap();
        let rc_path = tmp.path().join(".zshrc");
        fs::write(&rc_path, "# existing content\n").unwrap();

        let shell = Shell::new(tmp.path().join(".kit"), rc_path.clone());
        shell.inject_rc().unwrap();

        let content = fs::read_to_string(&rc_path).unwrap();
        assert!(content.contains("# >>> kit >>>"));
        assert!(content.contains("# <<< kit <<<"));
        assert!(content.contains("# existing content"));
    }

    #[test]
    fn test_inject_rc_replace() {
        let tmp = TempDir::new().unwrap();
        let rc_path = tmp.path().join(".zshrc");
        fs::write(
            &rc_path,
            "before\n# >>> kit >>>\nold stuff\n# <<< kit <<<\nafter\n",
        )
        .unwrap();

        let shell = Shell::new(tmp.path().join(".kit"), rc_path.clone());
        shell.inject_rc().unwrap();

        let content = fs::read_to_string(&rc_path).unwrap();
        assert!(content.contains("before"));
        assert!(content.contains("after"));
        assert!(!content.contains("old stuff"));
        assert_eq!(content.matches("# >>> kit >>>").count(), 1);
    }
}
```

**Step 2: Add module to main.rs**

Add after `mod sources;` in `src/main.rs`:
```rust
mod shell;
```

**Step 3: Run tests**

Run:
```bash
cargo test
```
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/shell.rs src/main.rs
git commit -m "feat: add shell integration module"
```

---

### Task 10: Implement Aliases Command

**Files:**
- Modify: `src/commands/aliases.rs`

**Step 1: Implement aliases command**

Replace `src/commands/aliases.rs`:
```rust
use crate::config::Config;
use crate::shell::Shell;
use anyhow::Result;

pub fn run(config: &Config) -> Result<()> {
    let shell = Shell::new(
        config.config.bin_dir.parent().unwrap().to_path_buf(),
        config.config.shell_rc.clone(),
    );

    // Collect all aliases
    let mut aliases: Vec<(String, String)> = Vec::new();
    for (name, tool) in &config.tools {
        for alias in tool.aliases() {
            aliases.push((alias.clone(), name.clone()));
        }
    }

    // Write aliases file
    shell.write_aliases(&aliases)?;
    println!("Wrote {} aliases to {:?}", aliases.len(), shell.aliases_path());

    // Create symlinks
    for (name, _tool) in &config.tools {
        if let Ok(path) = which::which(name) {
            // Create symlink for main binary
            shell.create_symlink(name, &path)?;

            // Create symlinks for aliases
            for alias in _tool.aliases() {
                shell.create_symlink(alias, &path)?;
            }
        }
    }

    // Inject into shell rc
    shell.inject_rc()?;
    println!("Updated {:?}", config.config.shell_rc);

    println!("\nRun `source {:?}` or start a new shell to apply changes", config.config.shell_rc);

    Ok(())
}
```

**Step 2: Verify it compiles**

Run:
```bash
cargo check
```
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/commands/aliases.rs
git commit -m "feat: implement aliases command"
```

---

## Phase 6: List & Add Commands

### Task 11: Implement List Command

**Files:**
- Modify: `src/commands/list.rs`

**Step 1: Implement list command with install status**

Replace `src/commands/list.rs`:
```rust
use crate::config::{Config, Tool};
use crate::sources::{brew::Brew, curl::Curl, mise::Mise};

pub fn run(config: &Config) -> anyhow::Result<()> {
    if config.tools.is_empty() {
        println!("No tools configured in kit.toml");
        println!("Run `kit add <tool>` to add tools or create ~/.config/kit/kit.toml");
        return Ok(());
    }

    println!("Configured tools:\n");

    for (name, tool) in &config.tools {
        let (source, installed) = match tool {
            Tool::Brew { aliases } => {
                let installed = Brew::is_installed(name);
                let aliases_str = if aliases.is_empty() {
                    String::new()
                } else {
                    format!(" [aliases: {}]", aliases.join(", "))
                };
                (format!("brew{}", aliases_str), installed)
            }
            Tool::Mise { version, aliases } => {
                let installed = Mise::is_installed(name, version.as_deref());
                let ver_str = version.as_deref().unwrap_or("latest");
                let aliases_str = if aliases.is_empty() {
                    String::new()
                } else {
                    format!(" [aliases: {}]", aliases.join(", "))
                };
                (format!("mise@{}{}", ver_str, aliases_str), installed)
            }
            Tool::Curl { binary, aliases, .. } => {
                let installed = Curl::is_installed(binary);
                let aliases_str = if aliases.is_empty() {
                    String::new()
                } else {
                    format!(" [aliases: {}]", aliases.join(", "))
                };
                (format!("curl ({}){}", binary, aliases_str), installed)
            }
        };

        let status = if installed { "\u{2713}" } else { "\u{2717}" };
        println!("  {} {} ({})", status, name, source);
    }

    Ok(())
}
```

**Step 2: Verify it compiles**

Run:
```bash
cargo check
```
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/commands/list.rs
git commit -m "feat: implement list command with install status"
```

---

### Task 12: Implement Add Command

**Files:**
- Modify: `src/commands/add.rs`
- Modify: `src/config.rs`

**Step 1: Add config save method**

Add to `src/config.rs` inside `impl Config`:
```rust
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::config_path();
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &PathBuf) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::Parse(toml::de::Error::custom(e.to_string())))?;
        std::fs::write(path, contents)?;
        Ok(())
    }
```

Also add this import at the top of `src/config.rs`:
```rust
use serde::de::Error as _;
```

**Step 2: Implement add command**

Replace `src/commands/add.rs`:
```rust
use crate::config::{Config, Tool};
use crate::scanner::{DetectedSource, Scanner};
use anyhow::{bail, Result};
use dialoguer::{Confirm, Input, Select};

pub fn run(config: &Config, tool: String) -> Result<()> {
    // Check if already configured
    if config.tools.contains_key(&tool) {
        bail!("Tool '{}' is already in your config", tool);
    }

    // Try to find the binary
    let scanner = Scanner::new();
    let binaries = scanner.scan_path();
    let found = binaries.iter().find(|b| b.name == tool);

    let detected_source = found.map(|b| &b.source);

    // Present source options
    let sources = ["brew", "mise", "curl"];
    let default = match detected_source {
        Some(DetectedSource::Brew) => 0,
        Some(DetectedSource::Mise) => 1,
        _ => 0,
    };

    let selection = Select::new()
        .with_prompt(format!("Select source for '{}'", tool))
        .items(&sources)
        .default(default)
        .interact()?;

    let new_tool = match sources[selection] {
        "brew" => {
            let aliases: String = Input::new()
                .with_prompt("Aliases (comma-separated, or empty)")
                .allow_empty(true)
                .interact_text()?;

            let aliases: Vec<String> = aliases
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            Tool::Brew { aliases }
        }
        "mise" => {
            let version: String = Input::new()
                .with_prompt("Version (or empty for latest)")
                .allow_empty(true)
                .interact_text()?;

            let version = if version.is_empty() {
                None
            } else {
                Some(version)
            };

            let aliases: String = Input::new()
                .with_prompt("Aliases (comma-separated, or empty)")
                .allow_empty(true)
                .interact_text()?;

            let aliases: Vec<String> = aliases
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            Tool::Mise { version, aliases }
        }
        "curl" => {
            let install_url: String = Input::new()
                .with_prompt("Install URL")
                .interact_text()?;

            let binary: String = Input::new()
                .with_prompt("Binary name (to verify installation)")
                .default(tool.clone())
                .interact_text()?;

            let aliases: String = Input::new()
                .with_prompt("Aliases (comma-separated, or empty)")
                .allow_empty(true)
                .interact_text()?;

            let aliases: Vec<String> = aliases
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            Tool::Curl {
                install_url,
                binary,
                aliases,
            }
        }
        _ => unreachable!(),
    };

    // Confirm and save
    println!("\nWill add to kit.toml:");
    println!("  [tools.{}]", tool);
    println!("  {:?}", new_tool);

    if Confirm::new()
        .with_prompt("Add this tool?")
        .default(true)
        .interact()?
    {
        let mut config = config.clone();
        config.tools.insert(tool.clone(), new_tool);
        config.save()?;
        println!("\n\u{2713} Added '{}' to kit.toml", tool);
        println!("Run `kit install {}` to install it", tool);
    } else {
        println!("Cancelled");
    }

    Ok(())
}
```

**Step 3: Run tests and verify**

Run:
```bash
cargo check
cargo test
```
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/commands/add.rs src/config.rs
git commit -m "feat: implement add command with interactive prompts"
```

---

## Phase 7: Final Polish

### Task 13: Add Error Handling and Polish

**Files:**
- Modify: `src/main.rs`

**Step 1: Add better error display**

Replace `src/main.rs`:
```rust
mod commands;
mod config;
mod scanner;
mod shell;
mod sources;

use clap::{Parser, Subcommand};
use config::Config;

#[derive(Parser)]
#[command(name = "kit")]
#[command(version)]
#[command(about = "CLI Tool Manager - track, install, and manage your command-line tools")]
#[command(long_about = None)]
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

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        for cause in e.chain().skip(1) {
            eprintln!("  Caused by: {}", cause);
        }
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
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
```

**Step 2: Build release binary**

Run:
```bash
cargo build --release
./target/release/kit --version
./target/release/kit --help
```
Expected: Shows version and help

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: add error handling and CLI polish"
```

---

### Task 14: Create Sample Config

**Files:**
- Create: `examples/kit.toml`

**Step 1: Create example config**

Create `examples/kit.toml`:
```toml
# Kit configuration example
# Copy to ~/.config/kit/kit.toml

[config]
bin_dir = "~/.kit/bin"
shell_rc = "~/.zshrc"

# Homebrew tools
[tools.ripgrep]
source = "brew"
aliases = ["rg"]

[tools.bat]
source = "brew"
aliases = ["cat"]

[tools.fd]
source = "brew"

[tools.eza]
source = "brew"
aliases = ["ls", "ll"]

# Mise-managed tools
[tools.python]
source = "mise"
version = "3.12"

[tools.node]
source = "mise"
version = "20"

# Curl install scripts
[tools.starship]
source = "curl"
install_url = "https://starship.rs/install.sh"
binary = "starship"

[tools.rustup]
source = "curl"
install_url = "https://sh.rustup.rs"
binary = "rustup"
```

**Step 2: Commit**

```bash
git add examples/kit.toml
git commit -m "docs: add example kit.toml configuration"
```

---

### Task 15: Final Integration Test

**Step 1: Run all tests**

Run:
```bash
cargo test
```
Expected: All tests pass

**Step 2: Test CLI commands**

Run:
```bash
./target/release/kit scan
./target/release/kit list
./target/release/kit export
```
Expected: All commands work

**Step 3: Create final commit**

```bash
git add -A
git commit -m "chore: kit v0.1.0 complete" --allow-empty
```

---

## Summary

**Total Tasks:** 15
**Estimated Time:** 1-2 hours of focused work

**Key Files Created:**
- `src/main.rs` - CLI entry point
- `src/config.rs` - TOML config parsing
- `src/scanner.rs` - PATH scanning and detection
- `src/shell.rs` - Alias generation and zshrc injection
- `src/sources/` - brew, mise, curl installers
- `src/commands/` - scan, list, install, aliases, export, add

**To Run:**
```bash
cargo build --release
./target/release/kit --help
```
