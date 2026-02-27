use std::fs;
use std::path::PathBuf;

use flate2::read::GzDecoder;
use serde::Deserialize;

const REPO: &str = "hwhang0917/sqlfmt";
const BINARY_NAME: &str = "sqlfmt";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

fn parse_semver(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_semver(latest), parse_semver(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

pub fn run() {
    let current_version = env!("CARGO_PKG_VERSION");
    let target = env!("TARGET");

    println!("Current version: v{current_version}");
    println!("Checking for updates...");

    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let release: Release = match ureq::get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "sqlfmt-updater")
        .call()
    {
        Ok(mut response) => match response.body_mut().read_json() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("sqlfmt: failed to parse release info: {e}");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("sqlfmt: failed to check for updates: {e}");
            std::process::exit(1);
        }
    };

    let tag = &release.tag_name;
    let latest_version = tag.strip_prefix('v').unwrap_or(tag);

    if !is_newer(latest_version, current_version) {
        println!("Already up to date (v{current_version}).");
        return;
    }

    println!("Updating v{current_version} -> {tag}...");

    let asset_url = format!(
        "https://github.com/{REPO}/releases/download/{tag}/{BINARY_NAME}-{tag}-{target}.tar.gz"
    );

    let tarball_bytes = match ureq::get(&asset_url)
        .header("User-Agent", "sqlfmt-updater")
        .call()
    {
        Ok(response) => match response.into_body().read_to_vec() {
            Ok(buf) => buf,
            Err(e) => {
                eprintln!("sqlfmt: failed to download update: {e}");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("sqlfmt: failed to download update: {e}");
            std::process::exit(1);
        }
    };

    let decoder = GzDecoder::new(tarball_bytes.as_slice());
    let mut archive = tar::Archive::new(decoder);

    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("sqlfmt: failed to locate current binary: {e}");
            std::process::exit(1);
        }
    };

    let tmp_path = current_exe.with_extension("tmp");

    let mut found = false;
    for entry in archive.entries().unwrap() {
        let mut entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("sqlfmt: failed to read archive entry: {e}");
                std::process::exit(1);
            }
        };

        let path = entry.path().unwrap().to_path_buf();
        if path.file_name().and_then(|n| n.to_str()) == Some(BINARY_NAME) {
            if let Err(e) = entry.unpack(&tmp_path) {
                eprintln!("sqlfmt: failed to extract binary: {e}");
                std::process::exit(1);
            }
            found = true;
            break;
        }
    }

    if !found {
        eprintln!("sqlfmt: binary not found in archive");
        std::process::exit(1);
    }

    set_executable(&tmp_path);

    if let Err(e) = fs::rename(&tmp_path, &current_exe) {
        let _ = fs::remove_file(&tmp_path);
        eprintln!("sqlfmt: failed to replace binary: {e}");
        std::process::exit(1);
    }

    println!("Updated to {tag} successfully.");
}

#[cfg(unix)]
fn set_executable(path: &PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
}

#[cfg(not(unix))]
fn set_executable(_path: &PathBuf) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_semver("0.2.0"), Some((0, 2, 0)));
        assert_eq!(parse_semver("invalid"), None);
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.3.0", "0.2.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(!is_newer("0.2.0", "0.2.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
    }
}
