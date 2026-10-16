use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, CustomSection, EntityType, ExportKind, ExportSection, Function,
    FunctionSection, GlobalSection, GlobalType, ImportSection, Instruction, MemArg,
    MemorySection, MemoryType, Module, TypeSection, ValType,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrType {
    I32,
    Bool,
    Record(String),
}

impl IrType {
    pub fn as_str(&self) -> &str {
        match self {
            IrType::I32 => "i32",
            IrType::Bool => "bool",
            IrType::Record(name) => name.as_str(),
        }
    }

    pub fn from_str(s: &str, known_records: &HashSet<String>) -> Option<Self> {
        match s {
            "i32" => Some(IrType::I32),
            "bool" => Some(IrType::Bool),
            other if known_records.contains(other) => Some(IrType::Record(other.to_string())),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSignature {
    pub module: String,
    pub name: String,
    pub capability: String,
    pub wit_interface: Option<String>,
    pub params: Vec<(String, IrType)>,
    pub returns: IrType,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordDef {
    pub name: String,
    pub fields: Vec<(String, IrType)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WitContractDef {
    pub package: String,
    pub world: String,
    pub export_interface: String,
    pub import_interfaces: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub ir_version: String,
    pub wit_contract: Option<WitContractDef>,
    pub capabilities: HashSet<String>,
    pub imports: HashMap<String, ImportSignature>,
    pub import_order: Vec<String>,
    pub records: HashMap<String, RecordDef>,
    pub functions: HashMap<String, FuncSignature>,
    pub num_imports: u32,
}

#[derive(Debug, Clone)]
pub struct FuncSignature {
    pub name: String,
    pub export: bool,
    pub wit_interface: Option<String>,
    pub params: Vec<(String, IrType)>,
    pub returns: IrType,
    pub index: u32,
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

/// Validates a Semantic IR v0.1 or v0.2 JSON document and returns all diagnostics.
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

    let ir_version = match obj.get("ir_version").and_then(|v| v.as_str()) {
        Some("0.1") => "0.1",
        Some("0.2") => "0.2",
        Some("0.3") => "0.3",
        Some("0.4") => "0.4",
        Some(other) => {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: None,
                path: "$.ir_version".to_string(),
                message: "ir_version must be \"0.1\", \"0.2\", \"0.3\", or \"0.4\".".to_string(),
                expected: Some("\"0.1\" | \"0.2\" | \"0.3\" | \"0.4\"".to_string()),
                found: Some(other.to_string()),
            });
            "0.1"
        }
        None => {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: None,
                path: "$.ir_version".to_string(),
                message: "Missing required property 'ir_version'.".to_string(),
                expected: Some("\"0.1\" | \"0.2\" | \"0.3\" | \"0.4\"".to_string()),
                found: None,
            });
            "0.1"
        }
    };

    for key in obj.keys() {
        let is_allowed = match ir_version {
            "0.1" => matches!(key.as_str(), "ir_version" | "spec_version" | "functions"),
            "0.2" => matches!(
                key.as_str(),
                "ir_version" | "spec_version" | "types" | "functions"
            ),
            "0.3" => matches!(
                key.as_str(),
                "ir_version" | "spec_version" | "capabilities" | "imports" | "types" | "functions"
            ),
            "0.4" => matches!(
                key.as_str(),
                "ir_version" | "spec_version" | "wit_contract" | "capabilities" | "imports" | "types" | "functions"
            ),
            _ => false,
        };
        if !is_allowed {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: None,
                path: format!("$.{key}"),
                message: format!("Unknown root property '{key}'."),
                expected: Some(if ir_version == "0.1" {
                    "ir_version | spec_version | functions".to_string()
                } else if ir_version == "0.2" {
                    "ir_version | spec_version | types | functions".to_string()
                } else if ir_version == "0.3" {
                    "ir_version | spec_version | capabilities | imports | types | functions".to_string()
                } else {
                    "ir_version | spec_version | wit_contract | capabilities | imports | types | functions".to_string()
                }),
                found: Some(key.clone()),
            });
        }
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

    let mut wit_contract_def = None;
    if ir_version == "0.4" {
        if let Some(wit_val) = obj.get("wit_contract") {
            if let Some(wit_obj) = wit_val.as_object() {
                let pkg = wit_obj.get("package").and_then(|v| v.as_str()).unwrap_or("");
                let world = wit_obj.get("world").and_then(|v| v.as_str()).unwrap_or("");
                let exp_iface = wit_obj.get("export_interface").and_then(|v| v.as_str()).unwrap_or("");
                let mut imp_ifaces = Vec::new();
                if let Some(arr) = wit_obj.get("import_interfaces").and_then(|v| v.as_array()) {
                    for iface_val in arr {
                        if let Some(s) = iface_val.as_str() {
                            imp_ifaces.push(s.to_string());
                        }
                    }
                }
                if pkg.is_empty() {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: None,
                        path: "$.wit_contract.package".to_string(),
                        message: "Missing 'package' in wit_contract.".to_string(),
                        expected: Some("package string (e.g. kaddo:commercial@0.1.0)".to_string()),
                        found: None,
                    });
                }
                if exp_iface.is_empty() {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: None,
                        path: "$.wit_contract.export_interface".to_string(),
                        message: "Missing 'export_interface' in wit_contract.".to_string(),
                        expected: Some("export_interface name".to_string()),
                        found: None,
                    });
                }
                wit_contract_def = Some(WitContractDef {
                    package: pkg.to_string(),
                    world: world.to_string(),
                    export_interface: exp_iface.to_string(),
                    import_interfaces: imp_ifaces,
                });
            } else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: None,
                    path: "$.wit_contract".to_string(),
                    message: "wit_contract must be an object.".to_string(),
                    expected: Some("object".to_string()),
                    found: Some(value_kind(wit_val).to_string()),
                });
            }
        } else {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: None,
                path: "$.wit_contract".to_string(),
                message: "Missing required property 'wit_contract' in IR v0.4.".to_string(),
                expected: Some("wit_contract object".to_string()),
                found: None,
            });
        }
    }

    let mut record_defs: HashMap<String, RecordDef> = HashMap::new();
    let mut known_record_names: HashSet<String> = HashSet::new();

    if let Some(types_val) = obj.get("types") {
        if let Some(types_arr) = types_val.as_array() {
            for (t_idx, type_val) in types_arr.iter().enumerate() {
                let path = format!("$.types[{t_idx}]");
                let Some(type_obj) = type_val.as_object() else {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: None,
                        path,
                        message: "Each type declaration must be a JSON object.".to_string(),
                        expected: Some("object".to_string()),
                        found: Some(value_kind(type_val).to_string()),
                    });
                    continue;
                };

                let name = match type_obj.get("name").and_then(|v| v.as_str()) {
                    Some(n) if is_valid_identifier(n) => n.to_string(),
                    Some(n) => {
                        diags.push(Diagnostic {
                            code: "E_SCHEMA_INVALID".to_string(),
                            function: None,
                            path: format!("{path}.name"),
                            message: format!("Type name '{n}' is not a valid identifier."),
                            expected: Some("^[A-Za-z_][A-Za-z0-9_]*$".to_string()),
                            found: Some(n.to_string()),
                        });
                        continue;
                    }
                    None => {
                        diags.push(Diagnostic {
                            code: "E_SCHEMA_INVALID".to_string(),
                            function: None,
                            path: format!("{path}.name"),
                            message: "Missing string property 'name' in type declaration."
                                .to_string(),
                            expected: Some("identifier string".to_string()),
                            found: None,
                        });
                        continue;
                    }
                };

                if known_record_names.contains(&name) {
                    diags.push(Diagnostic {
                        code: "E_DUPLICATE_TYPE".to_string(),
                        function: None,
                        path: format!("{path}.name"),
                        message: format!("Duplicate type name '{name}'."),
                        expected: Some("unique type name".to_string()),
                        found: Some(name.clone()),
                    });
                    continue;
                }
                known_record_names.insert(name.clone());

                let fields_arr = match type_obj.get("fields").and_then(|v| v.as_array()) {
                    Some(arr) if !arr.is_empty() => arr,
                    _ => {
                        diags.push(Diagnostic {
                            code: "E_SCHEMA_INVALID".to_string(),
                            function: None,
                            path: format!("{path}.fields"),
                            message: format!("Type '{name}' must have a non-empty 'fields' array."),
                            expected: Some("non-empty array".to_string()),
                            found: type_obj.get("fields").map(|v| value_kind(v).to_string()),
                        });
                        continue;
                    }
                };

                let mut field_names = HashSet::new();
                let mut parsed_fields = Vec::new();

                for (f_idx, f_val) in fields_arr.iter().enumerate() {
                    let f_path = format!("{path}.fields[{f_idx}]");
                    let Some(f_obj) = f_val.as_object() else {
                        diags.push(Diagnostic {
                            code: "E_SCHEMA_INVALID".to_string(),
                            function: None,
                            path: f_path,
                            message: "Field must be a JSON object.".to_string(),
                            expected: Some("object".to_string()),
                            found: Some(value_kind(f_val).to_string()),
                        });
                        continue;
                    };

                    let f_name = match f_obj.get("name").and_then(|v| v.as_str()) {
                        Some(fnm) if is_valid_identifier(fnm) => fnm.to_string(),
                        Some(fnm) => {
                            diags.push(Diagnostic {
                                code: "E_SCHEMA_INVALID".to_string(),
                                function: None,
                                path: format!("{f_path}.name"),
                                message: format!("Field name '{fnm}' is not a valid identifier."),
                                expected: Some("^[A-Za-z_][A-Za-z0-9_]*$".to_string()),
                                found: Some(fnm.to_string()),
                            });
                            continue;
                        }
                        None => {
                            diags.push(Diagnostic {
                                code: "E_SCHEMA_INVALID".to_string(),
                                function: None,
                                path: format!("{f_path}.name"),
                                message: "Missing string property 'name' in field.".to_string(),
                                expected: Some("identifier string".to_string()),
                                found: None,
                            });
                            continue;
                        }
                    };

                    if field_names.contains(&f_name) {
                        diags.push(Diagnostic {
                            code: "E_DUPLICATE_PARAM".to_string(),
                            function: None,
                            path: format!("{f_path}.name"),
                            message: format!(
                                "Duplicate field name '{f_name}' in record type '{name}'."
                            ),
                            expected: Some("unique field name".to_string()),
                            found: Some(f_name.clone()),
                        });
                    }
                    field_names.insert(f_name.clone());

                    let f_type_str = f_obj.get("type").and_then(|v| v.as_str());
                    let f_type = match f_type_str {
                        Some("i32") => IrType::I32,
                        Some("bool") => IrType::Bool,
                        Some(other) => {
                            diags.push(Diagnostic {
                                code: "E_UNKNOWN_TYPE".to_string(),
                                function: None,
                                path: format!("{f_path}.type"),
                                message: format!("Unsupported or unknown field type '{other}'."),
                                expected: Some("i32 | bool".to_string()),
                                found: Some(other.to_string()),
                            });
                            IrType::I32
                        }
                        None => {
                            diags.push(Diagnostic {
                                code: "E_SCHEMA_INVALID".to_string(),
                                function: None,
                                path: format!("{f_path}.type"),
                                message: "Missing 'type' in field declaration.".to_string(),
                                expected: Some("i32 | bool".to_string()),
                                found: None,
                            });
                            IrType::I32
                        }
                    };

                    parsed_fields.push((f_name, f_type));
                }

                record_defs.insert(
                    name.clone(),
                    RecordDef {
                        name,
                        fields: parsed_fields,
                    },
                );
            }
        }
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
            message: "'functions' array must contain at least one function.".to_string(),
            expected: Some("non-empty array".to_string()),
            found: Some("[]".to_string()),
        });
        return diags;
    }

    let mut function_signatures: HashMap<String, FuncSignature> = HashMap::new();
    let mut func_names = HashSet::new();

    for (f_idx, func_val) in functions.iter().enumerate() {
        let f_path = format!("$.functions[{f_idx}]");
        let Some(func_obj) = func_val.as_object() else {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: None,
                path: f_path,
                message: "Function entry must be a JSON object.".to_string(),
                expected: Some("object".to_string()),
                found: Some(value_kind(func_val).to_string()),
            });
            continue;
        };

        let name = match func_obj.get("name").and_then(|v| v.as_str()) {
            Some(n) if is_valid_identifier(n) => n.to_string(),
            Some(n) => {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: None,
                    path: format!("{f_path}.name"),
                    message: format!("Function name '{n}' is not a valid identifier."),
                    expected: Some("^[A-Za-z_][A-Za-z0-9_]*$".to_string()),
                    found: Some(n.to_string()),
                });
                format!("$anon_{f_idx}")
            }
            None => {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: None,
                    path: format!("{f_path}.name"),
                    message: "Missing string property 'name' in function.".to_string(),
                    expected: Some("identifier string".to_string()),
                    found: None,
                });
                format!("$anon_{f_idx}")
            }
        };

        if func_names.contains(&name) {
            diags.push(Diagnostic {
                code: "E_DUPLICATE_FUNCTION".to_string(),
                function: Some(name.clone()),
                path: format!("{f_path}.name"),
                message: format!("Duplicate function name '{name}'."),
                expected: Some("unique function name".to_string()),
                found: Some(name.clone()),
            });
        }
        func_names.insert(name.clone());

        let is_export = func_obj
            .get("export")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let returns_ty = match func_obj.get("returns").and_then(|v| v.as_str()) {
            Some(s) => match IrType::from_str(s, &known_record_names) {
                Some(ty) => ty,
                None => {
                    diags.push(Diagnostic {
                        code: "E_UNKNOWN_TYPE".to_string(),
                        function: Some(name.clone()),
                        path: format!("{f_path}.returns"),
                        message: format!("Unknown return type '{s}' in function '{name}'."),
                        expected: Some("i32 | bool | declared record type".to_string()),
                        found: Some(s.to_string()),
                    });
                    IrType::I32
                }
            },
            None => {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: Some(name.clone()),
                    path: format!("{f_path}.returns"),
                    message: "Missing 'returns' property in function.".to_string(),
                    expected: Some("type string".to_string()),
                    found: None,
                });
                IrType::I32
            }
        };

        let mut parsed_params = Vec::new();
        let mut param_names = HashSet::new();

        if let Some(params_arr) = func_obj.get("params").and_then(|v| v.as_array()) {
            for (p_idx, p_val) in params_arr.iter().enumerate() {
                let p_path = format!("{f_path}.params[{p_idx}]");
                let Some(p_obj) = p_val.as_object() else {
                    continue;
                };
                let p_name = match p_obj.get("name").and_then(|v| v.as_str()) {
                    Some(pn) if is_valid_identifier(pn) => pn.to_string(),
                    _ => continue,
                };
                if param_names.contains(&p_name) {
                    diags.push(Diagnostic {
                        code: "E_DUPLICATE_PARAM".to_string(),
                        function: Some(name.clone()),
                        path: format!("{p_path}.name"),
                        message: format!(
                            "Duplicate parameter '{p_name}' in function '{name}'."
                        ),
                        expected: Some("unique parameter name".to_string()),
                        found: Some(p_name.clone()),
                    });
                }
                param_names.insert(p_name.clone());

                let p_type = match p_obj.get("type").and_then(|v| v.as_str()) {
                    Some(s) => IrType::from_str(s, &known_record_names).unwrap_or(IrType::I32),
                    None => IrType::I32,
                };
                parsed_params.push((p_name, p_type));
            }
        }

        let wit_interface = func_obj
            .get("wit_interface")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if ir_version == "0.4" && is_export {
            if let Some(ref wit_c) = wit_contract_def {
                match &wit_interface {
                    Some(iface) if iface == &wit_c.export_interface => {}
                    Some(iface) => {
                        diags.push(Diagnostic {
                            code: "E_UNKNOWN_EXPORT_INTERFACE".to_string(),
                            function: Some(name.clone()),
                            path: format!("{f_path}.wit_interface"),
                            message: format!(
                                "Exported function '{name}' specifies wit_interface '{iface}', but wit_contract exports '{}'.",
                                wit_c.export_interface
                            ),
                            expected: Some(wit_c.export_interface.clone()),
                            found: Some(iface.clone()),
                        });
                    }
                    None => {
                        diags.push(Diagnostic {
                            code: "E_UNKNOWN_EXPORT_INTERFACE".to_string(),
                            function: Some(name.clone()),
                            path: format!("{f_path}.wit_interface"),
                            message: format!(
                                "Exported function '{name}' in IR v0.4 must declare wit_interface matching '{}'.",
                                wit_c.export_interface
                            ),
                            expected: Some(wit_c.export_interface.clone()),
                            found: None,
                        });
                    }
                }
            }
        }

        function_signatures.insert(
            name.clone(),
            FuncSignature {
                name,
                export: is_export,
                wit_interface,
                params: parsed_params,
                returns: returns_ty,
                index: f_idx as u32,
            },
        );
    }

    let mut capabilities = HashSet::new();
    if let Some(caps_val) = obj.get("capabilities") {
        if let Some(caps_arr) = caps_val.as_array() {
            for (c_idx, c_val) in caps_arr.iter().enumerate() {
                if let Some(s) = c_val.as_str() {
                    capabilities.insert(s.to_string());
                } else {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: None,
                        path: format!("$.capabilities[{c_idx}]"),
                        message: "Capability name must be a string.".to_string(),
                        expected: Some("string".to_string()),
                        found: Some(value_kind(c_val).to_string()),
                    });
                }
            }
        }
    }

    let mut imports: HashMap<String, ImportSignature> = HashMap::new();
    let mut import_order: Vec<String> = Vec::new();
    if let Some(imp_val) = obj.get("imports") {
        if let Some(imp_arr) = imp_val.as_array() {
            for (i_idx, item_val) in imp_arr.iter().enumerate() {
                let path = format!("$.imports[{i_idx}]");
                let Some(imp_obj) = item_val.as_object() else {
                    diags.push(Diagnostic {
                        code: "E_SCHEMA_INVALID".to_string(),
                        function: None,
                        path,
                        message: "Import must be a JSON object.".to_string(),
                        expected: Some("object".to_string()),
                        found: Some(value_kind(item_val).to_string()),
                    });
                    continue;
                };

                let module = imp_obj.get("module").and_then(|v| v.as_str()).unwrap_or("");
                let name = imp_obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let capability = imp_obj.get("capability").and_then(|v| v.as_str()).unwrap_or("");

                if !capabilities.contains(capability) {
                    diags.push(Diagnostic {
                        code: "E_UNDECLARED_CAPABILITY".to_string(),
                        function: None,
                        path: format!("{path}.capability"),
                        message: format!(
                            "Import '{name}' requires capability '{capability}', which is not declared in root 'capabilities'."
                        ),
                        expected: Some("declared capability in root 'capabilities'".to_string()),
                        found: Some(capability.to_string()),
                    });
                }

                let mut parsed_params = Vec::new();
                if let Some(p_arr) = imp_obj.get("params").and_then(|v| v.as_array()) {
                    for (p_idx, p_val) in p_arr.iter().enumerate() {
                        let p_path = format!("{path}.params[{p_idx}]");
                        if let Some(p_obj) = p_val.as_object() {
                            let p_name = p_obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let p_ty_str = p_obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            let p_type = match IrType::from_str(p_ty_str, &known_record_names) {
                                Some(t) => t,
                                None => {
                                    diags.push(Diagnostic {
                                        code: "E_UNKNOWN_TYPE".to_string(),
                                        function: None,
                                        path: format!("{p_path}.type"),
                                        message: format!("Unknown type '{p_ty_str}' for import param '{p_name}'."),
                                        expected: Some("valid type".to_string()),
                                        found: Some(p_ty_str.to_string()),
                                    });
                                    IrType::I32
                                }
                            };
                            parsed_params.push((p_name.to_string(), p_type));
                        }
                    }
                }

                let ret_str = imp_obj.get("returns").and_then(|v| v.as_str()).unwrap_or("i32");
                let ret_type = match IrType::from_str(ret_str, &known_record_names) {
                    Some(t) => t,
                    None => {
                        diags.push(Diagnostic {
                            code: "E_UNKNOWN_TYPE".to_string(),
                            function: None,
                            path: format!("{path}.returns"),
                            message: format!("Unknown return type '{ret_str}' for import '{name}'."),
                            expected: Some("valid type".to_string()),
                            found: Some(ret_str.to_string()),
                        });
                        IrType::I32
                    }
                };

                let wit_interface = imp_obj
                    .get("wit_interface")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                imports.insert(name.to_string(), ImportSignature {
                    module: module.to_string(),
                    name: name.to_string(),
                    capability: capability.to_string(),
                    wit_interface,
                    params: parsed_params,
                    returns: ret_type,
                    index: i_idx as u32,
                });
                import_order.push(name.to_string());
            }
        }
    }

    let num_imports = import_order.len() as u32;

    let ctx = Context {
        ir_version: ir_version.to_string(),
        wit_contract: wit_contract_def,
        capabilities,
        imports,
        import_order,
        records: record_defs,
        functions: function_signatures,
        num_imports,
    };

    // Second pass: typecheck function bodies
    for (f_idx, func_val) in functions.iter().enumerate() {
        let f_path = format!("$.functions[{f_idx}]");
        let Some(func_obj) = func_val.as_object() else {
            continue;
        };
        let Some(name) = func_obj.get("name").and_then(|v| v.as_str()) else {
            continue;
        };
        let sig = &ctx.functions[name];

        let reqs = func_obj.get("requirements").and_then(|v| v.as_array());
        if sig.export && (reqs.is_none() || reqs.unwrap().is_empty()) {
            diags.push(Diagnostic {
                code: "E_MISSING_REQUIREMENT".to_string(),
                function: Some(name.to_string()),
                path: format!("{f_path}.requirements"),
                message: format!(
                    "Exported function '{name}' must declare at least one non-empty requirement ID."
                ),
                expected: Some("non-empty array of requirement strings".to_string()),
                found: reqs.map(|_| "[]".to_string()),
            });
        }

        let mut param_map: HashMap<String, IrType> = HashMap::new();
        for (p_name, p_type) in &sig.params {
            param_map.insert(p_name.clone(), p_type.clone());
        }

        let Some(body_val) = func_obj.get("body") else {
            diags.push(Diagnostic {
                code: "E_SCHEMA_INVALID".to_string(),
                function: Some(name.to_string()),
                path: format!("{f_path}.body"),
                message: format!("Function '{name}' is missing 'body'."),
                expected: Some("expression object".to_string()),
                found: None,
            });
            continue;
        };

        if let Some(body_ty) = check_expr(
            body_val,
            &format!("{f_path}.body"),
            Some(name),
            &param_map,
            &ctx,
            &mut diags,
        ) {
            if body_ty != sig.returns {
                diags.push(Diagnostic {
                    code: "E_RETURN_TYPE".to_string(),
                    function: Some(name.to_string()),
                    path: format!("{f_path}.body"),
                    message: format!(
                        "Function '{name}' body produces type '{}', but declares returns '{}'.",
                        body_ty.as_str(),
                        sig.returns.as_str()
                    ),
                    expected: Some(sig.returns.as_str().to_string()),
                    found: Some(body_ty.as_str().to_string()),
                });
            }
        }
    }

    diags
}

fn check_expr(
    expr: &Value,
    path: &str,
    func_name: Option<&str>,
    params: &HashMap<String, IrType>,
    ctx: &Context,
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
            let type_str = obj.get("type").and_then(|v| v.as_str());
            let ir_ty = match type_str {
                Some("i32") => IrType::I32,
                Some("bool") => IrType::Bool,
                _ => {
                    diags.push(Diagnostic {
                        code: "E_CONST_TYPE".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.type"),
                        message: "Const expression 'type' must be 'i32' or 'bool'.".to_string(),
                        expected: Some("i32 | bool".to_string()),
                        found: obj.get("type").map(ToString::to_string),
                    });
                    return None;
                }
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

            match (&ir_ty, val) {
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
                Some(ty.clone())
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
                ctx,
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
                ctx,
                diags,
            );
            let right_ty = check_expr(
                right_val,
                &format!("{path}.right"),
                func_name,
                params,
                ctx,
                diags,
            );

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
                    if (lt == IrType::I32 && rt == IrType::I32)
                        || (lt == IrType::Bool && rt == IrType::Bool)
                    {
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
                ctx,
                diags,
            );
            let then_ty = check_expr(
                then_val.unwrap(),
                &format!("{path}.then"),
                func_name,
                params,
                ctx,
                diags,
            );
            let else_ty = check_expr(
                else_val.unwrap(),
                &format!("{path}.else"),
                func_name,
                params,
                ctx,
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
        "call" => {
            let Some(callee_name) = obj.get("function").and_then(|v| v.as_str()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.function"),
                    message: "'call' expression requires a string 'function' property.".to_string(),
                    expected: Some("function name string".to_string()),
                    found: None,
                });
                return None;
            };

            let Some(sig) = ctx.functions.get(callee_name) else {
                diags.push(Diagnostic {
                    code: "E_UNKNOWN_FUNCTION".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.function"),
                    message: format!("Unknown function '{callee_name}' in 'call' expression."),
                    expected: Some("declared function name".to_string()),
                    found: Some(callee_name.to_string()),
                });
                return None;
            };

            let Some(args_arr) = obj.get("args").and_then(|v| v.as_array()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.args"),
                    message: "'call' expression requires an 'args' array.".to_string(),
                    expected: Some("array".to_string()),
                    found: None,
                });
                return None;
            };

            if args_arr.len() != sig.params.len() {
                diags.push(Diagnostic {
                    code: "E_CALL_ARITY".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.args"),
                    message: format!(
                        "Function '{callee_name}' expects {} argument(s), but {} were provided.",
                        sig.params.len(),
                        args_arr.len()
                    ),
                    expected: Some(format!("{} arguments", sig.params.len())),
                    found: Some(format!("{} arguments", args_arr.len())),
                });
                return None;
            }

            let mut match_ok = true;
            for (arg_idx, arg_val) in args_arr.iter().enumerate() {
                let arg_path = format!("{path}.args[{arg_idx}]");
                let expected_ty = &sig.params[arg_idx].1;
                let actual_ty = check_expr(arg_val, &arg_path, func_name, params, ctx, diags);
                if let Some(act) = actual_ty {
                    if act != *expected_ty {
                        diags.push(Diagnostic {
                            code: "E_TYPE_MISMATCH".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: arg_path,
                            message: format!(
                                "Argument {} for function '{callee_name}' expects type '{}', found '{}'.",
                                arg_idx,
                                expected_ty.as_str(),
                                act.as_str()
                            ),
                            expected: Some(expected_ty.as_str().to_string()),
                            found: Some(act.as_str().to_string()),
                        });
                        match_ok = false;
                    }
                } else {
                    match_ok = false;
                }
            }

            if match_ok {
                Some(sig.returns.clone())
            } else {
                None
            }
        }
        "call_import" => {
            let Some(import_name) = obj.get("import").and_then(|v| v.as_str()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.import"),
                    message: "'call_import' expression requires a string 'import' property.".to_string(),
                    expected: Some("import name string".to_string()),
                    found: None,
                });
                return None;
            };

            let Some(sig) = ctx.imports.get(import_name) else {
                diags.push(Diagnostic {
                    code: "E_UNKNOWN_IMPORT".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.import"),
                    message: format!("Unknown import '{import_name}' in 'call_import' expression."),
                    expected: Some("declared import name".to_string()),
                    found: Some(import_name.to_string()),
                });
                return None;
            };

            if !ctx.capabilities.contains(&sig.capability) {
                diags.push(Diagnostic {
                    code: "E_UNDECLARED_CAPABILITY".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.import"),
                    message: format!(
                        "Invocation of import '{import_name}' requires capability '{}', which is not declared in root 'capabilities'.",
                        sig.capability
                    ),
                    expected: Some(format!("capability '{}' in root 'capabilities'", sig.capability)),
                    found: None,
                });
            }

            let Some(args_arr) = obj.get("args").and_then(|v| v.as_array()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.args"),
                    message: "'call_import' expression requires an 'args' array.".to_string(),
                    expected: Some("array".to_string()),
                    found: None,
                });
                return None;
            };

            if args_arr.len() != sig.params.len() {
                diags.push(Diagnostic {
                    code: "E_CALL_ARITY".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.args"),
                    message: format!(
                        "Import '{import_name}' expects {} argument(s), but {} were provided.",
                        sig.params.len(),
                        args_arr.len()
                    ),
                    expected: Some(format!("{} arguments", sig.params.len())),
                    found: Some(format!("{} arguments", args_arr.len())),
                });
                return None;
            }

            let mut match_ok = true;
            for (arg_idx, arg_val) in args_arr.iter().enumerate() {
                let arg_path = format!("{path}.args[{arg_idx}]");
                let expected_ty = &sig.params[arg_idx].1;
                let actual_ty = check_expr(arg_val, &arg_path, func_name, params, ctx, diags);
                if let Some(act) = actual_ty {
                    if act != *expected_ty {
                        diags.push(Diagnostic {
                            code: "E_TYPE_MISMATCH".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: arg_path,
                            message: format!(
                                "Argument {} for import '{import_name}' expects type '{}', found '{}'.",
                                arg_idx,
                                expected_ty.as_str(),
                                act.as_str()
                            ),
                            expected: Some(expected_ty.as_str().to_string()),
                            found: Some(act.as_str().to_string()),
                        });
                        match_ok = false;
                    }
                } else {
                    match_ok = false;
                }
            }

            if match_ok {
                Some(sig.returns.clone())
            } else {
                None
            }
        }
        "record.construct" => {
            let Some(rec_type_name) = obj.get("type").and_then(|v| v.as_str()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.type"),
                    message: "'record.construct' requires a string 'type' property.".to_string(),
                    expected: Some("record type name".to_string()),
                    found: None,
                });
                return None;
            };

            let Some(rec_def) = ctx.records.get(rec_type_name) else {
                diags.push(Diagnostic {
                    code: "E_UNKNOWN_TYPE".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.type"),
                    message: format!("Unknown record type '{rec_type_name}' in 'record.construct'."),
                    expected: Some("declared record type".to_string()),
                    found: Some(rec_type_name.to_string()),
                });
                return None;
            };

            let Some(fields_map) = obj.get("fields").and_then(|v| v.as_object()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.fields"),
                    message: "'record.construct' requires a 'fields' object.".to_string(),
                    expected: Some("object".to_string()),
                    found: None,
                });
                return None;
            };

            let mut construct_ok = true;
            let declared_field_names: HashSet<&str> =
                rec_def.fields.iter().map(|(n, _)| n.as_str()).collect();

            // Check for unknown fields
            for key in fields_map.keys() {
                if !declared_field_names.contains(key.as_str()) {
                    diags.push(Diagnostic {
                        code: "E_UNKNOWN_FIELD".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.fields.{key}"),
                        message: format!(
                            "Unknown field '{key}' for record type '{rec_type_name}'."
                        ),
                        expected: Some(rec_def.fields.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(" | ")),
                        found: Some(key.clone()),
                    });
                    construct_ok = false;
                }
            }

            // Check missing fields and type check provided fields
            for (f_name, f_expected_ty) in &rec_def.fields {
                match fields_map.get(f_name) {
                    Some(f_expr) => {
                        let f_path = format!("{path}.fields.{f_name}");
                        let actual_ty = check_expr(f_expr, &f_path, func_name, params, ctx, diags);
                        if let Some(act) = actual_ty {
                            if act != *f_expected_ty {
                                diags.push(Diagnostic {
                                    code: "E_TYPE_MISMATCH".to_string(),
                                    function: func_name.map(ToOwned::to_owned),
                                    path: f_path,
                                    message: format!(
                                        "Field '{f_name}' in record '{rec_type_name}' expects type '{}', found '{}'.",
                                        f_expected_ty.as_str(),
                                        act.as_str()
                                    ),
                                    expected: Some(f_expected_ty.as_str().to_string()),
                                    found: Some(act.as_str().to_string()),
                                });
                                construct_ok = false;
                            }
                        } else {
                            construct_ok = false;
                        }
                    }
                    None => {
                        diags.push(Diagnostic {
                            code: "E_MISSING_FIELD".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: format!("{path}.fields.{f_name}"),
                            message: format!(
                                "Missing required field '{f_name}' in constructor for record '{rec_type_name}'."
                            ),
                            expected: Some(format!("{f_name}: {}", f_expected_ty.as_str())),
                            found: None,
                        });
                        construct_ok = false;
                    }
                }
            }

            if construct_ok {
                Some(IrType::Record(rec_type_name.to_string()))
            } else {
                None
            }
        }
        "record.get" => {
            let Some(target_val) = obj.get("target") else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.target"),
                    message: "'record.get' requires a 'target' expression.".to_string(),
                    expected: Some("record expression".to_string()),
                    found: None,
                });
                return None;
            };

            let Some(field_name) = obj.get("field").and_then(|v| v.as_str()) else {
                diags.push(Diagnostic {
                    code: "E_SCHEMA_INVALID".to_string(),
                    function: func_name.map(ToOwned::to_owned),
                    path: format!("{path}.field"),
                    message: "'record.get' requires a string 'field' property.".to_string(),
                    expected: Some("field name string".to_string()),
                    found: None,
                });
                return None;
            };

            let target_ty = check_expr(
                target_val,
                &format!("{path}.target"),
                func_name,
                params,
                ctx,
                diags,
            )?;

            match target_ty {
                IrType::Record(rec_name) => {
                    let Some(rec_def) = ctx.records.get(&rec_name) else {
                        diags.push(Diagnostic {
                            code: "E_UNKNOWN_TYPE".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: format!("{path}.target"),
                            message: format!("Unknown record type '{rec_name}'."),
                            expected: Some("valid record type".to_string()),
                            found: Some(rec_name),
                        });
                        return None;
                    };

                    if let Some((_, f_ty)) =
                        rec_def.fields.iter().find(|(name, _)| name == field_name)
                    {
                        Some(f_ty.clone())
                    } else {
                        let available: Vec<&str> =
                            rec_def.fields.iter().map(|(n, _)| n.as_str()).collect();
                        diags.push(Diagnostic {
                            code: "E_UNKNOWN_FIELD".to_string(),
                            function: func_name.map(ToOwned::to_owned),
                            path: format!("{path}.field"),
                            message: format!(
                                "Unknown field '{field_name}' on record type '{rec_name}'; available: {available:?}"
                            ),
                            expected: Some(available.join(" | ")),
                            found: Some(field_name.to_string()),
                        });
                        None
                    }
                }
                other => {
                    diags.push(Diagnostic {
                        code: "E_TYPE_MISMATCH".to_string(),
                        function: func_name.map(ToOwned::to_owned),
                        path: format!("{path}.target"),
                        message: format!(
                            "'record.get' target must be a record, found '{}'.",
                            other.as_str()
                        ),
                        expected: Some("record".to_string()),
                        found: Some(other.as_str().to_string()),
                    });
                    None
                }
            }
        }
        other => {
            diags.push(Diagnostic {
                code: "E_UNSUPPORTED_OP".to_string(),
                function: func_name.map(ToOwned::to_owned),
                path: format!("{path}.op"),
                message: format!("Unsupported operator '{other}' in Semantic IR."),
                expected: Some(
                    "const | param | not | add | sub | mul | div_s | eq | lt_s | gte_s | and | or | if | call | call_import | record.construct | record.get"
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

/// Compiles a verified Semantic IR v0.1 or v0.2 JSON document directly into deterministic WebAssembly bytes.
pub fn compile_ir_to_wasm(root: &Value) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let diags = verify_ir(root);
    if !diags.is_empty() {
        return Err(diags);
    }

    let ir_version = root
        .get("ir_version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.1");

    let mut record_defs: HashMap<String, RecordDef> = HashMap::new();
    if let Some(types_arr) = root.get("types").and_then(|v| v.as_array()) {
        for type_val in types_arr {
            let type_obj = type_val.as_object().unwrap();
            let name = type_obj["name"].as_str().unwrap().to_string();
            let mut fields = Vec::new();
            for f_val in type_obj["fields"].as_array().unwrap() {
                let f_obj = f_val.as_object().unwrap();
                let f_name = f_obj["name"].as_str().unwrap().to_string();
                let f_ty = match f_obj["type"].as_str().unwrap() {
                    "i32" => IrType::I32,
                    "bool" => IrType::Bool,
                    other => IrType::Record(other.to_string()),
                };
                fields.push((f_name, f_ty));
            }
            record_defs.insert(name.clone(), RecordDef { name, fields });
        }
    }

    let mut capabilities = HashSet::new();
    if let Some(caps_val) = root.get("capabilities") {
        if let Some(caps_arr) = caps_val.as_array() {
            for c_val in caps_arr {
                if let Some(s) = c_val.as_str() {
                    capabilities.insert(s.to_string());
                }
            }
        }
    }

    let mut import_signatures: HashMap<String, ImportSignature> = HashMap::new();
    let mut import_order: Vec<String> = Vec::new();
    if let Some(imp_val) = root.get("imports") {
        if let Some(imp_arr) = imp_val.as_array() {
            for (i_idx, item_val) in imp_arr.iter().enumerate() {
                let imp_obj = item_val.as_object().unwrap();
                let module = imp_obj["module"].as_str().unwrap().to_string();
                let name = imp_obj["name"].as_str().unwrap().to_string();
                let capability = imp_obj
                    .get("capability")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mut parsed_params = Vec::new();
                if let Some(p_arr) = imp_obj.get("params").and_then(|v| v.as_array()) {
                    for p_val in p_arr {
                        let p_obj = p_val.as_object().unwrap();
                        let p_name = p_obj["name"].as_str().unwrap().to_string();
                        let p_ty = match p_obj["type"].as_str().unwrap() {
                            "i32" => IrType::I32,
                            "bool" => IrType::Bool,
                            other => IrType::Record(other.to_string()),
                        };
                        parsed_params.push((p_name, p_ty));
                    }
                }
                let ret_ty = match imp_obj.get("returns").and_then(|v| v.as_str()).unwrap_or("i32")
                {
                    "i32" => IrType::I32,
                    "bool" => IrType::Bool,
                    other => IrType::Record(other.to_string()),
                };
                let wit_interface = imp_obj
                    .get("wit_interface")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                import_signatures.insert(
                    name.clone(),
                    ImportSignature {
                        module,
                        name: name.clone(),
                        capability,
                        wit_interface,
                        params: parsed_params,
                        returns: ret_ty,
                        index: i_idx as u32,
                    },
                );
                import_order.push(name);
            }
        }
    }
    let num_imports = import_order.len() as u32;

    let functions = root["functions"].as_array().expect("verified functions array");

    let mut function_signatures: HashMap<String, FuncSignature> = HashMap::new();
    for (f_idx, func_val) in functions.iter().enumerate() {
        let func_obj = func_val.as_object().unwrap();
        let name = func_obj["name"].as_str().unwrap().to_string();
        let is_export = func_obj.get("export").and_then(|v| v.as_bool()).unwrap_or(true);
        let wit_interface = func_obj
            .get("wit_interface")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let params_arr = func_obj["params"].as_array().unwrap();
        let mut params = Vec::new();
        for p_val in params_arr {
            let p_name = p_val["name"].as_str().unwrap().to_string();
            let p_ty = match p_val["type"].as_str().unwrap() {
                "i32" => IrType::I32,
                "bool" => IrType::Bool,
                other => IrType::Record(other.to_string()),
            };
            params.push((p_name, p_ty));
        }
        let returns = match func_obj["returns"].as_str().unwrap() {
            "i32" => IrType::I32,
            "bool" => IrType::Bool,
            other => IrType::Record(other.to_string()),
        };
        function_signatures.insert(
            name.clone(),
            FuncSignature {
                name,
                export: is_export,
                wit_interface,
                params,
                returns,
                index: num_imports + f_idx as u32,
            },
        );
    }

    let wit_contract_def = if ir_version == "0.4" {
        root.get("wit_contract").and_then(|v| v.as_object()).map(|obj| WitContractDef {
            package: obj.get("package").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            world: obj.get("world").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            export_interface: obj.get("export_interface").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            import_interfaces: obj
                .get("import_interfaces")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
        })
    } else {
        None
    };

    let ctx = Context {
        ir_version: ir_version.to_string(),
        wit_contract: wit_contract_def,
        capabilities,
        imports: import_signatures,
        import_order: import_order.clone(),
        records: record_defs,
        functions: function_signatures,
        num_imports,
    };

    let needs_memory = ir_version == "0.2" || ir_version == "0.3" || !ctx.records.is_empty();

    let mut module = Module::new();
    let mut types = TypeSection::new();
    let mut import_section = ImportSection::new();
    let mut func_section = FunctionSection::new();
    let mut exports = ExportSection::new();
    let mut code = CodeSection::new();

    for (imp_idx, imp_name) in import_order.iter().enumerate() {
        let imp_sig = &ctx.imports[imp_name];
        let wasm_params: Vec<ValType> = imp_sig.params.iter().map(|_| ValType::I32).collect();
        let results = [ValType::I32];
        let type_idx = imp_idx as u32;
        types.ty().function(wasm_params, results);
        import_section.import(&imp_sig.module, &imp_sig.name, EntityType::Function(type_idx));
    }

    for (func_idx, func_val) in functions.iter().enumerate() {
        let func_obj = func_val.as_object().unwrap();
        let name = func_obj["name"].as_str().unwrap();
        let is_export = func_obj.get("export").and_then(|v| v.as_bool()).unwrap_or(true);
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
        let func_type_idx = num_imports + func_idx as u32;
        types.ty().function(wasm_params, results);
        func_section.function(func_type_idx);

        if is_export {
            exports.export(name, ExportKind::Func, num_imports + func_idx as u32);
        }

        // Local 0..num_params-1: params
        // Local num_params: temporary rec_ptr (i32)
        let rec_ptr_local = params_arr.len() as u32;
        let mut wasm_func = Function::new([(1, ValType::I32)]);
        emit_expr(body, &param_map, rec_ptr_local, &ctx, &mut wasm_func);
        wasm_func.instruction(&Instruction::End);
        code.function(&wasm_func);
    }

    module.section(&types);
    if num_imports > 0 {
        module.section(&import_section);
    }
    module.section(&func_section);

    if needs_memory {
        let mut memory_section = MemorySection::new();
        memory_section.memory(MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
        module.section(&memory_section);

        let mut global_section = GlobalSection::new();
        global_section.global(
            GlobalType {
                val_type: ValType::I32,
                mutable: true,
                shared: false,
            },
            &ConstExpr::i32_const(1024),
        );
        module.section(&global_section);

        exports.export("memory", ExportKind::Memory, 0);
    }

    module.section(&exports);
    module.section(&code);

    if ir_version == "0.4" {
        if let Some(wit_val) = root.get("wit_contract") {
            let wit_json = serde_json::to_vec(wit_val).unwrap();
            let custom = CustomSection {
                name: "component-type:order-processor".into(),
                data: wit_json.into(),
            };
            module.section(&custom);
        }
    }

    let wasm_bytes = module.finish();
    Ok(wasm_bytes)
}

fn emit_expr(
    expr: &Value,
    params: &HashMap<String, u32>,
    rec_ptr_local: u32,
    ctx: &Context,
    func: &mut Function,
) {
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
            emit_expr(&obj["value"], params, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::I32Eqz);
        }
        "add" | "sub" | "mul" | "div_s" | "eq" | "lt_s" | "gte_s" | "and" | "or" => {
            emit_expr(&obj["left"], params, rec_ptr_local, ctx, func);
            emit_expr(&obj["right"], params, rec_ptr_local, ctx, func);
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
            emit_expr(&obj["condition"], params, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
            emit_expr(&obj["then"], params, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::Else);
            emit_expr(&obj["else"], params, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::End);
        }
        "call" => {
            let callee_name = obj["function"].as_str().unwrap();
            let callee_sig = &ctx.functions[callee_name];
            let args_arr = obj["args"].as_array().unwrap();
            for arg_val in args_arr {
                emit_expr(arg_val, params, rec_ptr_local, ctx, func);
            }
            func.instruction(&Instruction::Call(callee_sig.index));
        }
        "call_import" => {
            let import_name = obj["import"].as_str().unwrap();
            let imp_sig = &ctx.imports[import_name];
            let args_arr = obj["args"].as_array().unwrap();
            for arg_val in args_arr {
                emit_expr(arg_val, params, rec_ptr_local, ctx, func);
            }
            func.instruction(&Instruction::Call(imp_sig.index));
        }
        "record.construct" => {
            let rec_type_name = obj["type"].as_str().unwrap();
            let rec_def = &ctx.records[rec_type_name];
            let fields_map = obj["fields"].as_object().unwrap();
            let total_size_bytes = (rec_def.fields.len() * 4) as i32;

            // 1. Get current heap pointer and store in rec_ptr_local
            func.instruction(&Instruction::GlobalGet(0));
            func.instruction(&Instruction::LocalSet(rec_ptr_local));

            // 2. Bump global heap pointer
            func.instruction(&Instruction::GlobalGet(0));
            func.instruction(&Instruction::I32Const(total_size_bytes));
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::GlobalSet(0));

            // 3. For each field, store at (rec_ptr_local + offset)
            for (f_idx, (f_name, _)) in rec_def.fields.iter().enumerate() {
                let f_expr = &fields_map[f_name];
                let byte_offset = (f_idx * 4) as u64;

                func.instruction(&Instruction::LocalGet(rec_ptr_local));
                emit_expr(f_expr, params, rec_ptr_local, ctx, func);
                func.instruction(&Instruction::I32Store(MemArg {
                    offset: byte_offset,
                    align: 2,
                    memory_index: 0,
                }));
            }

            // 4. Leave rec_ptr_local on stack
            func.instruction(&Instruction::LocalGet(rec_ptr_local));
        }
        "record.get" => {
            let target_expr = &obj["target"];
            let field_name = obj["field"].as_str().unwrap();

            // Find target record type
            let rec_type_name = match infer_expr_type(target_expr, params, ctx) {
                Some(IrType::Record(n)) => n,
                _ => panic!("Expected record target in record.get"),
            };
            let rec_def = &ctx.records[&rec_type_name];
            let (f_idx, _) = rec_def
                .fields
                .iter()
                .enumerate()
                .find(|(_, (n, _))| n == field_name)
                .expect("field in record definition");

            emit_expr(target_expr, params, rec_ptr_local, ctx, func);
            func.instruction(&Instruction::I32Load(MemArg {
                offset: (f_idx * 4) as u64,
                align: 2,
                memory_index: 0,
            }));
        }
        _ => unreachable!(),
    }
}

fn infer_expr_type(expr: &Value, params: &HashMap<String, u32>, ctx: &Context) -> Option<IrType> {
    let obj = expr.as_object()?;
    let op = obj.get("op")?.as_str()?;
    match op {
        "record.construct" => {
            let rec_name = obj.get("type")?.as_str()?;
            Some(IrType::Record(rec_name.to_string()))
        }
        "param" => {
            let name = obj.get("name")?.as_str()?;
            let _local_idx = params.get(name)?;
            for sig in ctx.functions.values() {
                if let Some((_, ty)) = sig.params.iter().find(|(n, _)| n == name) {
                    return Some(ty.clone());
                }
            }
            None
        }
        "call" => {
            let func_name = obj.get("function")?.as_str()?;
            let sig = ctx.functions.get(func_name)?;
            Some(sig.returns.clone())
        }
        "call_import" => {
            let import_name = obj.get("import")?.as_str()?;
            let sig = ctx.imports.get(import_name)?;
            Some(sig.returns.clone())
        }
        "if" => infer_expr_type(obj.get("then")?, params, ctx),
        _ => None,
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
        assert!(diags.is_empty(), "smoke IR must have 0 diagnostics: {diags:?}");

        let wasm1 = compile_ir_to_wasm(&ir).expect("compile must succeed");
        let wasm2 = compile_ir_to_wasm(&ir).expect("compile must succeed");
        assert_eq!(sha256_hex(&wasm1), sha256_hex(&wasm2));
        assert!(validate_wasm_bytes(&wasm1).is_ok());
    }

    #[test]
    fn valid_v0_2_smoke_ir_passes_and_compiles() {
        let ir = json!({
            "ir_version": "0.2",
            "spec_version": 1,
            "types": [
                {
                    "name": "Point",
                    "fields": [
                        { "name": "x", "type": "i32" },
                        { "name": "y", "type": "i32" }
                    ]
                },
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

        let diags = verify_ir(&ir);
        assert!(diags.is_empty(), "v0.2 smoke IR must have 0 diagnostics: {diags:?}");

        let wasm = compile_ir_to_wasm(&ir).expect("compile must succeed");
        assert!(validate_wasm_bytes(&wasm).is_ok());
    }

    #[test]
    fn detects_v0_2_semantic_error_codes() {
        // 1. E_UNKNOWN_FUNCTION in call
        let unknown_func = json!({
            "ir_version": "0.2",
            "spec_version": 1,
            "functions": [{
                "name": "caller",
                "export": true,
                "params": [],
                "returns": "i32",
                "requirements": ["R1"],
                "body": { "op": "call", "function": "nonexistent", "args": [] }
            }]
        });
        assert!(verify_ir(&unknown_func).iter().any(|d| d.code == "E_UNKNOWN_FUNCTION"));

        // 2. E_CALL_ARITY
        let arity_err = json!({
            "ir_version": "0.2",
            "spec_version": 1,
            "functions": [
                {
                    "name": "target",
                    "export": false,
                    "params": [{ "name": "a", "type": "i32" }],
                    "returns": "i32",
                    "requirements": [],
                    "body": { "op": "param", "name": "a" }
                },
                {
                    "name": "caller",
                    "export": true,
                    "params": [],
                    "returns": "i32",
                    "requirements": ["R1"],
                    "body": { "op": "call", "function": "target", "args": [] }
                }
            ]
        });
        assert!(verify_ir(&arity_err).iter().any(|d| d.code == "E_CALL_ARITY"));

        // 3. E_UNKNOWN_FIELD in record.construct
        let unknown_field = json!({
            "ir_version": "0.2",
            "spec_version": 1,
            "types": [{ "name": "Item", "fields": [{ "name": "price", "type": "i32" }] }],
            "functions": [{
                "name": "test",
                "export": true,
                "params": [],
                "returns": "Item",
                "requirements": ["R1"],
                "body": {
                    "op": "record.construct",
                    "type": "Item",
                    "fields": { "price": { "op": "const", "type": "i32", "value": 10 }, "bogus": { "op": "const", "type": "i32", "value": 1 } }
                }
            }]
        });
        assert!(verify_ir(&unknown_field).iter().any(|d| d.code == "E_UNKNOWN_FIELD"));

        // 4. E_MISSING_FIELD in record.construct
        let missing_field = json!({
            "ir_version": "0.2",
            "spec_version": 1,
            "types": [{ "name": "Item", "fields": [{ "name": "price", "type": "i32" }, { "name": "qty", "type": "i32" }] }],
            "functions": [{
                "name": "test",
                "export": true,
                "params": [],
                "returns": "Item",
                "requirements": ["R1"],
                "body": {
                    "op": "record.construct",
                    "type": "Item",
                    "fields": { "price": { "op": "const", "type": "i32", "value": 10 } }
                }
            }]
        });
        assert!(verify_ir(&missing_field).iter().any(|d| d.code == "E_MISSING_FIELD"));
    }

    #[test]
    fn validates_and_compiles_v0_3_smoke_module() {
        let smoke_json_str = include_str!("../../../ir/smoke-v3.ir.json");
        let ir: Value = serde_json::from_str(smoke_json_str).expect("smoke-v3 must be valid JSON");

        let diags = verify_ir(&ir);
        assert!(diags.is_empty(), "smoke-v3 IR must have 0 diagnostics: {diags:?}");

        let wasm = compile_ir_to_wasm(&ir).expect("smoke-v3 compilation must succeed");
        assert!(validate_wasm_bytes(&wasm).is_ok(), "smoke-v3 WASM must pass wasmparser validation");
    }

    #[test]
    fn detects_v0_3_capability_and_import_diagnostics() {
        // 1. E_UNDECLARED_CAPABILITY at import declaration
        let undeclared_cap_import = json!({
            "ir_version": "0.3",
            "spec_version": 1,
            "capabilities": ["kv_store"],
            "imports": [
                {
                    "module": "host",
                    "name": "emit_audit",
                    "capability": "audit_log",
                    "params": [{ "name": "event_id", "type": "i32" }],
                    "returns": "i32"
                }
            ],
            "functions": [{
                "name": "noop",
                "export": true,
                "params": [],
                "returns": "i32",
                "requirements": ["R1"],
                "body": { "op": "const", "type": "i32", "value": 0 }
            }]
        });
        let diags1 = verify_ir(&undeclared_cap_import);
        assert!(diags1.iter().any(|d| d.code == "E_UNDECLARED_CAPABILITY"), "diags: {diags1:?}");

        // 2. E_UNKNOWN_IMPORT in call_import
        let unknown_import_call = json!({
            "ir_version": "0.3",
            "spec_version": 1,
            "capabilities": ["kv_store"],
            "imports": [
                {
                    "module": "host",
                    "name": "kv_get",
                    "capability": "kv_store",
                    "params": [{ "name": "key", "type": "i32" }],
                    "returns": "i32"
                }
            ],
            "functions": [{
                "name": "test",
                "export": true,
                "params": [],
                "returns": "i32",
                "requirements": ["R1"],
                "body": {
                    "op": "call_import",
                    "import": "nonexistent_import",
                    "args": [{ "op": "const", "type": "i32", "value": 1 }]
                }
            }]
        });
        let diags2 = verify_ir(&unknown_import_call);
        assert!(diags2.iter().any(|d| d.code == "E_UNKNOWN_IMPORT"), "diags: {diags2:?}");

        // 3. E_CALL_ARITY in call_import
        let arity_err_call = json!({
            "ir_version": "0.3",
            "spec_version": 1,
            "capabilities": ["kv_store"],
            "imports": [
                {
                    "module": "host",
                    "name": "kv_get",
                    "capability": "kv_store",
                    "params": [{ "name": "key", "type": "i32" }],
                    "returns": "i32"
                }
            ],
            "functions": [{
                "name": "test",
                "export": true,
                "params": [],
                "returns": "i32",
                "requirements": ["R1"],
                "body": {
                    "op": "call_import",
                    "import": "kv_get",
                    "args": []
                }
            }]
        });
        let diags3 = verify_ir(&arity_err_call);
        assert!(diags3.iter().any(|d| d.code == "E_CALL_ARITY"), "diags: {diags3:?}");

        // 4. E_TYPE_MISMATCH in call_import
        let type_mismatch_call = json!({
            "ir_version": "0.3",
            "spec_version": 1,
            "capabilities": ["kv_store"],
            "imports": [
                {
                    "module": "host",
                    "name": "kv_get",
                    "capability": "kv_store",
                    "params": [{ "name": "key", "type": "i32" }],
                    "returns": "i32"
                }
            ],
            "functions": [{
                "name": "test",
                "export": true,
                "params": [],
                "returns": "i32",
                "requirements": ["R1"],
                "body": {
                    "op": "call_import",
                    "import": "kv_get",
                    "args": [{ "op": "const", "type": "bool", "value": true }]
                }
            }]
        });
        let diags4 = verify_ir(&type_mismatch_call);
        assert!(diags4.iter().any(|d| d.code == "E_TYPE_MISMATCH"), "diags: {diags4:?}");
    }

    #[test]
    fn validates_and_compiles_v0_4_smoke_module() {
        let smoke_json = include_str!("../../../ir/smoke-v4.ir.json");
        let val: Value = serde_json::from_str(smoke_json).expect("valid JSON");
        let diags = verify_ir(&val);
        assert!(diags.is_empty(), "diags on v0.4 smoke: {diags:?}");

        let wasm = compile_ir_to_wasm(&val).expect("compile v0.4 to wasm");
        assert!(!wasm.is_empty());

        let mut validator = wasmparser::Validator::new();
        validator.validate_all(&wasm).expect("valid wasm module with component metadata");

        let parser = wasmparser::Parser::new(0);
        let mut found_custom = false;
        for payload in parser.parse_all(&wasm) {
            if let Ok(wasmparser::Payload::CustomSection(cs)) = payload {
                if cs.name() == "component-type:order-processor" {
                    found_custom = true;
                    break;
                }
            }
        }
        assert!(found_custom, "Custom section component-type:order-processor must be present in v0.4");
    }

    #[test]
    fn detects_v0_4_wit_contract_mismatch() {
        let smoke_json = include_str!("../../../ir/smoke-v4.ir.json");
        let mut val: Value = serde_json::from_str(smoke_json).expect("valid JSON");
        val["functions"][0]["wit_interface"] = json!("nonexistent-interface");
        let diags = verify_ir(&val);
        assert!(
            diags.iter().any(|d| d.code == "E_UNKNOWN_EXPORT_INTERFACE"),
            "diags: {diags:?}"
        );
    }
}
