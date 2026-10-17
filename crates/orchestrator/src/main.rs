use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "orchestrator",
    about = "Agent-Native Orchestrator CLI for POC 005"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes an Agent-Native project configuration in a directory
    Init {
        #[arg(long, default_value = ".")]
        project_root: PathBuf,
    },
    /// Ingests narrative or work items and generates a consolidated System Specification
    Spec {
        #[arg(long)]
        narrative: Option<PathBuf>,

        #[arg(long)]
        work_items_dir: Option<PathBuf>,

        #[arg(long)]
        base_spec: Option<PathBuf>,

        #[arg(short, long)]
        output: PathBuf,

        #[arg(long)]
        project_root: Option<PathBuf>,

        #[arg(long, default_value_t = false)]
        kaddo: bool,
    },
    /// Builds all components declared in the System Specification to WebAssembly Components
    Build {
        #[arg(short, long)]
        spec: PathBuf,

        #[arg(short, long)]
        out_dir: PathBuf,

        #[arg(long)]
        project_root: Option<PathBuf>,

        #[arg(long, default_value_t = false)]
        kaddo: bool,

        #[arg(long, default_value = "deterministic")]
        agent: String,
    },
    /// Compares two System Specifications and outputs the diff
    Diff {
        #[arg(long)]
        before: PathBuf,

        #[arg(long)]
        after: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long)]
        project_root: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { project_root } => {
            std::fs::create_dir_all(&project_root)?;
            let config_path = project_root.join("agent-native.toml");
            let default_config = r#"# Agent-Native Project Configuration (v0.1.0)
[project]
name = "agent-native-app"
version = "0.1.0"
spec_version = "1.0.0"

[orchestrator]
narrative_path = "spec/narrative.md"
work_items_dir = "spec/work-items"
output_dir = "dist"
target = "wasm32-wasip1"

[kaddo]
enabled = false
"#;
            std::fs::write(&config_path, default_config)?;
            println!(
                "[orchestrator] Initialized Agent-Native project at {}",
                config_path.display()
            );
        }

        Commands::Spec {
            narrative,
            work_items_dir,
            base_spec,
            output,
            project_root,
            kaddo,
        } => {
            let root = project_root.unwrap_or_else(|| PathBuf::from("."));
            if kaddo {
                println!(
                    "[orchestrator] Running in Kaddo Integrated Mode (root: {})",
                    root.display()
                );
            } else {
                println!(
                    "[orchestrator] Running in Standalone Mode (root: {})",
                    root.display()
                );
            }

            let base_spec_obj = if let Some(ref b_path) = base_spec {
                let actual_path = if b_path.is_relative() {
                    root.join(b_path)
                } else {
                    b_path.clone()
                };
                let content = std::fs::read_to_string(actual_path)?;
                let parsed: orchestrator::SystemSpec = serde_json::from_str(&content)?;
                Some(parsed)
            } else {
                None
            };

            let spec = if let Some(n_path) = narrative {
                let actual_path = if n_path.is_relative() {
                    root.join(&n_path)
                } else {
                    n_path.clone()
                };
                let content = std::fs::read_to_string(&actual_path)?;
                let file_name = actual_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                orchestrator::parse_narrative(&content, &file_name)
                    .map_err(|e| format!("Error parsing narrative: {e}"))?
            } else if let Some(w_dir) = work_items_dir {
                let actual_dir = if w_dir.is_relative() {
                    root.join(&w_dir)
                } else {
                    w_dir.clone()
                };
                let mut items = Vec::new();
                for entry in std::fs::read_dir(actual_dir)? {
                    let entry = entry?;
                    if entry.path().extension().is_some_and(|ext| ext == "md") {
                        let content = std::fs::read_to_string(entry.path())?;
                        let fname = entry.file_name().to_string_lossy().to_string();
                        items.push((fname, content));
                    }
                }
                let items_ref: Vec<(&str, &str)> = items
                    .iter()
                    .map(|(f, c)| (f.as_str(), c.as_str()))
                    .collect();
                orchestrator::parse_work_items_consolidated(base_spec_obj.as_ref(), &items_ref)
                    .map_err(|e| format!("Error parsing work items: {e}"))?
            } else {
                return Err("Either --narrative or --work-items-dir must be provided".into());
            };

            let actual_out = if output.is_relative() {
                root.join(&output)
            } else {
                output.clone()
            };
            let json = serde_json::to_string_pretty(&spec)?;
            if let Some(parent) = actual_out.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&actual_out, json)?;
            println!(
                "[orchestrator] Consolidated System Specification saved to {}",
                actual_out.display()
            );
        }

        Commands::Build {
            spec,
            out_dir,
            project_root,
            kaddo,
            agent,
        } => {
            let root = project_root.unwrap_or_else(|| PathBuf::from("."));
            if kaddo {
                println!(
                    "[orchestrator] Building components in Kaddo Integrated Mode (root: {})",
                    root.display()
                );
            } else {
                println!(
                    "[orchestrator] Building components in Standalone Mode (root: {})",
                    root.display()
                );
            }

            let actual_spec_path = if spec.is_relative() {
                root.join(&spec)
            } else {
                spec.clone()
            };
            let actual_out_dir = if out_dir.is_relative() {
                root.join(&out_dir)
            } else {
                out_dir.clone()
            };

            let spec_content = std::fs::read_to_string(&actual_spec_path)?;
            let system_spec: orchestrator::SystemSpec = serde_json::from_str(&spec_content)?;

            let summary = match agent.as_str() {
                "deterministic" => {
                    println!("[orchestrator] Using SynthesizerAgent (deterministic mode)");
                    orchestrator::build_all_components(&system_spec, &actual_out_dir)
                }
                "llm" => {
                    let config = orchestrator::agent::LlmConfig::for_cli_build();
                    println!(
                        "[orchestrator] Using LlmImplementationAgent (strict LLM mode, provider={}, model={})",
                        config.provider, config.model
                    );
                    let llm_agent = orchestrator::agent::LlmImplementationAgent::strict(config);
                    orchestrator::build_all_components_with_agent(&llm_agent, &system_spec, &actual_out_dir)
                }
                other => {
                    return Err(format!("Unknown agent mode '{other}'. Supported values are 'deterministic' and 'llm'.").into());
                }
            }.map_err(|e| format!("Build failed: {e}"))?;

            println!("[orchestrator] Build completed successfully:");
            for art in &summary.artifacts {
                println!(
                    "  - Component: {} -> {} ({} bytes, sha256={})",
                    art.component_id, art.wasm_file, art.bytes_len, art.sha256
                );
            }
        }

        Commands::Diff {
            before,
            after,
            output,
            project_root,
        } => {
            let root = project_root.unwrap_or_else(|| PathBuf::from("."));
            let actual_before = if before.is_relative() {
                root.join(&before)
            } else {
                before.clone()
            };
            let actual_after = if after.is_relative() {
                root.join(&after)
            } else {
                after.clone()
            };

            let before_content = std::fs::read_to_string(&actual_before)?;
            let after_content = std::fs::read_to_string(&actual_after)?;
            let spec_before: orchestrator::SystemSpec = serde_json::from_str(&before_content)?;
            let spec_after: orchestrator::SystemSpec = serde_json::from_str(&after_content)?;
            let diff = orchestrator::compute_spec_diff(&spec_before, &spec_after);

            let diff_json = serde_json::to_string_pretty(&diff)?;
            if let Some(out_path) = output {
                let actual_out = if out_path.is_relative() {
                    root.join(&out_path)
                } else {
                    out_path.clone()
                };
                if let Some(parent) = actual_out.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&actual_out, &diff_json)?;
                println!("[orchestrator] Spec diff saved to {}", actual_out.display());
            } else {
                println!("{diff_json}");
            }
        }
    }

    Ok(())
}
