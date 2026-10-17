use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;
use wasmtime::{Caller, Engine, Linker, Module, Store, Val};

#[derive(Parser)]
#[command(
    name = "runner",
    version,
    about = "Deterministic Wasmtime test runner for Kaddo Agent-Native POC"
)]
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
        #[arg(long, alias = "suite")]
        cases: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        link: Option<PathBuf>,
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
    #[serde(default)]
    host_state: Option<YamlValue>,
    #[serde(default)]
    expected_host_calls: Option<YamlValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct HostCallRecord {
    import: String,
    args: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
struct ParsedHostState {
    kv: HashMap<i32, i32>,
    credit_limits: HashMap<i32, i32>,
    credit_configured: bool,
    credit_ok: Option<bool>,
    tenant_status: HashMap<i32, i32>,
    tenant_status_configured: bool,
    custom_maps: HashMap<String, HashMap<i32, i32>>,
    custom_scalars: HashMap<String, i32>,
    unconfigured_capabilities: std::collections::HashSet<String>,
    failing_capabilities: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
struct HostStoreData {
    kv: HashMap<i32, i32>,
    credit_limits: HashMap<i32, i32>,
    credit_configured: bool,
    credit_ok: Option<bool>,
    tenant_status: HashMap<i32, i32>,
    tenant_status_configured: bool,
    custom_maps: HashMap<String, HashMap<i32, i32>>,
    custom_scalars: HashMap<String, i32>,
    unconfigured_capabilities: std::collections::HashSet<String>,
    failing_capabilities: HashMap<String, String>,
    all_calls: Vec<HostCallRecord>,
    linked_wasm_bytes: Option<Vec<u8>>,
    dependency_error: Option<String>,
}

impl HostStoreData {
    fn clone_for_child(&self) -> Self {
        Self {
            kv: self.kv.clone(),
            credit_limits: self.credit_limits.clone(),
            credit_configured: self.credit_configured,
            credit_ok: self.credit_ok,
            tenant_status: self.tenant_status.clone(),
            tenant_status_configured: self.tenant_status_configured,
            custom_maps: self.custom_maps.clone(),
            custom_scalars: self.custom_scalars.clone(),
            unconfigured_capabilities: self.unconfigured_capabilities.clone(),
            failing_capabilities: self.failing_capabilities.clone(),
            all_calls: Vec::new(),
            linked_wasm_bytes: None,
            dependency_error: None,
        }
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_host_calls: Option<Vec<HostCallRecord>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_host_calls: Option<Vec<HostCallRecord>>,
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

fn parse_i32_map(yaml_map: &serde_yaml::Mapping) -> HashMap<i32, i32> {
    let mut out = HashMap::new();
    for (k, v) in yaml_map {
        let key_i32 = match k {
            YamlValue::Number(n) => n.as_i64().and_then(|i| i32::try_from(i).ok()),
            YamlValue::String(s) => s.parse::<i32>().ok(),
            _ => None,
        };
        let val_i32 = match v {
            YamlValue::Number(n) => n.as_i64().and_then(|i| i32::try_from(i).ok()),
            YamlValue::Bool(b) => Some(if *b { 1 } else { 0 }),
            YamlValue::String(s) => s.parse::<i32>().ok(),
            _ => None,
        };
        if let (Some(k), Some(v)) = (key_i32, val_i32) {
            out.insert(k, v);
        }
    }
    out
}

fn register_custom_entry(state: &mut ParsedHostState, key: String, v: &YamlValue) {
    if v.is_null() {
        state.unconfigured_capabilities.insert(key);
    } else if let Some(s) = v.as_str() {
        if let Some(err_reason) = s.strip_prefix("error:") {
            state
                .failing_capabilities
                .insert(key, err_reason.trim().to_string());
        } else if let Ok(scalar) = s.parse::<i32>() {
            state.custom_scalars.insert(key, scalar);
        }
    } else if let Some(m) = v.as_mapping() {
        if let Some(err_val) = m
            .get(YamlValue::String("error".to_string()))
            .and_then(|ev| ev.as_str())
        {
            state.failing_capabilities.insert(key, err_val.to_string());
        } else if m.is_empty() {
            state.unconfigured_capabilities.insert(key);
        } else {
            let parsed_map = parse_i32_map(m);
            if !parsed_map.is_empty() {
                state.custom_maps.insert(key, parsed_map);
            } else {
                // Nested interface mapping: e.g. `custom_iface: { op_name: 42 }`
                for (sub_k, sub_v) in m {
                    if let Some(sub_name) = sub_k.as_str() {
                        let norm_sub = sub_name.replace('-', "_");
                        register_custom_entry(state, format!("{key}/{norm_sub}"), sub_v);
                    }
                }
            }
        }
    } else if let Ok(scalar) = yaml_val_to_i32(v) {
        state.custom_scalars.insert(key, scalar);
    }
}

fn parse_host_state(val: Option<&YamlValue>) -> ParsedHostState {
    let mut state = ParsedHostState::default();
    let Some(val) = val else {
        return state;
    };
    if let Some(map) = val.as_mapping() {
        for (k, v) in map {
            let Some(key_name) = k.as_str() else {
                continue;
            };
            let norm_key = key_name.replace('-', "_");
            match norm_key.as_str() {
                "kv" => {
                    if let Some(m) = v.as_mapping() {
                        state.kv = parse_i32_map(m);
                    }
                }
                "credit_limits" => {
                    if let Some(m) = v.as_mapping() {
                        state.credit_limits = parse_i32_map(m);
                        state.credit_configured = true;
                    }
                }
                "credit_ok" => {
                    if let Some(b) = v.as_bool() {
                        state.credit_ok = Some(b);
                        state.credit_configured = true;
                    }
                }
                "tenant_status" => {
                    if let Some(m) = v.as_mapping() {
                        state.tenant_status = parse_i32_map(m);
                        state.tenant_status_configured = !state.tenant_status.is_empty();
                    }
                }
                other => {
                    // Preserve `/` or `#` if qualified by interface, while normalizing `-` to `_` on operation segment
                    let reg_key = if let Some((iface_part, op_part)) = key_name.split_once('/') {
                        format!("{}/{}", iface_part, op_part.replace('-', "_"))
                    } else if let Some((iface_part, op_part)) = key_name.split_once('#') {
                        format!("{}/{}", iface_part, op_part.replace('-', "_"))
                    } else {
                        other.to_string()
                    };
                    register_custom_entry(&mut state, reg_key, v);
                }
            }
        }
    }
    state
}

fn short_wit_interface_name(full_iface: &str) -> &str {
    let after_slash = full_iface
        .rsplit_once('/')
        .map(|(_, s)| s)
        .unwrap_or(full_iface);
    after_slash
        .split_once('@')
        .map(|(s, _)| s)
        .unwrap_or(after_slash)
}

fn candidate_capability_keys(full_iface: &str, op_kebab: &str, op_snake: &str) -> Vec<String> {
    let short_iface = short_wit_interface_name(full_iface);
    let norm_short_iface = short_iface.replace('-', "_");
    let mut keys = vec![
        format!("{full_iface}/{op_snake}"),
        format!("{full_iface}/{op_kebab}"),
        format!("{short_iface}/{op_snake}"),
        format!("{short_iface}/{op_kebab}"),
        format!("{norm_short_iface}/{op_snake}"),
    ];
    if short_iface == "host-services" || short_iface == "host" {
        keys.push(op_snake.to_string());
        if op_kebab != op_snake {
            keys.push(op_kebab.to_string());
        }
    }
    keys
}

fn parse_expected_host_calls(val: Option<&YamlValue>) -> Option<Vec<HostCallRecord>> {
    let val = val?;
    let seq = val.as_sequence()?;
    let mut list = Vec::new();
    for item in seq {
        if let Some(map) = item.as_mapping() {
            let import_name = map
                .get(YamlValue::String("import".to_string()))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let args_vec = match map.get(YamlValue::String("args".to_string())) {
                Some(YamlValue::Sequence(s)) => s.iter().map(yaml_to_json_val).collect(),
                _ => Vec::new(),
            };
            list.push(HostCallRecord {
                import: import_name,
                args: args_vec,
            });
        }
    }
    Some(list)
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

fn component_val_to_json(
    val: &wasmtime::component::Val,
    expected_hint: &serde_json::Value,
) -> serde_json::Value {
    match val {
        wasmtime::component::Val::S32(v) => serde_json::json!(*v),
        wasmtime::component::Val::Bool(b) => {
            if expected_hint.is_number() {
                serde_json::json!(if *b { 1 } else { 0 })
            } else {
                serde_json::json!(*b)
            }
        }
        wasmtime::component::Val::Record(fields) => {
            let mut map = serde_json::Map::new();
            let exp_obj = expected_hint.as_object();
            for (k, v) in fields.iter() {
                let key_str: &str = k.as_ref();
                let snake_key = key_str.replace('-', "_");
                let chosen_key = if let Some(obj) = exp_obj {
                    if obj.contains_key(key_str) {
                        key_str.to_string()
                    } else {
                        snake_key
                    }
                } else {
                    snake_key
                };
                let field_hint = exp_obj
                    .and_then(|o| o.get(&chosen_key))
                    .unwrap_or(&serde_json::Value::Null);
                map.insert(chosen_key, component_val_to_json(v, field_hint));
            }
            serde_json::Value::Object(map)
        }
        other => serde_json::json!(format!("{other:?}")),
    }
}

fn find_exported_component_func(
    engine: &Engine,
    comp: &wasmtime::component::Component,
    instance: &wasmtime::component::Instance,
    store: &mut Store<HostStoreData>,
    target_fn: &str,
) -> Option<(
    wasmtime::component::Func,
    wasmtime::component::types::ComponentFunc,
)> {
    let target_kebab = target_fn.replace('_', "-");
    let target_snake = target_fn.replace('-', "_");
    let comp_ty = comp.component_type();

    for (export_name, item) in comp_ty.exports(engine) {
        match item {
            wasmtime::component::types::ComponentItem::ComponentFunc(func_ty) => {
                if export_name == target_fn
                    || export_name == target_kebab
                    || export_name == target_snake
                {
                    if let Some(f) = instance.get_func(&mut *store, export_name) {
                        return Some((f, func_ty));
                    }
                }
            }
            wasmtime::component::types::ComponentItem::ComponentInstance(inst_ty) => {
                let Some(iface_export_idx) = instance.get_export(&mut *store, None, export_name)
                else {
                    continue;
                };
                for (fn_name, sub_item) in inst_ty.exports(engine) {
                    if let wasmtime::component::types::ComponentItem::ComponentFunc(func_ty) =
                        sub_item
                    {
                        if fn_name == target_fn
                            || fn_name == target_kebab
                            || fn_name == target_snake
                        {
                            if let Some(fn_export_idx) =
                                instance.get_export(&mut *store, Some(&iface_export_idx), fn_name)
                            {
                                if let Some(f) = instance.get_func(&mut *store, fn_export_idx) {
                                    return Some((f, func_ty));
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn register_component_imports(
    engine: &Engine,
    comp: &wasmtime::component::Component,
    linker: &mut wasmtime::component::Linker<HostStoreData>,
) -> Result<(), String> {
    let comp_type = comp.component_type();
    for (import_name, item) in comp_type.imports(engine) {
        if let wasmtime::component::types::ComponentItem::ComponentInstance(inst_ty) = item {
            let mut linker_inst = linker.instance(import_name).map_err(|e| e.to_string())?;
            let short_iface = short_wit_interface_name(import_name);
            let is_host_service = short_iface == "host-services"
                || short_iface == "host"
                || short_iface.ends_with("-host-services");
            let iface_name_owned = import_name.to_string();

            for (func_name, export_item) in inst_ty.exports(engine) {
                if let wasmtime::component::types::ComponentItem::ComponentFunc(func_ty) =
                    export_item
                {
                    let fn_name_owned = func_name.to_string();
                    let norm_fn_name = fn_name_owned.replace('-', "_");
                    let iface_for_closure = iface_name_owned.clone();
                    let param_types: Vec<wasmtime::component::types::Type> =
                        func_ty.params().collect();
                    let ret_types: Vec<wasmtime::component::types::Type> =
                        func_ty.results().collect();

                    if is_host_service {
                        linker_inst
                            .func_new(func_name, move |mut store, params, results| {
                                // 1. Validate WIT signature types supported by host adapter
                                for p_ty in &param_types {
                                    if !matches!(
                                        p_ty,
                                        wasmtime::component::types::Type::S32
                                            | wasmtime::component::types::Type::Bool
                                    ) {
                                        let msg = format!(
                                            "[HostCapabilityError] interface={} operation={} kind=unsupported_capability_type: parameter type {:?} is not supported by host adapter",
                                            iface_for_closure, fn_name_owned, p_ty
                                        );
                                        store.data_mut().dependency_error = Some(msg.clone());
                                        return Err(wasmtime::Error::msg(msg));
                                    }
                                }
                                for r_ty in &ret_types {
                                    if !matches!(
                                        r_ty,
                                        wasmtime::component::types::Type::S32
                                            | wasmtime::component::types::Type::Bool
                                    ) {
                                        let msg = format!(
                                            "[HostCapabilityError] interface={} operation={} kind=unsupported_capability_type: return type {:?} is not supported by host adapter",
                                            iface_for_closure, fn_name_owned, r_ty
                                        );
                                        store.data_mut().dependency_error = Some(msg.clone());
                                        return Err(wasmtime::Error::msg(msg));
                                    }
                                }

                                let mut i32_params = Vec::with_capacity(params.len());
                                let mut json_params = Vec::with_capacity(params.len());
                                for p in params {
                                    match p {
                                        wasmtime::component::Val::S32(v) => {
                                            i32_params.push(*v);
                                            json_params.push(serde_json::json!(*v));
                                        }
                                        wasmtime::component::Val::Bool(b) => {
                                            let iv = if *b { 1 } else { 0 };
                                            i32_params.push(iv);
                                            json_params.push(serde_json::json!(iv));
                                        }
                                        other => {
                                            let msg = format!(
                                                "[HostCapabilityError] interface={} operation={} kind=unsupported_capability_type: runtime value {:?} is not supported",
                                                iface_for_closure, fn_name_owned, other
                                            );
                                            store.data_mut().dependency_error = Some(msg.clone());
                                            return Err(wasmtime::Error::msg(msg));
                                        }
                                    }
                                }

                                let short_iface_now = short_wit_interface_name(&iface_for_closure);
                                let is_default_host_iface =
                                    short_iface_now == "host-services" || short_iface_now == "host";
                                let candidates = candidate_capability_keys(
                                    &iface_for_closure,
                                    &fn_name_owned,
                                    &norm_fn_name,
                                );

                                // 2. Resolve explicitly registered custom or built-in capability
                                let resolved_i32: Result<i32, String> = {
                                    let data = store.data();
                                    if let Some(fail_reason) = candidates
                                        .iter()
                                        .find_map(|k| data.failing_capabilities.get(k))
                                    {
                                        Err(format!(
                                            "[HostCapabilityError] interface={} operation={} kind=capability_execution_failed: {}",
                                            iface_for_closure, fn_name_owned, fail_reason
                                        ))
                                    } else if candidates
                                        .iter()
                                        .any(|k| data.unconfigured_capabilities.contains(k))
                                    {
                                        Err(format!(
                                            "[HostCapabilityError] interface={} operation={} kind=unconfigured_capability: capability '{}' is registered without required configuration",
                                            iface_for_closure, fn_name_owned, fn_name_owned
                                        ))
                                    } else if let Some(&scalar) = candidates
                                        .iter()
                                        .find_map(|k| data.custom_scalars.get(k))
                                    {
                                        Ok(scalar)
                                    } else if let Some(map) = candidates
                                        .iter()
                                        .find_map(|k| data.custom_maps.get(k))
                                    {
                                        let Some(&key) = i32_params.first() else {
                                            let msg = format!(
                                                "[HostCapabilityError] interface={} operation={} kind=capability_execution_failed: map-backed capability requires at least one i32 argument",
                                                iface_for_closure, fn_name_owned
                                            );
                                            store.data_mut().dependency_error = Some(msg.clone());
                                            return Err(wasmtime::Error::msg(msg));
                                        };
                                        if let Some(&val) = map.get(&key) {
                                            Ok(val)
                                        } else {
                                            Err(format!(
                                                "[HostCapabilityError] interface={} operation={} kind=unconfigured_capability: no entry configured in custom map for key={}",
                                                iface_for_closure, fn_name_owned, key
                                            ))
                                        }
                                    } else if is_default_host_iface {
                                        match norm_fn_name.as_str() {
                                            "kv_get" => {
                                                let key = i32_params.first().copied().unwrap_or(0);
                                                Ok(*data.kv.get(&key).unwrap_or(&0))
                                            }
                                            "emit_audit" => Ok(1),
                                            "check_credit" => {
                                                if !data.credit_configured && data.credit_ok.is_none() {
                                                    Err(format!(
                                                        "[HostCapabilityError] interface={} operation={} kind=unconfigured_capability: check_credit requires 'credit_ok' or 'credit_limits' in host_state",
                                                        iface_for_closure, fn_name_owned
                                                    ))
                                                } else {
                                                    let customer_id =
                                                        i32_params.first().copied().unwrap_or(0);
                                                    let amount =
                                                        i32_params.get(1).copied().unwrap_or(0);
                                                    let ok = if let Some(flag) = data.credit_ok {
                                                        flag
                                                    } else {
                                                        let limit = *data
                                                            .credit_limits
                                                            .get(&customer_id)
                                                            .unwrap_or(&0);
                                                        amount <= limit
                                                    };
                                                    Ok(if ok { 1 } else { 0 })
                                                }
                                            }
                                            "tenant_status" => {
                                                let tenant_id =
                                                    i32_params.first().copied().unwrap_or(0);
                                                if !data.tenant_status_configured {
                                                    Err(format!(
                                                        "[HostCapabilityError] interface={} operation={} kind=unconfigured_capability: tenant_status requires 'tenant_status' map in host_state",
                                                        iface_for_closure, fn_name_owned
                                                    ))
                                                } else if let Some(&status) =
                                                    data.tenant_status.get(&tenant_id)
                                                {
                                                    Ok(status)
                                                } else {
                                                    Err(format!(
                                                        "[HostCapabilityError] interface={} operation={} kind=unconfigured_capability: tenant_status has no configured entry for tenant_id={}",
                                                        iface_for_closure, fn_name_owned, tenant_id
                                                    ))
                                                }
                                            }
                                            _ => Err(format!(
                                                "[HostCapabilityError] interface={} operation={} kind=unregistered_capability: operation is not registered in host capability registry",
                                                iface_for_closure, fn_name_owned
                                            )),
                                        }
                                    } else {
                                        Err(format!(
                                            "[HostCapabilityError] interface={} operation={} kind=unregistered_capability: operation is not registered for interface '{}'",
                                            iface_for_closure, fn_name_owned, iface_for_closure
                                        ))
                                    }
                                };

                                let ret_i32 = match resolved_i32 {
                                    Ok(v) => v,
                                    Err(err_msg) => {
                                        store.data_mut().dependency_error = Some(err_msg.clone());
                                        return Err(wasmtime::Error::msg(err_msg));
                                    }
                                };

                                store.data_mut().all_calls.push(HostCallRecord {
                                    import: norm_fn_name.clone(),
                                    args: json_params,
                                });

                                if !results.is_empty() {
                                    match ret_types.first() {
                                        Some(wasmtime::component::types::Type::Bool) => {
                                            results[0] = wasmtime::component::Val::Bool(ret_i32 != 0);
                                        }
                                        _ => {
                                            results[0] = wasmtime::component::Val::S32(ret_i32);
                                        }
                                    }
                                }
                                Ok(())
                            })
                            .map_err(|e| e.to_string())?;
                    } else {
                        linker_inst
                            .func_new(func_name, move |mut store, params, results| {
                                let mut json_params = Vec::with_capacity(params.len());
                                for p in params {
                                    match p {
                                        wasmtime::component::Val::S32(v) => json_params.push(serde_json::json!(*v)),
                                        wasmtime::component::Val::Bool(b) => {
                                            json_params.push(serde_json::json!(if *b { 1 } else { 0 }))
                                        }
                                        _ => {}
                                    }
                                }
                                store.data_mut().all_calls.push(HostCallRecord {
                                    import: norm_fn_name.clone(),
                                    args: json_params,
                                });

                                let Some(linked_bytes) = store.data().linked_wasm_bytes.clone() else {
                                    let msg = format!(
                                        "[DependencyError] component={} operation={} kind=missing_dependency: required linked component binary was not provided",
                                        iface_for_closure, fn_name_owned
                                    );
                                    store.data_mut().dependency_error = Some(msg.clone());
                                    return Err(wasmtime::Error::msg(msg));
                                };

                                let child_engine = store.engine().clone();
                                let child_comp = match wasmtime::component::Component::from_binary(
                                    &child_engine,
                                    &linked_bytes,
                                ) {
                                    Ok(c) => c,
                                    Err(e) => {
                                        let msg = format!(
                                            "[DependencyError] component={} operation={} kind=invalid_binary: failed to parse dependent component binary: {e}",
                                            iface_for_closure, fn_name_owned
                                        );
                                        store.data_mut().dependency_error = Some(msg.clone());
                                        return Err(wasmtime::Error::msg(msg));
                                    }
                                };

                                let mut child_store =
                                    Store::new(&child_engine, store.data().clone_for_child());
                                let mut child_linker =
                                    wasmtime::component::Linker::<HostStoreData>::new(&child_engine);
                                if let Err(e) = register_component_imports(
                                    &child_engine,
                                    &child_comp,
                                    &mut child_linker,
                                ) {
                                    let msg = format!(
                                        "[DependencyError] component={} operation={} kind=link_failed: failed to link dependent component imports: {e}",
                                        iface_for_closure, fn_name_owned
                                    );
                                    store.data_mut().dependency_error = Some(msg.clone());
                                    return Err(wasmtime::Error::msg(msg));
                                }

                                let child_instance =
                                    match child_linker.instantiate(&mut child_store, &child_comp) {
                                        Ok(inst) => inst,
                                        Err(e) => {
                                            let msg = format!(
                                                "[DependencyError] component={} operation={} kind=instantiation_failed: failed to instantiate dependent component: {e}",
                                                iface_for_closure, fn_name_owned
                                            );
                                            store.data_mut().dependency_error = Some(msg.clone());
                                            return Err(wasmtime::Error::msg(msg));
                                        }
                                    };

                                let Some((child_func, _)) = find_exported_component_func(
                                    &child_engine,
                                    &child_comp,
                                    &child_instance,
                                    &mut child_store,
                                    &fn_name_owned,
                                ) else {
                                    let msg = format!(
                                        "[DependencyError] component={} operation={} kind=instantiation_failed: dependent component does not export required operation '{}'",
                                        iface_for_closure, fn_name_owned, fn_name_owned
                                    );
                                    store.data_mut().dependency_error = Some(msg.clone());
                                    return Err(wasmtime::Error::msg(msg));
                                };

                                if let Err(e) = child_func.call(&mut child_store, params, results) {
                                    let msg = if let Some(nested_err) =
                                        child_store.data().dependency_error.clone()
                                    {
                                        nested_err
                                    } else {
                                        format!(
                                            "[DependencyError] component={} operation={} kind=execution_trap: dependent component failed during '{}': {e}",
                                            iface_for_closure, fn_name_owned, fn_name_owned
                                        )
                                    };
                                    store.data_mut().dependency_error = Some(msg.clone());
                                    return Err(wasmtime::Error::msg(msg));
                                }
                                let _ = child_func.post_return(&mut child_store);
                                Ok(())
                            })
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn run_suite_component(
    wasm_bytes: &[u8],
    test_cases: &[TestCaseInput],
    linked_wasm_bytes: Option<Vec<u8>>,
    start: Instant,
) -> Result<SuiteReport, String> {
    let engine = Engine::default();
    let component = wasmtime::component::Component::from_binary(&engine, wasm_bytes)
        .map_err(|e| format!("Invalid WebAssembly component binary: {e}"))?;

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
        let parsed_host_state = parse_host_state(tc.host_state.as_ref());
        let expected_calls = parse_expected_host_calls(tc.expected_host_calls.as_ref());

        if let Some(err_msg) = conv_err {
            failed_count += 1;
            results.push(CaseResult {
                id: tc.id.clone(),
                requirement: tc.requirement.clone(),
                function: tc.function.clone(),
                args: json_args,
                expected: expected_json,
                actual: None,
                expected_host_calls: expected_calls,
                actual_host_calls: None,
                passed: false,
                error: Some(err_msg),
            });
            continue;
        }

        let initial_store_data = HostStoreData {
            kv: parsed_host_state.kv,
            credit_limits: parsed_host_state.credit_limits,
            credit_configured: parsed_host_state.credit_configured,
            credit_ok: parsed_host_state.credit_ok,
            tenant_status: parsed_host_state.tenant_status,
            tenant_status_configured: parsed_host_state.tenant_status_configured,
            custom_maps: parsed_host_state.custom_maps,
            custom_scalars: parsed_host_state.custom_scalars,
            unconfigured_capabilities: parsed_host_state.unconfigured_capabilities,
            failing_capabilities: parsed_host_state.failing_capabilities,
            all_calls: Vec::new(),
            linked_wasm_bytes: linked_wasm_bytes.clone(),
            dependency_error: None,
        };

        let mut store = Store::new(&engine, initial_store_data);
        let mut linker = wasmtime::component::Linker::<HostStoreData>::new(&engine);

        if let Err(e) = register_component_imports(&engine, &component, &mut linker) {
            failed_count += 1;
            results.push(CaseResult {
                id: tc.id.clone(),
                requirement: tc.requirement.clone(),
                function: tc.function.clone(),
                args: json_args,
                expected: expected_json,
                actual: None,
                expected_host_calls: expected_calls,
                actual_host_calls: None,
                passed: false,
                error: Some(format!("Failed to link component imports: {e}")),
            });
            continue;
        }

        let instance = match linker.instantiate(&mut store, &component) {
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
                    expected_host_calls: expected_calls,
                    actual_host_calls: None,
                    passed: false,
                    error: Some(format!("Failed to instantiate component: {e}")),
                });
                continue;
            }
        };

        let Some((func, func_ty)) =
            find_exported_component_func(&engine, &component, &instance, &mut store, &tc.function)
        else {
            failed_count += 1;
            results.push(CaseResult {
                id: tc.id.clone(),
                requirement: tc.requirement.clone(),
                function: tc.function.clone(),
                args: json_args,
                expected: expected_json,
                actual: None,
                expected_host_calls: expected_calls,
                actual_host_calls: None,
                passed: false,
                error: Some(format!(
                    "Exported component function '{}' not found",
                    tc.function
                )),
            });
            continue;
        };

        let param_types: Vec<wasmtime::component::types::Type> = func_ty.params().collect();
        if i32_args.len() != param_types.len() {
            failed_count += 1;
            results.push(CaseResult {
                id: tc.id.clone(),
                requirement: tc.requirement.clone(),
                function: tc.function.clone(),
                args: json_args,
                expected: expected_json,
                actual: None,
                expected_host_calls: expected_calls,
                actual_host_calls: None,
                passed: false,
                error: Some(format!(
                    "Function '{}' requires {} arguments, found {}",
                    tc.function,
                    param_types.len(),
                    i32_args.len()
                )),
            });
            continue;
        }

        let mut wasm_params = Vec::with_capacity(i32_args.len());
        for (arg_val, param_ty) in i32_args.iter().zip(param_types.iter()) {
            match param_ty {
                wasmtime::component::types::Type::Bool => {
                    wasm_params.push(wasmtime::component::Val::Bool(*arg_val != 0));
                }
                _ => {
                    wasm_params.push(wasmtime::component::Val::S32(*arg_val));
                }
            }
        }

        let num_results = func_ty.results().len();
        let mut wasm_results = vec![wasmtime::component::Val::Bool(false); num_results];

        let (actual_json, mut is_pass, mut err) =
            match func.call(&mut store, &wasm_params, &mut wasm_results) {
                Ok(()) => {
                    let _ = func.post_return(&mut store);
                    if let Some(dep_err) = store.data().dependency_error.clone() {
                        (None, false, Some(dep_err))
                    } else if let Some(first_res) = wasm_results.first() {
                        let act_val = component_val_to_json(first_res, &expected_json);
                        let pass = act_val == expected_json;
                        (Some(act_val), pass, None)
                    } else {
                        (Some(serde_json::Value::Null), expected_json.is_null(), None)
                    }
                }
                Err(e) => {
                    let err_str = if let Some(dep_err) = store.data().dependency_error.clone() {
                        dep_err
                    } else {
                        format!("Component execution trap: {e:#}")
                    };
                    (None, false, Some(err_str))
                }
            };

        let store_calls = store.data().all_calls.clone();
        let actual_matched_calls = if actual_json.is_none() {
            None
        } else if let Some(ref exp_calls) = expected_calls {
            let filtered: Vec<HostCallRecord> = store_calls
                .into_iter()
                .filter(|c| {
                    c.import == "emit_audit"
                        || c.import == "check_credit"
                        || exp_calls.iter().any(|e| e.import == c.import)
                })
                .collect();
            if is_pass && &filtered != exp_calls {
                is_pass = false;
                err = Some(format!(
                    "Host call mismatch: expected {:?}, actual {:?}",
                    exp_calls, filtered
                ));
            }
            Some(filtered)
        } else {
            None
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
            expected_host_calls: expected_calls,
            actual_host_calls: actual_matched_calls,
            passed: is_pass,
            error: err,
        });
    }

    Ok(SuiteReport {
        passed: passed_count,
        failed: failed_count,
        total: test_cases.len(),
        execution_ms: start.elapsed().as_millis(),
        cases: results,
    })
}

#[allow(dead_code)]
fn run_suite(wasm_bytes: &[u8], test_cases: &[TestCaseInput]) -> Result<SuiteReport, String> {
    run_suite_with_link(wasm_bytes, test_cases, None)
}

fn run_suite_with_link(
    wasm_bytes: &[u8],
    test_cases: &[TestCaseInput],
    linked_wasm_bytes: Option<Vec<u8>>,
) -> Result<SuiteReport, String> {
    let start = Instant::now();
    let is_component =
        wasm_bytes.starts_with(b"\0asm\x0d\0\x01\0") || wasm_bytes.starts_with(b"\0asm\r\0\x01\0");
    if is_component {
        run_suite_component(wasm_bytes, test_cases, linked_wasm_bytes, start)
    } else {
        run_suite_core(wasm_bytes, test_cases, start)
    }
}

fn run_suite_core(
    wasm_bytes: &[u8],
    test_cases: &[TestCaseInput],
    start: Instant,
) -> Result<SuiteReport, String> {
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
        let parsed_host_state = parse_host_state(tc.host_state.as_ref());
        let expected_calls = parse_expected_host_calls(tc.expected_host_calls.as_ref());

        if let Some(err_msg) = conv_err {
            failed_count += 1;
            results.push(CaseResult {
                id: tc.id.clone(),
                requirement: tc.requirement.clone(),
                function: tc.function.clone(),
                args: json_args,
                expected: expected_json,
                actual: None,
                expected_host_calls: expected_calls,
                actual_host_calls: None,
                passed: false,
                error: Some(err_msg),
            });
            continue;
        }

        let initial_store_data = HostStoreData {
            kv: parsed_host_state.kv,
            credit_limits: parsed_host_state.credit_limits,
            credit_configured: parsed_host_state.credit_configured,
            credit_ok: parsed_host_state.credit_ok,
            tenant_status: parsed_host_state.tenant_status,
            tenant_status_configured: parsed_host_state.tenant_status_configured,
            custom_maps: parsed_host_state.custom_maps,
            custom_scalars: parsed_host_state.custom_scalars,
            unconfigured_capabilities: parsed_host_state.unconfigured_capabilities,
            failing_capabilities: parsed_host_state.failing_capabilities,
            all_calls: Vec::new(),
            linked_wasm_bytes: None,
            dependency_error: None,
        };

        let mut store = Store::new(&engine, initial_store_data);
        let mut linker = Linker::new(&engine);

        linker
            .func_wrap(
                "host",
                "kv_get",
                |mut caller: Caller<'_, HostStoreData>, key: i32| -> i32 {
                    let val = *caller.data().kv.get(&key).unwrap_or(&0);
                    caller.data_mut().all_calls.push(HostCallRecord {
                        import: "kv_get".to_string(),
                        args: vec![serde_json::json!(key)],
                    });
                    val
                },
            )
            .map_err(|e| e.to_string())?;

        linker
            .func_wrap(
                "host",
                "emit_audit",
                |mut caller: Caller<'_, HostStoreData>, event_id: i32, payload: i32| -> i32 {
                    caller.data_mut().all_calls.push(HostCallRecord {
                        import: "emit_audit".to_string(),
                        args: vec![serde_json::json!(event_id), serde_json::json!(payload)],
                    });
                    1
                },
            )
            .map_err(|e| e.to_string())?;

        linker
            .func_wrap(
                "host",
                "check_credit",
                |mut caller: Caller<'_, HostStoreData>, customer_id: i32, amount: i32| -> i32 {
                    let ok = if let Some(flag) = caller.data().credit_ok {
                        if flag {
                            1
                        } else {
                            0
                        }
                    } else {
                        let limit = *caller.data().credit_limits.get(&customer_id).unwrap_or(&0);
                        if amount <= limit {
                            1
                        } else {
                            0
                        }
                    };
                    caller.data_mut().all_calls.push(HostCallRecord {
                        import: "check_credit".to_string(),
                        args: vec![serde_json::json!(customer_id), serde_json::json!(amount)],
                    });
                    ok
                },
            )
            .map_err(|e| e.to_string())?;

        let instance = match linker.instantiate(&mut store, &module) {
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
                    expected_host_calls: expected_calls,
                    actual_host_calls: None,
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
                expected_host_calls: expected_calls,
                actual_host_calls: None,
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

                let (actual_json, mut is_pass, mut err) =
                    match &tc.expected {
                        YamlValue::Mapping(map) => {
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
                                        let bytes: [u8; 4] =
                                            mem_data[offset..offset + 4].try_into().unwrap();
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
                            let act_val = serde_json::json!(ret_i32);
                            let pass = act_val == expected_json;
                            (Some(act_val), pass, None)
                        }
                    };

                let store_calls = store.data().all_calls.clone();
                let actual_matched_calls = if let Some(ref exp_calls) = expected_calls {
                    let filtered: Vec<HostCallRecord> = store_calls
                        .into_iter()
                        .filter(|c| {
                            c.import == "emit_audit"
                                || c.import == "check_credit"
                                || exp_calls.iter().any(|e| e.import == c.import)
                        })
                        .collect();
                    if is_pass && &filtered != exp_calls {
                        is_pass = false;
                        err = Some(format!(
                            "Host call mismatch: expected {:?}, actual {:?}",
                            exp_calls, filtered
                        ));
                    }
                    Some(filtered)
                } else {
                    None
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
                    expected_host_calls: expected_calls,
                    actual_host_calls: actual_matched_calls,
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
                    expected_host_calls: expected_calls,
                    actual_host_calls: None,
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
            link,
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

            let linked_bytes = if let Some(ref lpath) = link {
                match fs::read(lpath) {
                    Ok(b) => Some(b),
                    Err(e) => {
                        eprintln!("Failed to read linked WASM '{}': {e}", lpath.display());
                        return ExitCode::from(2);
                    }
                }
            } else {
                None
            };

            let report = match run_suite_with_link(&wasm_bytes, &parsed_cases, linked_bytes) {
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

    #[test]
    fn executes_v0_3_imports_and_audits_in_wasmtime() {
        let smoke_str = include_str!("../../../ir/smoke-v3.ir.json");
        let ir: serde_json::Value = serde_json::from_str(smoke_str).expect("smoke-v3 JSON");

        let wasm_bytes = compile_ir_to_wasm(&ir).expect("smoke-v3 must compile");
        let yaml = r#"
cases:
  - id: SMOKE3-01
    requirement: SMOKE-001
    function: process_order
    args: [42, 500]
    host_state:
      kv:
        42: 15
    expected:
      discount_cents: 15
      audit_status: 1
    expected_host_calls:
      - import: emit_audit
        args: [1001, 500]
"#;
        let cases = parse_cases(yaml).expect("parse YAML");
        let report = run_suite(&wasm_bytes, &cases).expect("run suite");
        assert_eq!(report.passed, 1, "report cases: {:?}", report.cases);
        assert_eq!(report.failed, 0);
    }

    #[test]
    fn executes_v0_4_component_in_wasmtime() {
        let program_json = include_str!("../../../ir/program-004.ir.json");
        let val: serde_json::Value = serde_json::from_str(program_json).expect("valid JSON");
        let wasm_bytes = irtool::compile_ir_to_wasm(&val).expect("compile to wasm");
        assert!(
            wasm_bytes.starts_with(b"\0asm\x0d\0\x01\0")
                || wasm_bytes.starts_with(b"\0asm\r\0\x01\0"),
            "Must be a WebAssembly Component binary"
        );

        let pub_yaml = include_str!("../../../tests/public-004.yaml");
        let cases = parse_cases(pub_yaml).expect("parse public-004 YAML");
        let report = run_suite(&wasm_bytes, &cases).expect("run suite public-004");
        assert_eq!(
            report.failed,
            0,
            "Public-004 failures: {:?}",
            report
                .cases
                .iter()
                .filter(|c| !c.passed)
                .collect::<Vec<_>>()
        );
        assert_eq!(report.passed, cases.len());

        let hid_yaml = include_str!("../../../tests/hidden-004.yaml");
        let hid_cases = parse_cases(hid_yaml).expect("parse hidden-004 YAML");
        let hid_report = run_suite(&wasm_bytes, &hid_cases).expect("run suite hidden-004");
        assert_eq!(
            hid_report.failed,
            0,
            "Hidden-004 failures: {:?}",
            hid_report
                .cases
                .iter()
                .filter(|c| !c.passed)
                .collect::<Vec<_>>()
        );
        assert_eq!(hid_report.passed, hid_cases.len());
    }

    #[test]
    fn executes_v0_5_composed_components_in_wasmtime() {
        let shipping_str = include_str!("../../../ir/shipping-005.ir.json");
        let shipping_ir: serde_json::Value =
            serde_json::from_str(shipping_str).expect("shipping-005 JSON");
        let shipping_wasm =
            irtool::compile_ir_to_wasm(&shipping_ir).expect("compile shipping component");
        assert!(
            shipping_wasm.starts_with(b"\0asm\x0d\0\x01\0")
                || shipping_wasm.starts_with(b"\0asm\r\0\x01\0"),
            "shipping must be native Component Model"
        );

        let order_str = include_str!("../../../ir/order-005.ir.json");
        let order_ir: serde_json::Value = serde_json::from_str(order_str).expect("order-005 JSON");
        let order_wasm = irtool::compile_ir_to_wasm(&order_ir).expect("compile order component");
        assert!(
            order_wasm.starts_with(b"\0asm\x0d\0\x01\0")
                || order_wasm.starts_with(b"\0asm\r\0\x01\0"),
            "order must be native Component Model"
        );

        let yaml = include_str!("../../../tests/narrative-005.yaml");
        let cases = parse_cases(yaml).expect("parse narrative-005 YAML");
        let report = run_suite_with_link(&order_wasm, &cases, Some(shipping_wasm))
            .expect("run suite narrative-005 with composed shipping");
        assert_eq!(
            report.failed,
            0,
            "Narrative-005 failures: {:?}",
            report
                .cases
                .iter()
                .filter(|c| !c.passed)
                .collect::<Vec<_>>()
        );
        assert_eq!(report.passed, cases.len());
    }

    #[test]
    fn executes_reservation_coordinator_in_wasmtime() {
        let narrative = include_str!("../../../spec/narrative-resource-reservation.md");
        let spec = orchestrator::parse_narrative(narrative, "narrative-resource-reservation.md")
            .expect("parse resource narrative");

        let quota_ir = orchestrator::synthesize_quota_engine_ir("quota-engine", &spec)
            .expect("synthesize quota IR");
        let quota_wasm =
            irtool::compile_ir_to_wasm(&quota_ir).expect("compile quota-engine to wasm component");

        let coord_ir =
            orchestrator::synthesize_reservation_coordinator_ir("reservation-coordinator", &spec)
                .expect("synthesize reservation coordinator IR");
        let coord_wasm = irtool::compile_ir_to_wasm(&coord_ir)
            .expect("compile reservation-coordinator to wasm component");

        let yaml = include_str!("../../../tests/resource-reservation.yaml");
        let cases = parse_cases(yaml).expect("parse resource-reservation YAML");

        let report = run_suite_with_link(&coord_wasm, &cases, Some(quota_wasm))
            .expect("run suite resource-reservation with composed quota-engine");

        assert_eq!(
            report.failed,
            0,
            "Resource reservation failures: {:?}",
            report
                .cases
                .iter()
                .filter(|c| !c.passed)
                .collect::<Vec<_>>()
        );
        assert_eq!(report.passed, cases.len());
    }

    #[test]
    fn test_cumulative_reservation_evolution_v1_v2_v3() {
        // Step 1: Base spec (v1)
        let narrative = include_str!("../../../spec/narrative-resource-reservation.md");
        let spec_v1 = orchestrator::parse_narrative(narrative, "narrative-resource-reservation.md")
            .expect("parse resource narrative");

        let quota_v1_ir =
            orchestrator::synthesize_quota_engine_ir("quota-engine", &spec_v1).unwrap();
        let quota_v1_wasm = irtool::compile_ir_to_wasm(&quota_v1_ir).unwrap();
        let coord_v1_ir = orchestrator::synthesize_reservation_coordinator_ir(
            "reservation-coordinator",
            &spec_v1,
        )
        .unwrap();
        let coord_v1_wasm = irtool::compile_ir_to_wasm(&coord_v1_ir).unwrap();

        let cases_v1 =
            parse_cases(include_str!("../../../tests/resource-reservation.yaml")).unwrap();
        let report_v1 =
            run_suite_with_link(&coord_v1_wasm, &cases_v1, Some(quota_v1_wasm.clone())).unwrap();
        assert_eq!(report_v1.failed, 0, "Base v1 failures");

        // Step 2: WI-RES-01 (v2) - Expand quota to 2500, reservation-coordinator remains unchanged
        let wi1 = include_str!("../../../spec/work-items-res/WI-RES-01-quota-limit.md");
        let spec_v2 = orchestrator::parse_work_items_consolidated(
            Some(&spec_v1),
            &[("WI-RES-01-quota-limit.md", wi1)],
        )
        .unwrap();

        let quota_v2_ir =
            orchestrator::synthesize_quota_engine_ir("quota-engine", &spec_v2).unwrap();
        let quota_v2_wasm = irtool::compile_ir_to_wasm(&quota_v2_ir).unwrap();
        // Coordinator is unmodified: uses coord_v1_wasm
        let cases_v2 =
            parse_cases(include_str!("../../../tests/resource-reservation-v2.yaml")).unwrap();
        let report_v2 =
            run_suite_with_link(&coord_v1_wasm, &cases_v2, Some(quota_v2_wasm.clone())).unwrap();
        assert_eq!(
            report_v2.failed,
            0,
            "WI-RES-01 (v2) failures: {:?}",
            report_v2
                .cases
                .iter()
                .filter(|c| !c.passed)
                .collect::<Vec<_>>()
        );

        // Step 3: WI-RES-02 (v3) - Add penalty points to reservation-coordinator, quota-engine remains v2 (2500)
        let wi2 = include_str!("../../../spec/work-items-res/WI-RES-02-penalty-eligibility.md");
        let spec_v3 = orchestrator::parse_work_items_consolidated(
            Some(&spec_v2),
            &[("WI-RES-02-penalty-eligibility.md", wi2)],
        )
        .unwrap();

        let coord_v3_ir = orchestrator::synthesize_reservation_coordinator_ir(
            "reservation-coordinator",
            &spec_v3,
        )
        .unwrap();
        let coord_v3_wasm = irtool::compile_ir_to_wasm(&coord_v3_ir).unwrap();
        let cases_v3 =
            parse_cases(include_str!("../../../tests/resource-reservation-v3.yaml")).unwrap();
        let report_v3 =
            run_suite_with_link(&coord_v3_wasm, &cases_v3, Some(quota_v2_wasm)).unwrap();
        assert_eq!(
            report_v3.failed,
            0,
            "WI-RES-02 (v3) failures: {:?}",
            report_v3
                .cases
                .iter()
                .filter(|c| !c.passed)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_dependency_failures_never_fallback_to_zero() {
        let narrative = include_str!("../../../spec/narrative-resource-reservation.md");
        let spec = orchestrator::parse_narrative(narrative, "narrative-resource-reservation.md")
            .expect("parse resource narrative");
        let coord_ir =
            orchestrator::synthesize_reservation_coordinator_ir("reservation-coordinator", &spec)
                .expect("synthesize reservation coordinator IR");
        let coord_wasm = irtool::compile_ir_to_wasm(&coord_ir).expect("compile coordinator");

        let single_case_yaml = r#"
cases:
  - id: NEG-DEP-01
    requirement: RES-002
    function: reserve_resource
    args: [1, 100, 200, 0]
    host_state:
      tenant_status:
        1: 1
    expected:
      status_code: 0
      approved_units: 0
      penalty_points: 0
"#;
        let cases = parse_cases(single_case_yaml).expect("parse negative test case");

        // 1. Missing dependency (None): must fail with missing_dependency and never match status_code: 0
        let rep_missing =
            run_suite_with_link(&coord_wasm, &cases, None).expect("run with missing dep");
        assert_eq!(
            rep_missing.passed, 0,
            "Missing dependency must not pass even when expected is 0"
        );
        assert_eq!(rep_missing.failed, 1);
        assert!(
            rep_missing.cases[0].actual.is_none(),
            "Actual business output must be None on dependency error"
        );
        let err_missing = rep_missing.cases[0].error.as_deref().unwrap_or("");
        assert!(
            err_missing.contains("[DependencyError]")
                && err_missing.contains("kind=missing_dependency"),
            "Error was: {err_missing}"
        );

        // 2. Corrupt / invalid binary: must fail with invalid_binary
        let rep_invalid = run_suite_with_link(
            &coord_wasm,
            &cases,
            Some(b"not-a-valid-wasm-component".to_vec()),
        )
        .expect("run with invalid dep binary");
        assert_eq!(rep_invalid.passed, 0);
        assert_eq!(rep_invalid.failed, 1);
        assert!(rep_invalid.cases[0].actual.is_none());
        let err_invalid = rep_invalid.cases[0].error.as_deref().unwrap_or("");
        assert!(
            err_invalid.contains("[DependencyError]")
                && err_invalid.contains("kind=invalid_binary"),
            "Error was: {err_invalid}"
        );

        // 3. Instantiation / contract mismatch (shipping-calculator passed instead of quota-engine)
        let shipping_str = include_str!("../../../ir/shipping-005.ir.json");
        let shipping_ir: serde_json::Value = serde_json::from_str(shipping_str).unwrap();
        let shipping_wasm = irtool::compile_ir_to_wasm(&shipping_ir).unwrap();
        let rep_mismatch = run_suite_with_link(&coord_wasm, &cases, Some(shipping_wasm))
            .expect("run with mismatched dep component");
        assert_eq!(rep_mismatch.passed, 0);
        assert_eq!(rep_mismatch.failed, 1);
        assert!(rep_mismatch.cases[0].actual.is_none());
        let err_mismatch = rep_mismatch.cases[0].error.as_deref().unwrap_or("");
        assert!(
            err_mismatch.contains("[DependencyError]")
                && err_mismatch.contains("kind=instantiation_failed"),
            "Error was: {err_mismatch}"
        );

        // 4. Execution trap inside dependent component (e.g., runtime division by zero in check_quota)
        let mut trapping_quota_ir =
            orchestrator::synthesize_quota_engine_ir("quota-engine", &spec).unwrap();
        trapping_quota_ir["functions"][0]["body"] = json!({
            "op": "div_s",
            "left": { "op": "param", "name": "requested_units" },
            "right": { "op": "sub", "left": { "op": "param", "name": "tenant_id" }, "right": { "op": "param", "name": "tenant_id" } }
        });
        let trapping_quota_wasm = irtool::compile_ir_to_wasm(&trapping_quota_ir)
            .expect("compile trapping quota component");
        let rep_trap = run_suite_with_link(&coord_wasm, &cases, Some(trapping_quota_wasm))
            .expect("run with trapping dep component");
        assert_eq!(rep_trap.passed, 0);
        assert_eq!(rep_trap.failed, 1);
        assert!(rep_trap.cases[0].actual.is_none());
        let err_trap = rep_trap.cases[0].error.as_deref().unwrap_or("");
        assert!(
            err_trap.contains("[DependencyError]") && err_trap.contains("kind=execution_trap"),
            "Error was: {err_trap}"
        );
    }

    #[test]
    fn test_unseen_llm_domain_canal_lock_v1_and_v2() {
        use orchestrator::ImplementationAgent;

        // 1. Verify SynthesizerAgent does NOT know hydro-sensor or lock-controller
        let narrative =
            include_str!("../../../runs/unknown-domain-e2e/spec/narrative-canal-lock.md");
        let spec_v1 = orchestrator::parse_narrative(narrative, "narrative-canal-lock.md")
            .expect("parse unseen domain narrative");
        assert!(
            orchestrator::SynthesizerAgent
                .propose_ir("hydro-sensor", &spec_v1, &[])
                .is_err(),
            "SynthesizerAgent must not have a hardcoded branch for hydro-sensor"
        );
        assert!(
            orchestrator::SynthesizerAgent
                .propose_ir("lock-controller", &spec_v1, &[])
                .is_err(),
            "SynthesizerAgent must not have a hardcoded branch for lock-controller"
        );

        // 2. Verify v1 LLM-generated IRs compile and pass 5/5 acceptance tests in dynamic runner
        let hydro_v1_ir: serde_json::Value = serde_json::from_str(include_str!(
            "../../../runs/unknown-domain-e2e/dist/versions/build-1792206976764513300/hydro-sensor.ir.json"
        ))
        .unwrap();
        let lock_v1_ir: serde_json::Value = serde_json::from_str(include_str!(
            "../../../runs/unknown-domain-e2e/dist/versions/build-1792206976764513300/lock-controller.ir.json"
        ))
        .unwrap();

        assert!(irtool::verify_ir(&hydro_v1_ir).is_empty());
        assert!(irtool::verify_ir(&lock_v1_ir).is_empty());

        let hydro_v1_wasm = irtool::compile_ir_to_wasm(&hydro_v1_ir).expect("compile hydro v1");
        let lock_v1_wasm = irtool::compile_ir_to_wasm(&lock_v1_ir).expect("compile lock v1");

        let cases_v1 = parse_cases(include_str!(
            "../../../runs/unknown-domain-e2e/tests/acceptance-v1.yaml"
        ))
        .expect("parse acceptance-v1.yaml");
        let rep_v1 = run_suite_with_link(&lock_v1_wasm, &cases_v1, Some(hydro_v1_wasm))
            .expect("run v1 suite");
        assert_eq!(rep_v1.failed, 0, "v1 failures: {:?}", rep_v1.cases);
        assert_eq!(rep_v1.passed, 5);

        // 3. Verify v2 evolved LLM-generated IRs compile and pass 5/5 acceptance tests in dynamic runner
        let hydro_v2_ir: serde_json::Value = serde_json::from_str(include_str!(
            "../../../runs/unknown-domain-e2e/dist/versions/build-1792207389433634100/hydro-sensor.ir.json"
        ))
        .unwrap();
        let lock_v2_ir: serde_json::Value = serde_json::from_str(include_str!(
            "../../../runs/unknown-domain-e2e/dist/versions/build-1792207389433634100/lock-controller.ir.json"
        ))
        .unwrap();

        assert!(irtool::verify_ir(&hydro_v2_ir).is_empty());
        assert!(irtool::verify_ir(&lock_v2_ir).is_empty());

        let hydro_v2_wasm = irtool::compile_ir_to_wasm(&hydro_v2_ir).expect("compile hydro v2");
        let lock_v2_wasm = irtool::compile_ir_to_wasm(&lock_v2_ir).expect("compile lock v2");

        let cases_v2 = parse_cases(include_str!(
            "../../../runs/unknown-domain-e2e/tests/acceptance-v2.yaml"
        ))
        .expect("parse acceptance-v2.yaml");
        let rep_v2 = run_suite_with_link(&lock_v2_wasm, &cases_v2, Some(hydro_v2_wasm))
            .expect("run v2 suite");
        assert_eq!(rep_v2.failed, 0, "v2 failures: {:?}", rep_v2.cases);
        assert_eq!(rep_v2.passed, 5);
    }

    #[test]
    fn test_unknown_host_capabilities_fail_explicitly() {
        let temp_wit_path = std::env::temp_dir().join("test_host_cap_gate.wit");
        let wit_src = r#"package kaddo:security@0.1.0;

interface host-services {
    validate-access: func(user-id: s32) -> bool;
}

interface access-gate {
    authorize-action: func(user-id: s32) -> s32;
}

world security-gate {
    import host-services;
    export access-gate;
}
"#;
        std::fs::write(&temp_wit_path, wit_src).expect("write temp wit");

        let gate_ir = json!({
            "ir_version": "0.5",
            "spec_version": 1,
            "wit_contract": {
                "package": "kaddo:security@0.1.0",
                "world": "security-gate",
                "export_interface": "access-gate",
                "import_interfaces": ["host-services"],
                "wit_file": temp_wit_path.to_string_lossy().to_string()
            },
            "capabilities": ["custom_host"],
            "imports": [
                {
                    "module": "host",
                    "name": "validate_access",
                    "capability": "custom_host",
                    "wit_interface": "host-services",
                    "params": [{ "name": "user_id", "type": "i32" }],
                    "returns": "bool"
                }
            ],
            "functions": [
                {
                    "name": "authorize_action",
                    "export": true,
                    "wit_interface": "access-gate",
                    "params": [{ "name": "user_id", "type": "i32" }],
                    "returns": "i32",
                    "requirements": ["SEC-001"],
                    "body": {
                        "op": "if",
                        "condition": {
                            "op": "call_import",
                            "import": "validate_access",
                            "args": [{ "op": "param", "name": "user_id" }]
                        },
                        "then": { "op": "const", "type": "i32", "value": 100 },
                        "else": { "op": "const", "type": "i32", "value": 200 }
                    }
                }
            ]
        });

        let gate_wasm =
            irtool::compile_ir_to_wasm(&gate_ir).expect("compile security-gate component");
        let _ = std::fs::remove_file(&temp_wit_path);

        // Case 1: Unknown capability without any host_state -> unregistered_capability, passed: false, actual: None
        // Notice expected is 200 (the else branch if validate_access defaulted to false/0): it MUST NOT pass!
        let yaml_case1 = r#"
cases:
  - id: CAP-CASE-1-UNKNOWN
    requirement: SEC-001
    function: authorize_action
    args: [42]
    expected: 200
"#;
        let rep1 = run_suite(&gate_wasm, &parse_cases(yaml_case1).unwrap()).unwrap();
        assert_eq!(rep1.passed, 0);
        assert_eq!(rep1.failed, 1);
        assert!(rep1.cases[0].actual.is_none());
        let err1 = rep1.cases[0].error.as_deref().unwrap_or("");
        assert!(
            err1.contains("[HostCapabilityError]")
                && err1.contains("interface=kaddo:security/host-services@0.1.0")
                && err1.contains("operation=validate-access")
                && err1.contains("kind=unregistered_capability"),
            "Unexpected Case 1 error: {err1}"
        );

        // Case 2: Unknown capability when KV has data for key 42 -> must NOT fallback to kv_get
        let yaml_case2 = r#"
cases:
  - id: CAP-CASE-2-UNKNOWN-WITH-KV
    requirement: SEC-001
    function: authorize_action
    args: [42]
    host_state:
      kv:
        42: 1
    expected: 100
"#;
        let rep2 = run_suite(&gate_wasm, &parse_cases(yaml_case2).unwrap()).unwrap();
        assert_eq!(rep2.passed, 0);
        assert_eq!(rep2.failed, 1);
        assert!(rep2.cases[0].actual.is_none());
        let err2 = rep2.cases[0].error.as_deref().unwrap_or("");
        assert!(
            err2.contains("[HostCapabilityError]")
                && err2.contains("operation=validate-access")
                && err2.contains("kind=unregistered_capability"),
            "Unexpected Case 2 error: {err2}"
        );

        // Also verify interface collision protection: registering validate_access for another interface must NOT match host-services
        let yaml_case2_collision = r#"
cases:
  - id: CAP-CASE-2B-OTHER-INTERFACE
    requirement: SEC-001
    function: authorize_action
    args: [42]
    host_state:
      other-interface/validate-access: 1
    expected: 100
"#;
        let rep2b = run_suite(&gate_wasm, &parse_cases(yaml_case2_collision).unwrap()).unwrap();
        assert_eq!(rep2b.passed, 0);
        assert!(rep2b.cases[0].actual.is_none());
        assert!(rep2b.cases[0]
            .error
            .as_deref()
            .unwrap_or("")
            .contains("kind=unregistered_capability"));

        // Case 3: Valid custom capability registered via custom_maps and custom_scalars
        let yaml_case3 = r#"
cases:
  - id: CAP-CASE-3A-CUSTOM-MAP
    requirement: SEC-001
    function: authorize_action
    args: [42]
    host_state:
      validate_access:
        42: 1
    expected: 100
  - id: CAP-CASE-3B-CUSTOM-SCALAR
    requirement: SEC-001
    function: authorize_action
    args: [99]
    host_state:
      host-services/validate-access: 1
    expected: 100
"#;
        let rep3 = run_suite(&gate_wasm, &parse_cases(yaml_case3).unwrap()).unwrap();
        assert_eq!(rep3.failed, 0, "Case 3 failures: {:?}", rep3.cases);
        assert_eq!(rep3.passed, 2);

        // Case 4: Legitimate zero / false return from a registered capability
        let yaml_case4 = r#"
cases:
  - id: CAP-CASE-4A-CUSTOM-MAP-ZERO
    requirement: SEC-001
    function: authorize_action
    args: [42]
    host_state:
      validate_access:
        42: 0
    expected: 200
  - id: CAP-CASE-4B-CUSTOM-SCALAR-FALSE
    requirement: SEC-001
    function: authorize_action
    args: [42]
    host_state:
      validate_access: false
    expected: 200
"#;
        let rep4 = run_suite(&gate_wasm, &parse_cases(yaml_case4).unwrap()).unwrap();
        assert_eq!(rep4.failed, 0, "Case 4 failures: {:?}", rep4.cases);
        assert_eq!(rep4.passed, 2);

        // Case 5: Missing mandatory configuration -> unconfigured_capability (and capability_execution_failed)
        let yaml_case5 = r#"
cases:
  - id: CAP-CASE-5A-NULL-CONFIG
    requirement: SEC-001
    function: authorize_action
    args: [42]
    host_state:
      validate_access: null
    expected: 200
  - id: CAP-CASE-5B-MISSING-KEY-IN-MAP
    requirement: SEC-001
    function: authorize_action
    args: [42]
    host_state:
      validate_access:
        10: 1
    expected: 200
  - id: CAP-CASE-5C-EXECUTION-FAILURE
    requirement: SEC-001
    function: authorize_action
    args: [42]
    host_state:
      validate_access: "error:auth service unreachable"
    expected: 200
"#;
        let rep5 = run_suite(&gate_wasm, &parse_cases(yaml_case5).unwrap()).unwrap();
        assert_eq!(rep5.passed, 0);
        assert_eq!(rep5.failed, 3);
        assert!(rep5.cases[0].actual.is_none());
        assert!(rep5.cases[0]
            .error
            .as_deref()
            .unwrap_or("")
            .contains("kind=unconfigured_capability"));
        assert!(rep5.cases[1].actual.is_none());
        assert!(rep5.cases[1]
            .error
            .as_deref()
            .unwrap_or("")
            .contains("kind=unconfigured_capability"));
        assert!(rep5.cases[2].actual.is_none());
        assert!(rep5.cases[2]
            .error
            .as_deref()
            .unwrap_or("")
            .contains("kind=capability_execution_failed"));

        // Also verify built-in tenant_status fails with unconfigured_capability when tenant_status is omitted in host_state
        let res_narrative = include_str!("../../../spec/narrative-resource-reservation.md");
        let res_spec =
            orchestrator::parse_narrative(res_narrative, "narrative-resource-reservation.md")
                .unwrap();
        let quota_wasm = irtool::compile_ir_to_wasm(
            &orchestrator::synthesize_quota_engine_ir("quota-engine", &res_spec).unwrap(),
        )
        .unwrap();
        let coord_wasm = irtool::compile_ir_to_wasm(
            &orchestrator::synthesize_reservation_coordinator_ir(
                "reservation-coordinator",
                &res_spec,
            )
            .unwrap(),
        )
        .unwrap();
        let unconf_tenant_yaml = r#"
cases:
  - id: CAP-CASE-5D-UNCONFIGURED-TENANT-STATUS
    requirement: RES-002
    function: reserve_resource
    args: [10, 1, 100, 200]
    expected:
      status_code: 0
      approved_units: 100
      penalty_points: 0
"#;
        let rep5d = run_suite_with_link(
            &coord_wasm,
            &parse_cases(unconf_tenant_yaml).unwrap(),
            Some(quota_wasm.clone()),
        )
        .unwrap();
        assert_eq!(rep5d.passed, 0);
        assert!(rep5d.cases[0].actual.is_none());
        assert!(rep5d.cases[0]
            .error
            .as_deref()
            .unwrap_or("")
            .contains("kind=unconfigured_capability"));

        // Case 6: Regression check across existing domains (orders/shipping, reservations, canal lock v1 & v2)
        let shipping_ir: serde_json::Value =
            serde_json::from_str(include_str!("../../../ir/shipping-005.ir.json")).unwrap();
        let order_ir: serde_json::Value =
            serde_json::from_str(include_str!("../../../ir/order-005.ir.json")).unwrap();
        let shipping_wasm = irtool::compile_ir_to_wasm(&shipping_ir).unwrap();
        let order_wasm = irtool::compile_ir_to_wasm(&order_ir).unwrap();
        let order_cases = parse_cases(include_str!("../../../tests/narrative-005.yaml")).unwrap();
        let rep_orders =
            run_suite_with_link(&order_wasm, &order_cases, Some(shipping_wasm)).unwrap();
        assert_eq!(rep_orders.failed, 0);

        let res_cases =
            parse_cases(include_str!("../../../tests/resource-reservation.yaml")).unwrap();
        let rep_res = run_suite_with_link(&coord_wasm, &res_cases, Some(quota_wasm)).unwrap();
        assert_eq!(rep_res.failed, 0);
    }
}
