use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tar::Archive;

const REPO: &str = "nickcramaro/kit";
const GITHUB_API: &str = "https://api.github.com/repos";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn get_latest_version() -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("kit")
        .build()?;

    let url = format!("{}/{}/releases/latest", GITHUB_API, REPO);
    let release: Release = client.get(&url).send()?.json()?;
    Ok(release.tag_name)
}

pub fn check_for_update() -> Result<Option<String>> {
    let current = env!("CARGO_PKG_VERSION");
    let latest = get_latest_version()?;

    let latest_clean = latest.trim_start_matches('v');
    if latest_clean != current {
        Ok(Some(latest))
    } else {
        Ok(None)
    }
}

fn get_platform() -> Result<(&'static str, &'static str)> {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        return Err(anyhow!("Unsupported OS"));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        return Err(anyhow!("Unsupported architecture"));
    };

    Ok((os, arch))
}

fn get_current_exe_dir() -> Result<PathBuf> {
    let exe = env::current_exe().context("Failed to get current executable path")?;
    exe.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow!("Failed to get executable directory"))
}

pub fn run() -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    println!(
        "{} {}",
        "Current version:".bold(),
        format!("v{}", current).cyan()
    );
    println!("{}", "Checking for updates...".dimmed());

    let client = reqwest::blocking::Client::builder()
        .user_agent("kit")
        .build()?;

    let url = format!("{}/{}/releases/latest", GITHUB_API, REPO);
    let release: Release = client
        .get(&url)
        .send()
        .context("Failed to fetch release info")?
        .json()
        .context("Failed to parse release info")?;

    let latest_clean = release.tag_name.trim_start_matches('v');
    if latest_clean == current {
        println!("{} Already up to date!", "\u{2713}".green());
        return Ok(());
    }

    println!(
        "{} {}",
        "New version available:".yellow(),
        release.tag_name.green().bold()
    );

    let (os, arch) = get_platform()?;
    let binary_name = format!("kit-{}-{}.tar.gz", os, arch);

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == binary_name)
        .ok_or_else(|| anyhow!("No binary available for {}-{}", os, arch))?;

    println!("{} {}...", "Downloading".cyan(), binary_name.dimmed());

    let response = client
        .get(&asset.browser_download_url)
        .send()
        .context("Failed to download binary")?;

    let bytes = response.bytes().context("Failed to read download")?;

    let install_dir = get_current_exe_dir()?;
    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let tar_path = temp_dir.path().join("kit.tar.gz");

    {
        let mut file = File::create(&tar_path).context("Failed to create temp file")?;
        file.write_all(&bytes).context("Failed to write download")?;
    }

    let tar_gz = File::open(&tar_path).context("Failed to open downloaded file")?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    let kit_binary = temp_dir.path().join("kit");
    archive
        .unpack(temp_dir.path())
        .context("Failed to extract archive")?;

    if !kit_binary.exists() {
        return Err(anyhow!("Binary not found in archive"));
    }

    let dest = install_dir.join("kit");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&kit_binary, perms)?;
    }

    fs::rename(&kit_binary, &dest).or_else(|_| {
        fs::copy(&kit_binary, &dest)?;
        fs::remove_file(&kit_binary)?;
        Ok::<_, std::io::Error>(())
    })?;

    println!(
        "{} Updated to {}!",
        "\u{2713}".green(),
        release.tag_name.green().bold()
    );
    Ok(())
}
