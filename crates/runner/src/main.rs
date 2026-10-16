use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;
use wasmtime::{Engine, Instance, Module, Store, Val};

#[derive(Parser)]
#[command(name = "runner", version, about = "Deterministic Wasmtime test runner for Kaddo Agent-Native POC")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a YAML test suite against a compiled WebAssembly module
    Test {
        #[arg(long)]
        wasm: PathBuf,
        #[arg(long)]
        cases: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct TestCaseInput {
    id: String,
    #[serde(default)]
    requirement: Option<String>,
    function: String,
    args: Vec<YamlValue>,
    expected: YamlValue,
}

#[derive(Debug, Serialize, Deserialize)]
struct CaseResult {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    requirement: Option<String>,
    function: String,
    args: Vec<serde_json::Value>,
    expected: serde_json::Value,
    actual: Option<serde_json::Value>,
    passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SuiteReport {
    passed: usize,
    failed: usize,
    total: usize,
    execution_ms: u128,
    cases: Vec<CaseResult>,
}

fn yaml_val_to_i32(v: &YamlValue) -> Result<i32, String> {
    match v {
        YamlValue::Number(n) => n
            .as_i64()
            .and_then(|i| i32::try_from(i).ok())
            .ok_or_else(|| format!("Numeric value '{n:?}' is not a valid i32")),
        YamlValue::Bool(b) => Ok(if *b { 1 } else { 0 }),
        other => Err(format!("Unsupported argument value in YAML: {other:?}")),
    }
}

fn yaml_to_json_val(v: &YamlValue) -> serde_json::Value {
    match v {
        YamlValue::Null => serde_json::Value::Null,
        YamlValue::Bool(b) => serde_json::Value::Bool(*b),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::json!(i)
            } else if let Some(f) = n.as_f64() {
                serde_json::json!(f)
            } else {
                serde_json::Value::Null
            }
        }
        YamlValue::String(s) => serde_json::Value::String(s.clone()),
        YamlValue::Sequence(seq) => {
            serde_json::Value::Array(seq.iter().map(yaml_to_json_val).collect())
        }
        YamlValue::Mapping(map) => {
            let mut json_obj = serde_json::Map::new();
            for (k, val) in map {
                if let Some(key_str) = k.as_str() {
                    json_obj.insert(key_str.to_string(), yaml_to_json_val(val));
                }
            }
            serde_json::Value::Object(json_obj)
        }
        _ => serde_json::Value::Null,
    }
}

fn parse_cases(raw_yaml: &str) -> Result<Vec<TestCaseInput>, String> {
    let root: YamlValue = serde_yaml::from_str(raw_yaml).map_err(|e| e.to_string())?;
    if let Some(seq) = root.get("cases") {
        serde_yaml::from_value(seq.clone()).map_err(|e| e.to_string())
    } else if let Some(pub_seq) = root.get("acceptance").and_then(|a| a.get("public")) {
        serde_yaml::from_value(pub_seq.clone()).map_err(|e| e.to_string())
    } else if root.is_sequence() {
        serde_yaml::from_value(root).map_err(|e| e.to_string())
    } else {
        Err("YAML file must contain 'cases' or 'acceptance.public' list.".to_string())
    }
}

fn run_suite(wasm_bytes: &[u8], test_cases: &[TestCaseInput]) -> Result<SuiteReport, String> {
    let start = Instant::now();
    let engine = Engine::default();
    let module = Module::from_binary(&engine, wasm_bytes).map_err(|e| e.to_string())?;

    let mut passed_count = 0usize;
    let mut failed_count = 0usize;
    let mut results = Vec::with_capacity(test_cases.len());

    for tc in test_cases {
        let mut i32_args = Vec::with_capacity(tc.args.len());
        let mut json_args = Vec::with_capacity(tc.args.len());
        let mut conv_err = None;

        for a in &tc.args {
            json_args.push(yaml_to_json_val(a));
            match yaml_val_to_i32(a) {
                Ok(v) => i32_args.push(v),
                Err(e) => {
                    conv_err = Some(e);
                    break;
                }
            }
        }

        let expected_json = yaml_to_json_val(&tc.expected);

        if let Some(err_msg) = conv_err {
            failed_count += 1;
            results.push(CaseResult {
                id: tc.id.clone(),
                requirement: tc.requirement.clone(),
                function: tc.function.clone(),
                args: json_args,
                expected: expected_json,
                actual: None,
                passed: false,
                error: Some(err_msg),
            });
            continue;
        }

        let mut store = Store::new(&engine, ());
        let instance = match Instance::new(&mut store, &module, &[]) {
            Ok(inst) => inst,
            Err(e) => {
                failed_count += 1;
                results.push(CaseResult {
                    id: tc.id.clone(),
                    requirement: tc.requirement.clone(),
                    function: tc.function.clone(),
                    args: json_args,
                    expected: expected_json,
                    actual: None,
                    passed: false,
                    error: Some(format!("Failed to instantiate WASM module: {e}")),
                });
                continue;
            }
        };

        let Some(func) = instance.get_func(&mut store, &tc.function) else {
            failed_count += 1;
            results.push(CaseResult {
                id: tc.id.clone(),
                requirement: tc.requirement.clone(),
                function: tc.function.clone(),
                args: json_args,
                expected: expected_json,
                actual: None,
                passed: false,
                error: Some(format!("Exported function '{}' not found", tc.function)),
            });
            continue;
        };

        let wasm_args: Vec<Val> = i32_args.iter().map(|v| Val::I32(*v)).collect();
        let mut wasm_results = [Val::I32(0)];

        match func.call(&mut store, &wasm_args, &mut wasm_results) {
            Ok(()) => {
                let ret_i32 = wasm_results[0].i32().unwrap_or(0);

                let (actual_json, is_pass, err) = match &tc.expected {
                    YamlValue::Mapping(map) => {
                        // Expected a record struct in memory pointed to by ret_i32
                        let ptr = ret_i32;
                        if let Some(memory) = instance.get_memory(&mut store, "memory") {
                            let mem_data = memory.data(&store);
                            let num_fields = map.len();
                            let start_offset = ptr as usize;
                            let end_offset = start_offset + num_fields * 4;

                            if ptr < 0 || end_offset > mem_data.len() {
                                (
                                    None,
                                    false,
                                    Some(format!(
                                        "Record pointer {ptr} is out of bounds for memory size {}",
                                        mem_data.len()
                                    )),
                                )
                            } else {
                                let mut actual_map = serde_json::Map::new();
                                for (idx, (k, _)) in map.iter().enumerate() {
                                    let field_name = k.as_str().unwrap_or("");
                                    let offset = start_offset + idx * 4;
                                    let bytes: [u8; 4] = mem_data[offset..offset + 4]
                                        .try_into()
                                        .unwrap();
                                    let field_val = i32::from_le_bytes(bytes);
                                    actual_map.insert(
                                        field_name.to_string(),
                                        serde_json::json!(field_val),
                                    );
                                }
                                let act_val = serde_json::Value::Object(actual_map);
                                let pass = act_val == expected_json;
                                (Some(act_val), pass, None)
                            }
                        } else {
                            (
                                None,
                                false,
                                Some("WASM module returned a record but does not export 'memory'"
                                    .to_string()),
                            )
                        }
                    }
                    _ => {
                        // Scalar comparison
                        let act_val = serde_json::json!(ret_i32);
                        let pass = act_val == expected_json;
                        (Some(act_val), pass, None)
                    }
                };

                if is_pass {
                    passed_count += 1;
                } else {
                    failed_count += 1;
                }

                results.push(CaseResult {
                    id: tc.id.clone(),
                    requirement: tc.requirement.clone(),
                    function: tc.function.clone(),
                    args: json_args,
                    expected: expected_json,
                    actual: actual_json,
                    passed: is_pass,
                    error: err,
                });
            }
            Err(e) => {
                failed_count += 1;
                results.push(CaseResult {
                    id: tc.id.clone(),
                    requirement: tc.requirement.clone(),
                    function: tc.function.clone(),
                    args: json_args,
                    expected: expected_json,
                    actual: None,
                    passed: false,
                    error: Some(format!("WASM trap/execution error: {e}")),
                });
            }
        }
    }

    Ok(SuiteReport {
        passed: passed_count,
        failed: failed_count,
        total: test_cases.len(),
        execution_ms: start.elapsed().as_millis(),
        cases: results,
    })
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test {
            wasm,
            cases,
            output,
        } => {
            let wasm_bytes = match fs::read(&wasm) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed to read WASM file '{}': {e}", wasm.display());
                    return ExitCode::from(2);
                }
            };

            let raw_yaml = match fs::read_to_string(&cases) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to read test cases '{}': {e}", cases.display());
                    return ExitCode::from(2);
                }
            };

            let parsed_cases = match parse_cases(&raw_yaml) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to parse test cases '{}': {e}", cases.display());
                    return ExitCode::from(2);
                }
            };

            let report = match run_suite(&wasm_bytes, &parsed_cases) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to execute test suite: {e}");
                    return ExitCode::from(2);
                }
            };

            if let Some(parent) = output.parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = fs::create_dir_all(parent);
                }
            }

            let json_out = serde_json::to_string_pretty(&report).unwrap();
            if let Err(e) = fs::write(&output, format!("{json_out}\n")) {
                eprintln!("Failed to write test report '{}': {e}", output.display());
                return ExitCode::from(2);
            }

            println!(
                "Test suite finished: {}/{} passed ({} failed) in {} ms -> {}",
                report.passed,
                report.total,
                report.failed,
                report.execution_ms,
                output.display()
            );

            if report.failed == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irtool::compile_ir_to_wasm;
    use serde_json::json;

    #[test]
    fn executes_all_ir_v0_1_operators_in_wasmtime() {
        let ir = json!({
            "ir_version": "0.1",
            "spec_version": 1,
            "functions": [
                {
                    "name": "compute_rule",
                    "export": true,
                    "params": [
                        { "name": "x", "type": "i32" },
                        { "name": "flag", "type": "bool" }
                    ],
                    "returns": "i32",
                    "requirements": ["TEST-001"],
                    "body": {
                        "op": "if",
                        "condition": {
                            "op": "and",
                            "left": { "op": "param", "name": "flag" },
                            "right": {
                                "op": "gte_s",
                                "left": { "op": "param", "name": "x" },
                                "right": { "op": "const", "type": "i32", "value": 10 }
                            }
                        },
                        "then": {
                            "op": "div_s",
                            "left": {
                                "op": "mul",
                                "left": { "op": "param", "name": "x" },
                                "right": { "op": "const", "type": "i32", "value": 3 }
                            },
                            "right": { "op": "const", "type": "i32", "value": 2 }
                        },
                        "else": {
                            "op": "if",
                            "condition": {
                                "op": "or",
                                "left": {
                                    "op": "not",
                                    "value": { "op": "param", "name": "flag" }
                                },
                                "right": {
                                    "op": "eq",
                                    "left": { "op": "param", "name": "x" },
                                    "right": { "op": "const", "type": "i32", "value": 0 }
                                }
                            },
                            "then": {
                                "op": "sub",
                                "left": {
                                    "op": "add",
                                    "left": { "op": "param", "name": "x" },
                                    "right": { "op": "const", "type": "i32", "value": 5 }
                                },
                                "right": { "op": "const", "type": "i32", "value": 1 }
                            },
                            "else": { "op": "const", "type": "i32", "value": -1 }
                        }
                    }
                }
            ]
        });

        let wasm_bytes = compile_ir_to_wasm(&ir).expect("IR must compile");
        let yaml = r#"
cases:
  - id: T1
    requirement: TEST-001
    function: compute_rule
    args: [10, true]
    expected: 15
  - id: T2
    requirement: TEST-001
    function: compute_rule
    args: [10, false]
    expected: 14
  - id: T3
    requirement: TEST-001
    function: compute_rule
    args: [4, true]
    expected: -1
"#;
        let cases = parse_cases(yaml).expect("parse YAML");
        let report = run_suite(&wasm_bytes, &cases).expect("run suite");
        assert_eq!(report.passed, 3);
        assert_eq!(report.failed, 0);
    }

    #[test]
    fn executes_v0_2_record_and_call_in_wasmtime() {
        let ir = json!({
            "ir_version": "0.2",
            "spec_version": 1,
            "types": [
                {
                    "name": "PointSum",
                    "fields": [
                        { "name": "sum", "type": "i32" }
                    ]
                }
            ],
            "functions": [
                {
                    "name": "add_coords",
                    "export": false,
                    "params": [
                        { "name": "x", "type": "i32" },
                        { "name": "y", "type": "i32" }
                    ],
                    "returns": "i32",
                    "requirements": [],
                    "body": {
                        "op": "add",
                        "left": { "op": "param", "name": "x" },
                        "right": { "op": "param", "name": "y" }
                    }
                },
                {
                    "name": "make_point_sum",
                    "export": true,
                    "params": [
                        { "name": "x", "type": "i32" },
                        { "name": "y", "type": "i32" }
                    ],
                    "returns": "PointSum",
                    "requirements": ["SMOKE2-001"],
                    "body": {
                        "op": "record.construct",
                        "type": "PointSum",
                        "fields": {
                            "sum": {
                                "op": "call",
                                "function": "add_coords",
                                "args": [
                                    { "op": "param", "name": "x" },
                                    { "op": "param", "name": "y" }
                                ]
                            }
                        }
                    }
                }
            ]
        });

        let wasm_bytes = compile_ir_to_wasm(&ir).expect("IR must compile");
        let yaml = r#"
cases:
  - id: S1
    requirement: SMOKE2-001
    function: make_point_sum
    args: [15, 25]
    expected:
      sum: 40
"#;
        let cases = parse_cases(yaml).expect("parse YAML");
        let report = run_suite(&wasm_bytes, &cases).expect("run suite");
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
    }
}
