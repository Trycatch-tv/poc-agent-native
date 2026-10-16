use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "orchestrator", about = "Agent-Native Orchestrator CLI for POC 005")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingests narrative or work items and generates a consolidated System Specification
    Spec {
        #[arg(long)]
        narrative: Option<PathBuf>,

        #[arg(long)]
        work_items_dir: Option<PathBuf>,

        #[arg(short, long)]
        output: PathBuf,

        #[arg(long, default_value_t = false)]
        kaddo: bool,
    },
    /// Builds all components declared in the System Specification to WebAssembly Components
    Build {
        #[arg(short, long)]
        spec: PathBuf,

        #[arg(short, long)]
        out_dir: PathBuf,

        #[arg(long, default_value_t = false)]
        kaddo: bool,
    },
    /// Compares two System Specifications and outputs the diff
    Diff {
        #[arg(long)]
        before: PathBuf,

        #[arg(long)]
        after: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Spec {
            narrative,
            work_items_dir,
            output,
            kaddo,
        } => {
            if kaddo {
                println!("[orchestrator] Running in Kaddo Integrated Mode");
            } else {
                println!("[orchestrator] Running in Standalone Mode");
            }

            let spec = if let Some(n_path) = narrative {
                let content = std::fs::read_to_string(&n_path)?;
                let file_name = n_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                orchestrator::parse_narrative(&content, &file_name)
                    .map_err(|e| format!("Error parsing narrative: {e}"))?
            } else if let Some(w_dir) = work_items_dir {
                let mut items = Vec::new();
                for entry in std::fs::read_dir(w_dir)? {
                    let entry = entry?;
                    if entry.path().extension().is_some_and(|ext| ext == "md") {
                        let content = std::fs::read_to_string(entry.path())?;
                        let fname = entry.file_name().to_string_lossy().to_string();
                        items.push((fname, content));
                    }
                }
                let items_ref: Vec<(&str, &str)> = items.iter().map(|(f, c)| (f.as_str(), c.as_str())).collect();
                orchestrator::parse_work_items(&items_ref)
                    .map_err(|e| format!("Error parsing work items: {e}"))?
            } else {
                return Err("Either --narrative or --work-items-dir must be provided".into());
            };

            let json = serde_json::to_string_pretty(&spec)?;
            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&output, json)?;
            println!("[orchestrator] Consolidated System Specification saved to {}", output.display());
        }

        Commands::Build {
            spec,
            out_dir,
            kaddo,
        } => {
            if kaddo {
                println!("[orchestrator] Building components in Kaddo Integrated Mode");
            } else {
                println!("[orchestrator] Building components in Standalone Mode");
            }

            let spec_content = std::fs::read_to_string(&spec)?;
            let system_spec: orchestrator::SystemSpec = serde_json::from_str(&spec_content)?;
            let summary = orchestrator::build_all_components(&system_spec, &out_dir)
                .map_err(|e| format!("Build failed: {e}"))?;

            println!("[orchestrator] Build completed successfully:");
            for art in &summary.artifacts {
                println!("  - Component: {} -> {} ({} bytes, sha256={})", art.component_id, art.wasm_file, art.bytes_len, art.sha256);
            }
        }

        Commands::Diff {
            before,
            after,
            output,
        } => {
            let before_content = std::fs::read_to_string(&before)?;
            let after_content = std::fs::read_to_string(&after)?;
            let spec_before: orchestrator::SystemSpec = serde_json::from_str(&before_content)?;
            let spec_after: orchestrator::SystemSpec = serde_json::from_str(&after_content)?;
            let diff = orchestrator::compute_spec_diff(&spec_before, &spec_after);

            let diff_json = serde_json::to_string_pretty(&diff)?;
            if let Some(out_path) = output {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&out_path, &diff_json)?;
                println!("[orchestrator] Spec diff saved to {}", out_path.display());
            } else {
                println!("{diff_json}");
            }
        }
    }

    Ok(())
}
