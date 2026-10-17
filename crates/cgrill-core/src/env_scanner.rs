use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub port: u16,
    pub pid: Option<u32>,
    pub process_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSummary {
    pub is_git_repo: bool,
    pub branch: String,
    pub is_dirty: bool,
    pub modified_count: usize,
    pub untracked_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaleCacheWarning {
    pub cache_path: PathBuf,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvHealth {
    pub scanned_dir: PathBuf,
    pub active_ports: Vec<PortInfo>,
    pub git_status: GitSummary,
    pub stale_caches: Vec<StaleCacheWarning>,
    pub env_file_present: bool,
    pub env_example_discrepancy: Option<String>,
    pub warnings: Vec<String>,
    pub is_clean: bool,
}

pub fn scan_environment(dir: &Path) -> EnvHealth {
    let mut warnings = Vec::new();
    let scanned_dir = dir.to_path_buf();

    // 1. Scan active listening ports via lsof
    let active_ports = scan_listening_ports();

    // 2. Scan Git Status
    let git_status = scan_git_status(dir);
    if git_status.is_dirty {
        warnings.push(format!(
            "Working directory has {} modified and {} untracked files. Do not wipe or stash without user consent.",
            git_status.modified_count, git_status.untracked_count
        ));
    }

    // 3. Scan Stale Caches
    let stale_caches = scan_stale_caches(dir);
    for warning in &stale_caches {
        warnings.push(format!(
            "Cache directory '{}' is stale: {}",
            warning.cache_path.display(),
            warning.reason
        ));
    }

    // 4. Check .env vs .env.example
    let (env_file_present, env_example_discrepancy) = check_env_files(dir);
    if let Some(ref discrepancy) = env_example_discrepancy {
        warnings.push(discrepancy.clone());
    }

    let is_clean = warnings.is_empty();

    EnvHealth {
        scanned_dir,
        active_ports,
        git_status,
        stale_caches,
        env_file_present,
        env_example_discrepancy,
        warnings,
        is_clean,
    }
}

fn scan_listening_ports() -> Vec<PortInfo> {
    let mut ports = Vec::new();
    // Run lsof on macOS/Linux
    let output = Command::new("lsof")
        .args(["-iTCP", "-sTCP:LISTEN", "-n", "-P"])
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                let proc_name = parts[0].to_string();
                let pid = parts[1].parse::<u32>().ok();
                let address = parts[8];
                if let Some(idx) = address.rfind(':') {
                    if let Ok(port) = address[idx + 1..].parse::<u16>() {
                        // Avoid duplicates
                        if !ports.iter().any(|p: &PortInfo| p.port == port) {
                            ports.push(PortInfo {
                                port,
                                pid,
                                process_name: proc_name,
                            });
                        }
                    }
                }
            }
        }
    }
    ports
}

fn scan_git_status(dir: &Path) -> GitSummary {
    let branch_output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["branch", "--show-current"])
        .output();

    let branch = match branch_output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    };

    let status_output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["status", "--porcelain"])
        .output();

    match status_output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut modified = 0;
            let mut untracked = 0;
            for line in stdout.lines() {
                if line.starts_with("??") {
                    untracked += 1;
                } else if !line.trim().is_empty() {
                    modified += 1;
                }
            }
            GitSummary {
                is_git_repo: true,
                branch,
                is_dirty: modified > 0 || untracked > 0,
                modified_count: modified,
                untracked_count: untracked,
            }
        }
        _ => GitSummary {
            is_git_repo: false,
            branch: "none".to_string(),
            is_dirty: false,
            modified_count: 0,
            untracked_count: 0,
        },
    }
}

fn scan_stale_caches(dir: &Path) -> Vec<StaleCacheWarning> {
    let mut warnings = Vec::new();
    let cache_dirs = ["dist", ".next", ".turbo", "node_modules/.cache"];

    // Find latest source mtime
    let latest_src_mtime = find_latest_src_mtime(dir);

    for sub in &cache_dirs {
        let cache_path = dir.join(sub);
        if cache_path.exists() {
            if let Ok(metadata) = fs::metadata(&cache_path) {
                if let Ok(cache_mtime) = metadata.modified() {
                    if let Some(src_mtime) = latest_src_mtime {
                        if cache_mtime < src_mtime {
                            warnings.push(StaleCacheWarning {
                                cache_path: cache_path.clone(),
                                reason: "Source files were modified after this build cache was written. Rebuild recommended.".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    warnings
}

fn find_latest_src_mtime(dir: &Path) -> Option<SystemTime> {
    let src_dir = dir.join("src");
    let target = if src_dir.exists() { &src_dir } else { dir };
    let mut latest = None;

    if let Ok(entries) = fs::read_dir(target) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if let Ok(mtime) = meta.modified() {
                    latest = match latest {
                        None => Some(mtime),
                        Some(prev) => Some(if mtime > prev { mtime } else { prev }),
                    };
                }
            }
        }
    }
    latest
}

fn check_env_files(dir: &Path) -> (bool, Option<String>) {
    let env_file = dir.join(".env");
    let env_example = dir.join(".env.example");

    let env_present = env_file.exists();
    let example_present = env_example.exists();

    if example_present && !env_present {
        return (
            false,
            Some(".env.example exists but .env is missing. Application may panic on missing environment variables.".to_string()),
        );
    }

    (env_present, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_dir() {
        let current = std::env::current_dir().unwrap();
        let health = scan_environment(&current);
        assert!(!health.scanned_dir.as_os_str().is_empty());
    }
}
