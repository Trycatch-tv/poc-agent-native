use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use wasm_encoder::{
    BlockType, CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction,
    Module, TypeSection, ValType,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub function: Option<String>,
    pub path: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub found: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrType {
    I32,
    Bool,
}

impl IrType {
    pub fn as_str(self) -> &'static str {
        match self {
            IrType::I32 => "i32",
            IrType::Bool => "bool",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "i32" => Some(IrType::I32),
            "bool" => Some(IrType::Bool),
            _ => None,
        }
    }
}

fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Validates a Semantic IR v0.1 JSON document and returns all diagnostics.
/// An empty vector means the IR is both structurally and semantically valid.
pub fn verify_ir(root: &Value) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let Some(obj) = root.as_object() else {
        diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$".to_string(),
            message: "Root of IR document must be a JSON object.".to_string(),
            expected: Some("object".to_string()),
            found: Some(value_kind(root).to_string()),
        });
        return diags;
    };

    for key in obj.keys() {
        if !matches!(key.as_str(), "ir_version" | "spec_version" | "functions") {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: None,
                path: format!("$.{key}"),
                message: format!("Unknown root property '{key}'."),
                expected: Some("ir_version | spec_version | functions".to_string()),
                found: Some(key.clone()),
            });
        }
    }

    match obj.get("ir_version") {
        Some(Value::String(v)) if v == "0.1" => {}
        Some(other) => diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$.ir_version".to_string(),
            message: "ir_version must be the string \"0.1\".".to_string(),
            expected: Some("\"0.1\"".to_string()),
            found: Some(other.to_string()),
        }),
        None => diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$.ir_version".to_string(),
            message: "Missing required property 'ir_version'.".to_string(),
            expected: Some("\"0.1\"".to_string()),
            found: None,
        }),
    }

    match obj.get("spec_version") {
        Some(Value::Number(n)) if n.as_i64().is_some_and(|v| v >= 1) => {}
        Some(other) => diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$.spec_version".to_string(),
            message: "spec_version must be an integer >= 1.".to_string(),
            expected: Some("integer >= 1".to_string()),
            found: Some(other.to_string()),
        }),
        None => diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$.spec_version".to_string(),
            message: "Missing required property 'spec_version'.".to_string(),
            expected: Some("integer >= 1".to_string()),
            found: None,
        }),
    }

    let Some(functions_val) = obj.get("functions") else {
        diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$.functions".to_string(),
            message: "Missing required property 'functions'.".to_string(),
            expected: Some("non-empty array".to_string()),
            found: None,
        });
        return diags;
    };

    let Some(functions) = functions_val.as_array() else {
        diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$.functions".to_string(),
            message: "'functions' must be an array.".to_string(),
            expected: Some("array".to_string()),
            found: Some(value_kind(functions_val).to_string()),
        });
        return diags;
    };

    if functions.is_empty() {
        diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: None,
            path: "$.functions".to_string(),
            message: "'functions' must contain at least 1 function.".to_string(),
            expected: Some("minItems: 1".to_string()),
            found: Some("0".to_string()),
        });
        return diags;
    }

    let mut seen_functions = HashSet::new();

    for (idx, func_val) in functions.iter().enumerate() {
        let func_path = format!("$.functions[{idx}]");
        let Some(func_obj) = func_val.as_object() else {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: None,
                path: func_path,
                message: "Function entry must be a JSON object.".to_string(),
                expected: Some("object".to_string()),
                found: Some(value_kind(func_val).to_string()),
            });
            continue;
        };

        let func_name = func_obj
            .get("name")
            .and_then(|v| v.as_str())
            .map(ToOwned::to_owned);

        for key in func_obj.keys() {
            if !matches!(
                key.as_str(),
                "name" | "export" | "params" | "returns" | "requirements" | "body"
            ) {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.clone(),
                    path: format!("{func_path}.{key}"),
                    message: format!("Unknown property '{key}' in function object."),
                    expected: Some(
                        "name | export | params | returns | requirements | body".to_string(),
                    ),
                    found: Some(key.clone()),
                });
            }
        }

        match &func_name {
            Some(name) if is_valid_identifier(name) => {
                if !seen_functions.insert(name.clone()) {
                    diags.push(Diagnostic {
                        code: "E_DUPLICATE_FUNCTION".to_string(),
                        function: Some(name.clone()),
                        path: format!("{func_path}.name"),
                        message: format!("Duplicate function name '{name}'."),
                        expected: Some("unique function name".to_string()),
                        found: Some(name.clone()),
                    });
                }
            }
            Some(invalid) => {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: Some(invalid.clone()),
                    path: format!("{func_path}.name"),
                    message: format!("Invalid function identifier '{invalid}'."),
                    expected: Some("^[A-Za-z_][A-Za-z0-9_]*$".to_string()),
                    found: Some(invalid.clone()),
                });
            }
            None => {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: None,
                    path: format!("{func_path}.name"),
                    message: "Function is missing a valid string 'name'.".to_string(),
                    expected: Some("string".to_string()),
                    found: None,
                });
            }
        }

        let is_export = match func_obj.get("export") {
            Some(Value::Bool(b)) => *b,
            Some(other) => {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.clone(),
                    path: format!("{func_path}.export"),
                    message: "'export' must be a boolean.".to_string(),
                    expected: Some("boolean".to_string()),
                    found: Some(value_kind(other).to_string()),
                });
                false
            }
            None => {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.clone(),
                    path: format!("{func_path}.export"),
                    message: "Missing required property 'export'.".to_string(),
                    expected: Some("boolean".to_string()),
                    found: None,
                });
                false
            }
        };

        // Check requirements (E_MISSING_REQUIREMENT)
        match func_obj.get("requirements") {
            Some(Value::Array(reqs)) => {
                if is_export && reqs.is_empty() {
                    diags.push(Diagnostic {
                        code: "E_MISSING_REQUIREMENT".to_string(),
                        function: func_name.clone(),
                        path: format!("{func_path}.requirements"),
                        message:
                            "Exported function must reference at least one requirement ID."
                                .to_string(),
                        expected: Some("non-empty array of requirement IDs".to_string()),
                        found: Some("[]".to_string()),
                    });
                }
                let mut seen_reqs = HashSet::new();
                for (r_idx, req) in reqs.iter().enumerate() {
                    match req.as_str() {
                        Some(s) if !s.trim().is_empty() => {
                            if !seen_reqs.insert(s.to_string()) {
                                diags.push(Diagnostic {
                                    code: "E_SCHEMA_INVALID".to_string(),
                                    function: func_name.clone(),
                                    path: format!("{func_path}.requirements[{r_idx}]"),
                                    message: format!("Duplicate requirement ID '{s}'."),
                                    expected: Some("unique requirement IDs".to_string()),
                                    found: Some(s.to_string()),
                                });
                            }
                        }
                        _ => {
                            diags.push(Diagnostic {
                                code: "E_MISSING_REQUIREMENT".to_string(),
                                function: func_name.clone(),
                                path: format!("{func_path}.requirements[{r_idx}]"),
                                message: "Requirement ID must be a non-empty string.".to_string(),
                                expected: Some("non-empty string".to_string()),
                                found: Some(req.to_string()),
                            });
                        }
                    }
                }
            }
            Some(other) => diags.push(Diagnostic {
                code: "E_MISSING_REQUIREMENT".to_string(),
                function: func_name.clone(),
                path: format!("{func_path}.requirements"),
                message: "'requirements' must be an array of strings.".to_string(),
                expected: Some("array".to_string()),
                found: Some(value_kind(other).to_string()),
            }),
            None => diags.push(Diagnostic {
                code: "E_MISSING_REQUIREMENT".to_string(),
                function: func_name.clone(),
                path: format!("{func_path}.requirements"),
                message: "Missing required property 'requirements'.".to_string(),
                expected: Some("array of requirement IDs".to_string()),
                found: None,
            }),
        }

        // Check params (E_DUPLICATE_PARAM)
        let mut param_types: HashMap<String, IrType> = HashMap::new();
        match func_obj.get("params") {
            Some(Value::Array(params)) => {
                for (p_idx, p_val) in params.iter().enumerate() {
                    let p_path = format!("{func_path}.params[{p_idx}]");
                    let Some(p_obj) = p_val.as_object() else {
                        diags.push(Diagnostic {
                            code: "E_SCHEMA_INVALID".to_string(),
                            function: func_name.clone(),
                            path: p_path,
                            message: "Parameter must be a JSON object.".to_string(),
                            expected: Some("object".to_string()),
                            found: Some(value_kind(p_val).to_string()),
                        });
                        continue;
                    };
                    let p_name = p_obj.get("name").and_then(|v| v.as_str());
                    let p_type_str = p_obj.get("type").and_then(|v| v.as_str());

                    let valid_name = match p_name {
                        Some(n) if is_valid_identifier(n) => Some(n.to_string()),
                        Some(invalid) => {
                            diags.push(Diagnostic {
                                code: "E_SCHEMA_INVALID".to_string(),
                                function: func_name.clone(),
                                path: format!("{p_path}.name"),
                                message: format!("Invalid parameter identifier '{invalid}'."),
                                expected: Some("^[A-Za-z_][A-Za-z0-9_]*$".to_string()),
                                found: Some(invalid.to_string()),
                            });
                            None
                        }
                        None => {
                            diags.push(Diagnostic {
                                code: "E_SCHEMA_INVALID".to_string(),
                                function: func_name.clone(),
                                path: format!("{p_path}.name"),
                                message: "Parameter is missing 'name'.".to_string(),
                                expected: Some("string".to_string()),
                                found: None,
                            });
                            None
                        }
                    };

                    let valid_type = match p_type_str.and_then(IrType::from_str) {
                        Some(t) => Some(t),
                        None => {
                            diags.push(Diagnostic {
                                code: "E_TYPE_MISMATCH".to_string(),
                                function: func_name.clone(),
                                path: format!("{p_path}.type"),
                                message: "Invalid parameter type; must be 'i32' or 'bool'."
                                    .to_string(),
                                expected: Some("i32 | bool".to_string()),
                                found: p_type_str.map(ToOwned::to_owned),
                            });
                            None
                        }
                    };

                    if let (Some(n), Some(t)) = (valid_name, valid_type) {
                        if param_types.insert(n.clone(), t).is_some() {
                            diags.push(Diagnostic {
                                code: "E_DUPLICATE_PARAM".to_string(),
                                function: func_name.clone(),
                                path: format!("{p_path}.name"),
                                message: format!(
                                    "Duplicate parameter '{n}' in function '{}'.",
                                    func_name.as_deref().unwrap_or("<anonymous>")
                                ),
                                expected: Some("unique parameter name".to_string()),
                                found: Some(n),
                            });
                        }
                    }
                }
            }
            Some(other) => diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: func_name.clone(),
                path: format!("{func_path}.params"),
                message: "'params' must be an array.".to_string(),
                expected: Some("array".to_string()),
                found: Some(value_kind(other).to_string()),
            }),
            None => diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: func_name.clone(),
                path: format!("{func_path}.params"),
                message: "Missing required property 'params'.".to_string(),
                expected: Some("array".to_string()),
                found: None,
            }),
        }

        let declared_return = match func_obj.get("returns").and_then(|v| v.as_str()) {
            Some(s) => match IrType::from_str(s) {
                Some(t) => Some(t),
                None => {
                    diags.push(Diagnostic {
                        code: "E_RETURN_TYPE".to_string(),
                        function: func_name.clone(),
                        path: format!("{func_path}.returns"),
                        message: format!("Unsupported return type '{s}'; expected 'i32' or 'bool'."),
                        expected: Some("i32 | bool".to_string()),
                        found: Some(s.to_string()),
                    });
                    None
                }
            },
            None => {
                diags.push(Diagnostic {
                    code: "E_RETURN_TYPE".to_string(),
                    function: func_name.clone(),
                    path: format!("{func_path}.returns"),
                    message: "Missing required string property 'returns'.".to_string(),
                    expected: Some("i32 | bool".to_string()),
                    found: None,
                });
                None
            }
        };

        match func_obj.get("body") {
            Some(body_val) => {
                let body_path = format!("{func_path}.body");
                let body_type = check_expr(
                    body_val,
                    &body_path,
                    func_name.as_deref(),
                    &param_types,
                    &mut diags,
                );
                if let (Some(ret_ty), Some(actual_ty)) = (declared_return, body_type) {
                    if ret_ty != actual_ty {
                        diags.push(Diagnostic {
                            code: "E_RETURN_TYPE".to_string(),
                            function: func_name.clone(),
                            path: body_path,
                            message: format!(
                                "Function body evaluates to '{}', which does not match declared return type '{}'.",
                                actual_ty.as_str(),
                                ret_ty.as_str()
                            ),
                            expected: Some(ret_ty.as_str().to_string()),
                            found: Some(actual_ty.as_str().to_string()),
                        });
                    }
                }
            }
            None => diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: func_name.clone(),
                path: format!("{func_path}.body"),
                message: "Missing required property 'body'.".to_string(),
                expected: Some("expression object".to_string()),
                found: None,
            }),
        }
    }

    diags
}

fn check_expr(
    expr: &Value,
    path: &str,
    func_name: Option<&str>,
    params: &HashMap<String, IrType>,
    diags: &mut Vec<Diagnostic>,
) -> Option<IrType> {
    let Some(obj) = expr.as_object() else {
        diags.push(Diagnostic {
            code: "E_SCHEMA_INVALID".to_string(),
            function: func_name.map(ToOwned::to_owned),
            path: path.to_string(),
            message: "Expression must be a JSON object.".to_string(),
            expected: Some("object".to_string()),
            found: Some(value_kind(expr).to_string()),
        });
        return None;
    };

    let Some(op) = obj.get("op").and_then(|v| v.as_str()) else {
        diags.push(Diagnostic {
            code: "E_UNSUPPORTED_OP".to_string(),
            function: func_name.map(ToOwned::to_owned),
            path: format!("{path}.op"),
            message: "Expression is missing a string 'op' field.".to_string(),
            expected: Some("valid operator string".to_string()),
            found: obj.get("op").map(ToString::to_string),
        });
        return None;
    };

    match op {
        "const" => {
            for key in obj.keys() {
                if !matches!(key.as_str(), "op" | "type" | "value") {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.{key}"),
                        message: format!("Unknown property '{key}' in 'const' expression."),
                        expected: Some("op | type | value".to_string()),
                        found: Some(key.clone()),
                    });
                }
            }
            let type_str = obj.get("type").and_then(|v| v.as_str());
            let Some(ir_ty) = type_str.and_then(IrType::from_str) else {
                diags.push(Diagnostic {
                    code: "E_CONST_TYPE".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.type"),
                    message: "Const expression 'type' must be 'i32' or 'bool'.".to_string(),
                    expected: Some("i32 | bool".to_string()),
                    found: obj.get("type").map(ToString::to_string),
                });
                return None;
            };
            let Some(val) = obj.get("value") else {
                diags.push(Diagnostic {
                    code: "E_CONST_TYPE".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.value"),
                    message: "Const expression is missing 'value'.".to_string(),
                    expected: Some(ir_ty.as_str().to_string()),
                    found: None,
                });
                return None;
            };
            match (ir_ty, val) {
                (IrType::I32, Value::Number(n)) => {
                    if let Some(i) = n.as_i64() {
                        if i >= i64::from(i32::MIN) && i <= i64::from(i32::MAX) {
                            Some(IrType::I32)
                        } else {
                            diags.push(Diagnostic {
                                code: "E_CONST_TYPE".to_string(),
                                function: func_name.map(ToOwned::to_owned),
                                path: format!("{path}.value"),
                                message: format!("Integer literal {i} is out of signed i32 range."),
                                expected: Some("i32 (-2147483648..=2147483647)".to_string()),
                                found: Some(i.to_string()),
                            });
                            None
                        }
                    } else {
                        diags.push(Diagnostic {
                            code: "E_CONST_TYPE".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: format!("{path}.value"),
                            message: "Const of type 'i32' requires a signed integer value."
                                .to_string(),
                            expected: Some("integer".to_string()),
                            found: Some(val.to_string()),
                        });
                        None
                    }
                }
                (IrType::Bool, Value::Bool(_)) => Some(IrType::Bool),
                (expected_ty, other_val) => {
                    diags.push(Diagnostic {
                        code: "E_CONST_TYPE".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.value"),
                        message: format!(
                            "Const value {} does not match declared const type '{}'.",
                            other_val,
                            expected_ty.as_str()
                        ),
                        expected: Some(expected_ty.as_str().to_string()),
                        found: Some(value_kind(other_val).to_string()),
                    });
                    None
                }
            }
        }
        "param" => {
            for key in obj.keys() {
                if !matches!(key.as_str(), "op" | "name") {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.{key}"),
                        message: format!("Unknown property '{key}' in 'param' expression."),
                        expected: Some("op | name".to_string()),
                        found: Some(key.clone()),
                    });
                }
            }
            let Some(name) = obj.get("name").and_then(|v| v.as_str()) else {
                diags.push(Diagnostic {
                    code: "E_UNKNOWN_PARAM".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.name"),
                    message: "Param expression is missing string 'name'.".to_string(),
                    expected: Some("declared parameter name".to_string()),
                    found: None,
                });
                return None;
            };
            if let Some(ty) = params.get(name) {
                Some(*ty)
            } else {
                let mut available: Vec<&str> = params.keys().map(String::as_str).collect();
                available.sort_unstable();
                diags.push(Diagnostic {
                    code: "E_UNKNOWN_PARAM".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: path.to_string(),
                    message: format!(
                        "Unknown parameter '{name}'; available parameters: {available:?}"
                    ),
                    expected: Some(available.join(" | ")),
                    found: Some(name.to_string()),
                });
                None
            }
        }
        "not" => {
            for key in obj.keys() {
                if !matches!(key.as_str(), "op" | "value") {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.{key}"),
                        message: format!("Unknown property '{key}' in 'not' expression."),
                        expected: Some("op | value".to_string()),
                        found: Some(key.clone()),
                    });
                }
            }
            let Some(inner) = obj.get("value") else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.value"),
                    message: "Unary 'not' expression is missing 'value'.".to_string(),
                    expected: Some("bool expression".to_string()),
                    found: None,
                });
                return None;
            };
            let inner_ty = check_expr(
                inner,
                &format!("{path}.value"),
                func_name,
                params,
                diags,
            )?;
            if inner_ty != IrType::Bool {
                diags.push(Diagnostic {
                    code: "E_TYPE_MISMATCH".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: path.to_string(),
                    message: format!(
                        "Operator 'not' requires operand of type 'bool', found '{}'.",
                        inner_ty.as_str()
                    ),
                    expected: Some("bool".to_string()),
                    found: Some(inner_ty.as_str().to_string()),
                });
                return None;
            }
            Some(IrType::Bool)
        }
        "add" | "sub" | "mul" | "div_s" | "lt_s" | "gte_s" | "eq" | "and" | "or" => {
            for key in obj.keys() {
                if !matches!(key.as_str(), "op" | "left" | "right") {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.{key}"),
                        message: format!("Unknown property '{key}' in binary expression '{op}'."),
                        expected: Some("op | left | right".to_string()),
                        found: Some(key.clone()),
                    });
                }
            }
            let left_val = obj.get("left");
            let right_val = obj.get("right");
            if left_val.is_none() || right_val.is_none() {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: path.to_string(),
                    message: format!("Binary operator '{op}' requires both 'left' and 'right'."),
                    expected: Some("left and right expressions".to_string()),
                    found: None,
                });
                return None;
            }
            let left_val = left_val.unwrap();
            let right_val = right_val.unwrap();

            let left_ty = check_expr(
                left_val,
                &format!("{path}.left"),
                func_name,
                params,
                diags,
            );
            let right_ty = check_expr(
                right_val,
                &format!("{path}.right"),
                func_name,
                params,
                diags,
            );

            // Check static division by zero (E_CONST_DIV_ZERO)
            if op == "div_s" && is_const_zero(right_val) {
                diags.push(Diagnostic {
                    code: "E_CONST_DIV_ZERO".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.right"),
                    message: "Static division by zero is not allowed in 'div_s'.".to_string(),
                    expected: Some("non-zero i32 divisor".to_string()),
                    found: Some("0".to_string()),
                });
            }

            let (Some(lt), Some(rt)) = (left_ty, right_ty) else {
                return None;
            };

            match op {
                "add" | "sub" | "mul" | "div_s" => {
                    if lt == IrType::I32 && rt == IrType::I32 {
                        Some(IrType::I32)
                    } else {
                        diags.push(Diagnostic {
                            code: "E_TYPE_MISMATCH".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: path.to_string(),
                            message: format!(
                                "Arithmetic operator '{op}' requires (i32, i32), found ({}, {}).",
                                lt.as_str(),
                                rt.as_str()
                            ),
                            expected: Some("(i32, i32)".to_string()),
                            found: Some(format!("({}, {})", lt.as_str(), rt.as_str())),
                        });
                        None
                    }
                }
                "lt_s" | "gte_s" => {
                    if lt == IrType::I32 && rt == IrType::I32 {
                        Some(IrType::Bool)
                    } else {
                        diags.push(Diagnostic {
                            code: "E_TYPE_MISMATCH".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: path.to_string(),
                            message: format!(
                                "Comparison operator '{op}' requires (i32, i32), found ({}, {}).",
                                lt.as_str(),
                                rt.as_str()
                            ),
                            expected: Some("(i32, i32)".to_string()),
                            found: Some(format!("({}, {})", lt.as_str(), rt.as_str())),
                        });
                        None
                    }
                }
                "eq" => {
                    if lt == rt {
                        Some(IrType::Bool)
                    } else {
                        diags.push(Diagnostic {
                            code: "E_TYPE_MISMATCH".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: path.to_string(),
                            message: format!(
                                "Equality operator 'eq' requires matching operand types (i32, i32) or (bool, bool), found ({}, {}).",
                                lt.as_str(),
                                rt.as_str()
                            ),
                            expected: Some(format!("({}, {})", lt.as_str(), lt.as_str())),
                            found: Some(format!("({}, {})", lt.as_str(), rt.as_str())),
                        });
                        None
                    }
                }
                "and" | "or" => {
                    if lt == IrType::Bool && rt == IrType::Bool {
                        Some(IrType::Bool)
                    } else {
                        diags.push(Diagnostic {
                            code: "E_TYPE_MISMATCH".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: path.to_string(),
                            message: format!(
                                "Logical operator '{op}' requires (bool, bool), found ({}, {}).",
                                lt.as_str(),
                                rt.as_str()
                            ),
                            expected: Some("(bool, bool)".to_string()),
                            found: Some(format!("({}, {})", lt.as_str(), rt.as_str())),
                        });
                        None
                    }
                }
                _ => unreachable!(),
            }
        }
        "if" => {
            for key in obj.keys() {
                if !matches!(key.as_str(), "op" | "condition" | "then" | "else") {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.{key}"),
                        message: format!("Unknown property '{key}' in 'if' expression."),
                        expected: Some("op | condition | then | else".to_string()),
                        found: Some(key.clone()),
                    });
                }
            }
            let cond_val = obj.get("condition");
            let then_val = obj.get("then");
            let else_val = obj.get("else");
            if cond_val.is_none() || then_val.is_none() || else_val.is_none() {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: path.to_string(),
                    message: "'if' expression requires 'condition', 'then', and 'else'."
                        .to_string(),
                    expected: Some("condition, then, else".to_string()),
                    found: None,
                });
                return None;
            }
            let cond_ty = check_expr(
                cond_val.unwrap(),
                &format!("{path}.condition"),
                func_name,
                params,
                diags,
            );
            let then_ty = check_expr(
                then_val.unwrap(),
                &format!("{path}.then"),
                func_name,
                params,
                diags,
            );
            let else_ty = check_expr(
                else_val.unwrap(),
                &format!("{path}.else"),
                func_name,
                params,
                diags,
            );

            if let Some(ct) = cond_ty {
                if ct != IrType::Bool {
                    diags.push(Diagnostic {
                        code: "E_IF_CONDITION".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.condition"),
                        message: format!(
                            "'if.condition' must evaluate to 'bool', found '{}'.",
                            ct.as_str()
                        ),
                        expected: Some("bool".to_string()),
                        found: Some(ct.as_str().to_string()),
                    });
                }
            }

            if let (Some(tt), Some(et)) = (then_ty, else_ty) {
                if tt != et {
                    diags.push(Diagnostic {
                        code: "E_IF_BRANCH_TYPE".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: path.to_string(),
                        message: format!(
                            "'if' branches must return the same type, found then='{}' and else='{}'.",
                            tt.as_str(),
                            et.as_str()
                        ),
                        expected: Some(tt.as_str().to_string()),
                        found: Some(et.as_str().to_string()),
                    });
                    return None;
                }
                return Some(tt);
            }
            None
        }
        other => {
            diags.push(Diagnostic {
                code: "E_UNSUPPORTED_OP".to_string(),
                function: func_name.map(ToOwned::to_owned),
                path: format!("{path}.op"),
                message: format!("Unsupported operator '{other}' in Semantic IR v0.1."),
                expected: Some(
                    "const | param | not | add | sub | mul | div_s | eq | lt_s | gte_s | and | or | if"
                        .to_string(),
                ),
                found: Some(other.to_string()),
            });
            None
        }
    }
}

fn is_const_zero(expr: &Value) -> bool {
    let Some(obj) = expr.as_object() else {
        return false;
    };
    obj.get("op").and_then(|v| v.as_str()) == Some("const")
        && obj.get("value").and_then(|v| v.as_i64()) == Some(0)
}

fn value_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Compiles a verified Semantic IR v0.1 JSON document directly into deterministic WebAssembly bytes.
pub fn compile_ir_to_wasm(root: &Value) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let diags = verify_ir(root);
    if !diags.is_empty() {
        return Err(diags);
    }

    let functions = root["functions"].as_array().expect("verified functions array");

    let mut module = Module::new();
    let mut types = TypeSection::new();
    let mut func_section = FunctionSection::new();
    let mut exports = ExportSection::new();
    let mut code = CodeSection::new();

    for (func_idx, func_val) in functions.iter().enumerate() {
        let func_obj = func_val.as_object().unwrap();
        let name = func_obj["name"].as_str().unwrap();
        let is_export = func_obj["export"].as_bool().unwrap();
        let params_arr = func_obj["params"].as_array().unwrap();
        let body = &func_obj["body"];

        let mut param_map = HashMap::new();
        let mut wasm_params = Vec::with_capacity(params_arr.len());
        for (p_idx, p_val) in params_arr.iter().enumerate() {
            let p_name = p_val["name"].as_str().unwrap().to_string();
            param_map.insert(p_name, p_idx as u32);
            wasm_params.push(ValType::I32);
        }

        let results = [ValType::I32];
        types.ty().function(wasm_params, results);
        func_section.function(func_idx as u32);

        if is_export {
            exports.export(name, ExportKind::Func, func_idx as u32);
        }

        let mut wasm_func = Function::new([]);
        emit_expr(body, &param_map, &mut wasm_func);
        wasm_func.instruction(&Instruction::End);
        code.function(&wasm_func);
    }

    module.section(&types);
    module.section(&func_section);
    module.section(&exports);
    module.section(&code);

    let wasm_bytes = module.finish();
    Ok(wasm_bytes)
}

fn emit_expr(expr: &Value, params: &HashMap<String, u32>, func: &mut Function) {
    let obj = expr.as_object().unwrap();
    let op = obj["op"].as_str().unwrap();
    match op {
        "const" => {
            let ty = obj["type"].as_str().unwrap();
            let val_i32 = match ty {
                "i32" => obj["value"].as_i64().unwrap() as i32,
                "bool" => {
                    if obj["value"].as_bool().unwrap() {
                        1
                    } else {
                        0
                    }
                }
                _ => unreachable!(),
            };
            func.instruction(&Instruction::I32Const(val_i32));
        }
        "param" => {
            let name = obj["name"].as_str().unwrap();
            let local_idx = params[name];
            func.instruction(&Instruction::LocalGet(local_idx));
        }
        "not" => {
            emit_expr(&obj["value"], params, func);
            func.instruction(&Instruction::I32Eqz);
        }
        "add" | "sub" | "mul" | "div_s" | "eq" | "lt_s" | "gte_s" | "and" | "or" => {
            emit_expr(&obj["left"], params, func);
            emit_expr(&obj["right"], params, func);
            match op {
                "add" => func.instruction(&Instruction::I32Add),
                "sub" => func.instruction(&Instruction::I32Sub),
                "mul" => func.instruction(&Instruction::I32Mul),
                "div_s" => func.instruction(&Instruction::I32DivS),
                "eq" => func.instruction(&Instruction::I32Eq),
                "lt_s" => func.instruction(&Instruction::I32LtS),
                "gte_s" => func.instruction(&Instruction::I32GeS),
                "and" => func.instruction(&Instruction::I32And),
                "or" => func.instruction(&Instruction::I32Or),
                _ => unreachable!(),
            };
        }
        "if" => {
            emit_expr(&obj["condition"], params, func);
            func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
            emit_expr(&obj["then"], params, func);
            func.instruction(&Instruction::Else);
            emit_expr(&obj["else"], params, func);
            func.instruction(&Instruction::End);
        }
        _ => unreachable!(),
    }
}

/// Validates a WebAssembly binary using `wasmparser` (the engine behind `wasm-tools validate`).
pub fn validate_wasm_bytes(bytes: &[u8]) -> Result<(), String> {
    wasmparser::Validator::new()
        .validate_all(bytes)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Converts a WebAssembly binary into WAT text format using `wasmprinter` (the engine behind `wasm-tools print`).
pub fn print_wasm_to_wat(bytes: &[u8]) -> Result<String, String> {
    wasmprinter::print_bytes(bytes).map_err(|e| e.to_string())
}

/// Computes the lowercase hex SHA-256 digest of a byte slice.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn valid_smoke_ir_passes_and_compiles_reproducibly() {
        let ir = json!({
            "ir_version": "0.1",
            "spec_version": 1,
            "functions": [
                {
                    "name": "identity",
                    "export": true,
                    "params": [{ "name": "value", "type": "i32" }],
                    "returns": "i32",
                    "requirements": ["SMOKE-001"],
                    "body": { "op": "param", "name": "value" }
                }
            ]
        });

        let diags = verify_ir(&ir);
        assert!(diags.is_empty(), "Expected no diagnostics, got: {diags:?}");

        let b1 = compile_ir_to_wasm(&ir).expect("compile 1");
        let b2 = compile_ir_to_wasm(&ir).expect("compile 2");
        let b3 = compile_ir_to_wasm(&ir).expect("compile 3");

        validate_wasm_bytes(&b1).expect("wasm validation must succeed");
        assert_eq!(sha256_hex(&b1), sha256_hex(&b2));
        assert_eq!(sha256_hex(&b2), sha256_hex(&b3));

        let wat = print_wasm_to_wat(&b1).expect("wat printing must succeed");
        assert!(wat.contains("(export \"identity\""));
    }

    #[test]
    fn detects_all_11_semantic_error_codes() {
        // 1. E_DUPLICATE_FUNCTION
        let dup_fn = json!({
            "ir_version": "0.1",
            "spec_version": 1,
            "functions": [
                {
                    "name": "f",
                    "export": true,
                    "params": [],
                    "returns": "i32",
                    "requirements": ["R1"],
                    "body": { "op": "const", "type": "i32", "value": 1 }
                },
                {
                    "name": "f",
                    "export": true,
                    "params": [],
                    "returns": "i32",
                    "requirements": ["R2"],
                    "body": { "op": "const", "type": "i32", "value": 2 }
                }
            ]
        });
        assert!(verify_ir(&dup_fn).iter().any(|d| d.code == "E_DUPLICATE_FUNCTION"));

        // 2. E_DUPLICATE_PARAM
        let dup_param = json!({
            "ir_version": "0.1",
            "spec_version": 1,
            "functions": [{
                "name": "f",
                "export": true,
                "params": [
                    { "name": "x", "type": "i32" },
                    { "name": "x", "type": "i32" }
                ],
                "returns": "i32",
                "requirements": ["R1"],
                "body": { "op": "param", "name": "x" }
            }]
        });
        assert!(verify_ir(&dup_param).iter().any(|d| d.code == "E_DUPLICATE_PARAM"));

        // 3. E_UNKNOWN_PARAM
        let unk_param = json!({
            "ir_version": "0.1",
            "spec_version": 1,
            "functions": [{
                "name": "f",
                "export": true,
                "params": [{ "name": "x", "type": "i32" }],
                "returns": "i32",
                "requirements": ["R1"],
                "body": { "op": "param", "name": "y" }
            }]
        });
        assert!(verify_ir(&unk_param).iter().any(|d| d.code == "E_UNKNOWN_PARAM"));

        // 4. E_TYPE_MISMATCH
        let type_mismatch = json!({
            "ir_version": "0.1",
            "spec_version": 1,
            "functions": [{
                "name": "f",
                "export": true,
                "params": [{ "name": "x", "type": "i32" }],
                "returns": "i32",
                "requirements": ["R1"],
                "body": {
                    "op": "add",
                    "left": { "op": "param", "name": "x" },
                    "right": { "op": "const", "type": "bool", "value": true }
                }
            }]
        });
        assert!(verify_ir(&type_mismatch).iter().any(|d| d.code == "E_TYPE_MISMATCH"));

        // 5. E_IF_CONDITION
        let if_cond = json!({
            "ir_version": "0.1",
            "spec_version": 1,
            "functions": [{
                "name": "f",
                "export": true,
                "params": [],
                "returns": "i32",
                "requirements": ["R1"],
                "body": {
                    "op": "if",
                    "condition": { "op": "const", "type": "i32", "value": 1 },
                    "then": { "op": "const", "type": "i32", "value": 10 },
                    "else": { "op": "const", "type": "i32", "value": 20 }
                }
            }]
        });
        assert!(verify_ir(&if_cond).iter().any(|d| d.code == "E_IF_CONDITION"));

        // 6. E_IF_BRANCH_TYPE
        let if_branch = json!({
            "ir_version": "0.1",
            "spec_version": 1,
            "functions": [{
                "name": "f",
                "export": true,
                "params": [],
                "returns": "i32",
                "requirements": ["R1"],
                "body": {
                    "op": "if",
                    "condition": { "op": "const", "type": "bool", "value": true },
                    "then": { "op": "const", "type": "i32", "value": 10 },
                    "else": { "op": "const", "type": "bool", "value": false }
                }
            }]
        });
        assert!(verify_ir(&if_branch).iter().any(|d| d.code == "E_IF_BRANCH_TYPE"));

        // 7. E_RETURN_TYPE
        let ret_type = json!({
            "ir_version": "0.1",
            "spec_version": 1,
            "functions": [{
                "name": "f",
                "export": true,
                "params": [],
                "returns": "i32",
                "requirements": ["R1"],
                "body": { "op": "const", "type": "bool", "value": true }
            }]
        });
        assert!(verify_ir(&ret_type).iter().any(|d| d.code == "E_RETURN_TYPE"));

        // 8. E_UNSUPPORTED_OP
        let unsup_op = json!({
            "ir_version": "0.1",
            "spec_version": 1,
            "functions": [{
                "name": "f",
                "export": true,
                "params": [],
                "returns": "i32",
                "requirements": ["R1"],
                "body": { "op": "pow", "left": { "op": "const", "type": "i32", "value": 2 }, "right": { "op": "const", "type": "i32", "value": 3 } }
            }]
        });
        assert!(verify_ir(&unsup_op).iter().any(|d| d.code == "E_UNSUPPORTED_OP"));

        // 9. E_CONST_TYPE
        let const_type = json!({
            "ir_version": "0.1",
            "spec_version": 1,
            "functions": [{
                "name": "f",
                "export": true,
                "params": [],
                "returns": "i32",
                "requirements": ["R1"],
                "body": { "op": "const", "type": "i32", "value": true }
            }]
        });
        assert!(verify_ir(&const_type).iter().any(|d| d.code == "E_CONST_TYPE"));

        // 10. E_CONST_DIV_ZERO
        let div_zero = json!({
            "ir_version": "0.1",
            "spec_version": 1,
            "functions": [{
                "name": "f",
                "export": true,
                "params": [{ "name": "x", "type": "i32" }],
                "returns": "i32",
                "requirements": ["R1"],
                "body": {
                    "op": "div_s",
                    "left": { "op": "param", "name": "x" },
                    "right": { "op": "const", "type": "i32", "value": 0 }
                }
            }]
        });
        assert!(verify_ir(&div_zero).iter().any(|d| d.code == "E_CONST_DIV_ZERO"));

        // 11. E_MISSING_REQUIREMENT
        let missing_req = json!({
            "ir_version": "0.1",
            "spec_version": 1,
            "functions": [{
                "name": "f",
                "export": true,
                "params": [],
                "returns": "i32",
                "requirements": [],
                "body": { "op": "const", "type": "i32", "value": 0 }
            }]
        });
        assert!(verify_ir(&missing_req).iter().any(|d| d.code == "E_MISSING_REQUIREMENT"));
    }
}
