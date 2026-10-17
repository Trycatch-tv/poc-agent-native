use clap::{Parser, Subcommand};
use irtool::{
    compile_ir_to_wasm, print_wasm_to_wat, sha256_hex, validate_wasm_bytes, verify_ir, Diagnostic,
};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "irtool",
    version,
    about = "Deterministic Semantic IR v0.1 verifier and direct WASM compiler"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate IR structure and semantics, emitting diagnostics JSON
    Verify {
        #[arg(long)]
        schema: Option<PathBuf>,
        #[arg(long)]
        ir: PathBuf,
        #[arg(long)]
        diagnostics: PathBuf,
    },
    /// Compile verified Semantic IR v0.1 directly into a deterministic .wasm binary
    Compile {
        #[arg(long)]
        ir: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        wat: Option<PathBuf>,
    },
    /// Validate a .wasm binary using wasmparser (equivalent to wasm-tools validate)
    Validate { wasm: PathBuf },
    /// Print a .wasm binary as WAT text using wasmprinter (equivalent to wasm-tools print)
    Print {
        wasm: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Compute SHA-256 hash of a file
    Sha256 { file: PathBuf },
}

fn ensure_parent_dir(path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Verify {
            schema,
            ir,
            diagnostics,
        } => {
            if let Some(schema_path) = schema {
                if let Err(e) = fs::read_to_string(&schema_path) {
                    eprintln!(
                        "Failed to read schema file '{}': {e}",
                        schema_path.display()
                    );
                    return ExitCode::from(2);
                }
            }

            let ir_raw = match fs::read_to_string(&ir) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to read IR file '{}': {e}", ir.display());
                    return ExitCode::from(2);
                }
            };

            let diags = match serde_json::from_str::<Value>(&ir_raw) {
                Ok(val) => verify_ir(&val),
                Err(e) => vec![Diagnostic {
                    code: "E_JSON_PARSE".to_string(),
                    function: None,
                    path: "$".to_string(),
                    message: format!("Invalid JSON in IR file: {e}"),
                    expected: Some("valid JSON".to_string()),
                    found: None,
                }],
            };

            if let Err(e) = ensure_parent_dir(&diagnostics) {
                eprintln!(
                    "Failed to create parent directory for '{}': {e}",
                    diagnostics.display()
                );
                return ExitCode::from(2);
            }

            let diag_json = serde_json::to_string_pretty(&diags).unwrap();
            if let Err(e) = fs::write(&diagnostics, format!("{diag_json}\n")) {
                eprintln!(
                    "Failed to write diagnostics to '{}': {e}",
                    diagnostics.display()
                );
                return ExitCode::from(2);
            }

            if diags.is_empty() {
                println!(
                    "IR verification passed: 0 diagnostics written to {}",
                    diagnostics.display()
                );
                ExitCode::SUCCESS
            } else {
                eprintln!(
                    "IR verification failed with {} diagnostic(s) written to {}",
                    diags.len(),
                    diagnostics.display()
                );
                ExitCode::from(1)
            }
        }
        Commands::Compile { ir, output, wat } => {
            let ir_raw = match fs::read_to_string(&ir) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to read IR file '{}': {e}", ir.display());
                    return ExitCode::from(2);
                }
            };

            let ir_val: Value = match serde_json::from_str(&ir_raw) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Invalid JSON in IR file '{}': {e}", ir.display());
                    return ExitCode::from(1);
                }
            };

            let wasm_bytes = match compile_ir_to_wasm(&ir_val) {
                Ok(bytes) => bytes,
                Err(diags) => {
                    eprintln!(
                        "Refusing to compile invalid IR ({} diagnostic(s)):",
                        diags.len()
                    );
                    eprintln!("{}", serde_json::to_string_pretty(&diags).unwrap());
                    return ExitCode::from(1);
                }
            };

            if let Err(e) = validate_wasm_bytes(&wasm_bytes) {
                eprintln!("Internal compiler error: emitted WASM failed validation: {e}");
                return ExitCode::from(2);
            }

            if let Err(e) = ensure_parent_dir(&output) {
                eprintln!("Failed to create output directory: {e}");
                return ExitCode::from(2);
            }

            if let Err(e) = fs::write(&output, &wasm_bytes) {
                eprintln!("Failed to write WASM binary '{}': {e}", output.display());
                return ExitCode::from(2);
            }

            if let Some(wat_path) = wat {
                match print_wasm_to_wat(&wasm_bytes) {
                    Ok(wat_str) => {
                        let _ = ensure_parent_dir(&wat_path);
                        if let Err(e) = fs::write(&wat_path, wat_str) {
                            eprintln!("Failed to write WAT file '{}': {e}", wat_path.display());
                            return ExitCode::from(2);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to format WAT: {e}");
                        return ExitCode::from(2);
                    }
                }
            }

            let digest = sha256_hex(&wasm_bytes);
            println!(
                "Compiled {} ({} bytes, sha256={digest})",
                output.display(),
                wasm_bytes.len()
            );
            ExitCode::SUCCESS
        }
        Commands::Validate { wasm } => {
            let bytes = match fs::read(&wasm) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed to read WASM file '{}': {e}", wasm.display());
                    return ExitCode::from(2);
                }
            };
            match validate_wasm_bytes(&bytes) {
                Ok(()) => {
                    println!(
                        "Valid WASM module: {} ({} bytes)",
                        wasm.display(),
                        bytes.len()
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Invalid WASM module '{}': {e}", wasm.display());
                    ExitCode::from(1)
                }
            }
        }
        Commands::Print { wasm, output } => {
            let bytes = match fs::read(&wasm) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed to read WASM file '{}': {e}", wasm.display());
                    return ExitCode::from(2);
                }
            };
            match print_wasm_to_wat(&bytes) {
                Ok(wat_str) => {
                    if let Some(out_path) = output {
                        if let Err(e) = ensure_parent_dir(&out_path) {
                            eprintln!("Failed to create parent dir: {e}");
                            return ExitCode::from(2);
                        }
                        if let Err(e) = fs::write(&out_path, &wat_str) {
                            eprintln!("Failed to write WAT '{}': {e}", out_path.display());
                            return ExitCode::from(2);
                        }
                    } else {
                        print!("{wat_str}");
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Failed to print WAT for '{}': {e}", wasm.display());
                    ExitCode::from(1)
                }
            }
        }
        Commands::Sha256 { file } => {
            let bytes = match fs::read(&file) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed to read '{}': {e}", file.display());
                    return ExitCode::from(2);
                }
            };
            println!("{}  {}", sha256_hex(&bytes), file.display());
            ExitCode::SUCCESS
        }
    }
}
