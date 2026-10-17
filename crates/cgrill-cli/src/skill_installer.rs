use std::fs;
use std::path::{Path, PathBuf};

pub const CANONICAL_SKILL_MD: &str = include_str!("../../../SKILL.md");

pub struct PlatformTarget {
    pub name: &'static str,
    pub key: &'static str,
    pub global_path: Option<PathBuf>,
    pub local_path: PathBuf,
}

pub fn get_platforms(home: Option<&Path>) -> Vec<PlatformTarget> {
    vec![
        PlatformTarget {
            name: "Antigravity (AGY) & Universal Agent Skills",
            key: "agy",
            global_path: home.map(|h| h.join(".agents").join("skills").join("context-grill").join("SKILL.md")),
            local_path: PathBuf::from(".agents/skills/context-grill/SKILL.md"),
        },
        PlatformTarget {
            name: "Claude Code",
            key: "claude",
            global_path: home.map(|h| h.join(".claude").join("skills").join("context-grill").join("SKILL.md")),
            local_path: PathBuf::from(".claude/skills/context-grill/SKILL.md"),
        },
        PlatformTarget {
            name: "Cursor",
            key: "cursor",
            global_path: home.map(|h| h.join(".cursor").join("skills").join("context-grill").join("SKILL.md")),
            local_path: PathBuf::from(".cursor/skills/context-grill/SKILL.md"),
        },
        PlatformTarget {
            name: "Windsurf",
            key: "windsurf",
            global_path: home.map(|h| h.join(".codeium").join("windsurf").join("skills").join("context-grill").join("SKILL.md")),
            local_path: PathBuf::from(".windsurf/skills/context-grill/SKILL.md"),
        },
        PlatformTarget {
            name: "OpenCode",
            key: "opencode",
            global_path: home.map(|h| h.join(".opencode").join("skills").join("context-grill").join("SKILL.md")),
            local_path: PathBuf::from(".opencode/skills/context-grill/SKILL.md"),
        },
    ]
}

pub fn install_skill(target: &str, local: bool) -> Result<(), std::io::Error> {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    let platforms = get_platforms(home.as_deref());
    let mut installed_count = 0;

    println!("\n=== CONTEXT-GRILL SKILL INSTALLER ===");

    for p in platforms {
        if target != "all" && target != p.key {
            continue;
        }

        let target_file = if local {
            p.local_path
        } else if let Some(ref gp) = p.global_path {
            gp.clone()
        } else {
            p.local_path
        };

        if let Some(parent) = target_file.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&target_file, CANONICAL_SKILL_MD)?;
        println!("  ✓ {}: Installed to {}", p.name, target_file.display());
        installed_count += 1;
    }

    if installed_count > 0 {
        println!("\nSuccess! context-grill skill is now active across {} platform(s).", installed_count);
        println!("Your agents will automatically interrogate vague prompts, inspect environment state, and guard file edits.\n");
    } else {
        println!("\nNo matching platform found for '{}'. Choose from: all, agy, claude, cursor, windsurf, opencode.\n", target);
    }

    Ok(())
}

pub fn skill_status() {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    let platforms = get_platforms(home.as_deref());

    println!("\n=== CONTEXT-GRILL SKILL INSTALLATION STATUS ===");
    for p in platforms {
        let global_installed = p.global_path.as_ref().map(|path| path.exists()).unwrap_or(false);
        let local_installed = p.local_path.exists();

        let status_str = if global_installed && local_installed {
            "Installed (Global + Local)"
        } else if global_installed {
            "Installed (Global)"
        } else if local_installed {
            "Installed (Local Workspace)"
        } else {
            "Not Installed"
        };

        let icon = if global_installed || local_installed { "✓" } else { "○" };
        println!("  {} {}: {}", icon, p.name, status_str);
        if let Some(ref gp) = p.global_path {
            if global_installed {
                println!("     Path: {}", gp.display());
            }
        }
    }
    println!();
}

pub fn uninstall_skill(target: &str) -> Result<(), std::io::Error> {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    let platforms = get_platforms(home.as_deref());
    let mut removed = 0;

    println!("\n=== CONTEXT-GRILL SKILL UNINSTALLER ===");
    for p in platforms {
        if target != "all" && target != p.key {
            continue;
        }

        if let Some(ref gp) = p.global_path {
            if gp.exists() {
                fs::remove_file(gp)?;
                println!("  ✓ Removed global skill for {}: {}", p.name, gp.display());
                removed += 1;
            }
        }
        if p.local_path.exists() {
            fs::remove_file(&p.local_path)?;
            println!("  ✓ Removed local skill for {}: {}", p.name, p.local_path.display());
            removed += 1;
        }
    }

    if removed > 0 {
        println!("\nRemoved context-grill from {} location(s).\n", removed);
    } else {
        println!("\nNo active installations found for '{}'.\n", target);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_skill_not_empty() {
        assert!(!CANONICAL_SKILL_MD.is_empty());
        assert!(CANONICAL_SKILL_MD.contains("name: context-grill"));
    }

    #[test]
    fn test_platforms_list() {
        let platforms = get_platforms(None);
        assert_eq!(platforms.len(), 5);
        assert!(platforms.iter().any(|p| p.key == "agy"));
        assert!(platforms.iter().any(|p| p.key == "claude"));
    }
}
