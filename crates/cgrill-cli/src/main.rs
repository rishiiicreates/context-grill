mod mcp;

use cgrill_core::{
    analyze_prompt, generate_questions, scan_environment, verify_patch, Contract,
};
use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cgrill")]
#[command(about = "Socratic ambiguity scanner, runtime inspector, and line-drift shield for AI coding agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan a prompt for ambiguity and generate Socratic questions
    Scan {
        /// The prompt or task description to analyze
        prompt: String,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Inspect local runtime environment (ports, daemons, git diff, stale build cache)
    Env {
        /// Directory to inspect (defaults to current directory)
        #[arg(short, long)]
        dir: Option<PathBuf>,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Manage Ground Truth Contract (.cgrill/contract.json)
    Contract {
        #[command(subcommand)]
        action: ContractAction,
    },

    /// Verify a patch before touching disk (checks line drift and bracket integrity)
    PatchGuard {
        /// Path to the target file
        #[arg(short, long)]
        file: PathBuf,

        /// Starting line number (1-indexed)
        #[arg(short, long)]
        start: usize,

        /// Ending line number (1-indexed)
        #[arg(short, long)]
        end: usize,

        /// Exact target content to replace
        #[arg(short, long)]
        target: String,

        /// Replacement content
        #[arg(short, long)]
        replacement: String,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Full pre-flight check (env health + contract status)
    Check,

    /// Run as a native stdio JSON-RPC Model Context Protocol (MCP) server
    Mcp,
}

#[derive(Subcommand)]
enum ContractAction {
    /// Initialize a new contract
    Init {
        /// The primary goal or task statement
        #[arg(short, long)]
        goal: String,

        /// Diff appetite (e.g. "surgical (<20 lines)", "targeted (<100 lines)")
        #[arg(short, long, default_value = "surgical (<20 lines)")]
        diff: String,

        /// Comma-separated allowed file paths
        #[arg(short, long)]
        allowed: Option<String>,

        /// Comma-separated forbidden file paths
        #[arg(short, long)]
        forbidden: Option<String>,

        /// Comma-separated acceptance criteria commands
        #[arg(short, long)]
        criteria: Option<String>,
    },

    /// Lock the contract to prevent scope creep
    Lock,

    /// Output the compact memory pin for long-loop context maintenance
    Pin,

    /// Check if modifying a specific file is permitted by the contract
    CheckWrite {
        /// Relative path to the file
        path: String,
    },

    /// Display current contract status
    Status,
}

fn main() {
    let cli = Cli::parse();
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    match cli.command {
        Commands::Scan { prompt, json } => {
            let report = analyze_prompt(&prompt);
            let questions = if report.requires_interview {
                generate_questions(&report, &prompt)
            } else {
                Vec::new()
            };

            if json {
                let out = serde_json::json!({
                    "report": report,
                    "questions": questions,
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap());
            } else {
                println!("\n=== CGRILL AMBIGUITY REPORT ===");
                println!("Prompt: \"{}\"", prompt);
                println!("Ambiguity Score: {}/100", report.score);
                println!("Summary: {}", report.summary);

                if !report.vague_terms_found.is_empty() {
                    println!("\nVague terms detected: {}", report.vague_terms_found.join(", "));
                }

                if report.requires_interview {
                    println!("\n[ACTION REQUIRED] Agent must cross-question user with these questions before writing code:\n");
                    for (idx, q) in questions.iter().enumerate() {
                        println!("{}. [{}] {}", idx + 1, q.title, q.question);
                        println!("   Context: {}", q.context_hint);
                        println!("   Suggested options:");
                        for opt in &q.suggested_options {
                            println!("     • {}", opt);
                        }
                        println!();
                    }
                } else {
                    println!("\n[PASS] Task is specific enough. You may proceed directly with contract initialization.");
                }
            }
        }

        Commands::Env { dir, json } => {
            let target_dir = dir.unwrap_or(current_dir);
            let health = scan_environment(&target_dir);

            if json {
                println!("{}", serde_json::to_string_pretty(&health).unwrap());
            } else {
                println!("\n=== CGRILL RUNTIME REALITY CHECK ===");
                println!("Target Directory: {}", health.scanned_dir.display());
                println!("Git Repo: {}", if health.git_status.is_git_repo { format!("Yes (branch: {})", health.git_status.branch) } else { "No".to_string() });
                println!("Working Tree Dirty: {}", if health.git_status.is_dirty { format!("YES ({} modified, {} untracked)", health.git_status.modified_count, health.git_status.untracked_count) } else { "No (Clean)".to_string() });

                if !health.active_ports.is_empty() {
                    println!("\nActive Listening Ports:");
                    for p in &health.active_ports {
                        let pid_str = p.pid.map(|pid| pid.to_string()).unwrap_or_else(|| "unknown".to_string());
                        println!("  • Port {}: {} (PID: {})", p.port, p.process_name, pid_str);
                    }
                } else {
                    println!("\nActive Listening Ports: None detected on local standard dev ports.");
                }

                if !health.stale_caches.is_empty() {
                    println!("\nStale Build Caches:");
                    for c in &health.stale_caches {
                        println!("  • {}: {}", c.cache_path.display(), c.reason);
                    }
                }

                if !health.warnings.is_empty() {
                    println!("\nWarnings:");
                    for w in &health.warnings {
                        println!("  ! {}", w);
                    }
                } else {
                    println!("\nEnvironment is clean and ready.");
                }
            }
        }

        Commands::Contract { action } => match action {
            ContractAction::Init {
                goal,
                diff,
                allowed,
                forbidden,
                criteria,
            } => {
                let mut contract = Contract::new(&goal, &diff);
                if let Some(a) = allowed {
                    contract.allowed_write_scope = a.split(',').map(|s| s.trim().to_string()).collect();
                }
                if let Some(f) = forbidden {
                    contract.forbidden_scope = f.split(',').map(|s| s.trim().to_string()).collect();
                }
                if let Some(c) = criteria {
                    contract.acceptance_criteria = c.split(',').map(|s| s.trim().to_string()).collect();
                }

                match contract.save(&current_dir) {
                    Ok(path) => {
                        println!("Contract initialized and saved to {}", path.display());
                        println!("\n{}", contract.generate_memory_pin());
                    }
                    Err(e) => eprintln!("Failed to save contract: {}", e),
                }
            }

            ContractAction::Lock => match Contract::load(&current_dir) {
                Ok(mut contract) => {
                    contract.lock();
                    if let Err(e) = contract.save(&current_dir) {
                        eprintln!("Failed to lock contract: {}", e);
                    } else {
                        println!("Contract locked successfully.");
                        println!("\n{}", contract.generate_memory_pin());
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            },

            ContractAction::Pin => match Contract::load(&current_dir) {
                Ok(contract) => println!("{}", contract.generate_memory_pin()),
                Err(e) => eprintln!("Error: {}", e),
            },

            ContractAction::CheckWrite { path } => match Contract::load(&current_dir) {
                Ok(contract) => {
                    let (allowed, err) = contract.is_write_allowed(&path);
                    if allowed {
                        println!("PASS: Write to '{}' is allowed by contract.", path);
                    } else {
                        eprintln!("DENIED: {}", err.unwrap_or_default());
                        std::process::exit(1);
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            },

            ContractAction::Status => match Contract::load(&current_dir) {
                Ok(contract) => {
                    println!("{}", serde_json::to_string_pretty(&contract).unwrap());
                }
                Err(e) => eprintln!("Error: {}", e),
            },
        },

        Commands::PatchGuard {
            file,
            start,
            end,
            target,
            replacement,
            json,
        } => {
            let res = verify_patch(&file, start, end, &target, &replacement);
            if json {
                println!("{}", serde_json::to_string_pretty(&res).unwrap());
            } else if res.valid {
                println!("PASS: Patch is valid, line anchors verified, and delimiters balanced.");
            } else {
                eprintln!("FAIL: Patch verification failed.");
                if res.drift_detected {
                    eprintln!(
                        "Line drift detected! Target was found at lines [{:?}-{:?}].",
                        res.suggested_start_line, res.suggested_end_line
                    );
                }
                if !res.delimiter_balanced {
                    eprintln!(
                        "Delimiter mismatch: {}",
                        res.delimiter_mismatch_detail.unwrap_or_default()
                    );
                }
                if let Some(err) = res.error_message {
                    eprintln!("Detail: {}", err);
                }
                std::process::exit(1);
            }
        }

        Commands::Check => {
            println!("=== PRE-FLIGHT SYSTEM & CONTRACT CHECK ===");
            let health = scan_environment(&current_dir);
            println!("Working Tree Dirty: {}", health.git_status.is_dirty);
            println!("Active Ports: {}", health.active_ports.len());

            match Contract::load(&current_dir) {
                Ok(contract) => {
                    println!("Active Contract: {} (Status: {})", contract.task_id, contract.status);
                    println!("\n{}", contract.generate_memory_pin());
                }
                Err(_) => {
                    println!("No active contract found in this directory. Run `cgrill contract init` to set up.");
                }
            }
        }

        Commands::Mcp => {
            if let Err(e) = mcp::run_mcp_server() {
                eprintln!("MCP server error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
