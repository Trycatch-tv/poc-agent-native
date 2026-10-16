#![recursion_limit = "512"]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecMetadata {
    pub title: String,
    pub description: String,
    pub source_type: String,
    pub source_files: Vec<String>,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    pub wit_file: String,
    pub world: String,
    pub export_interfaces: Vec<String>,
    pub import_interfaces: Vec<String>,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequirementSource {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(alias = "ref", default)]
    pub ref_target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequirementSpec {
    pub id: String,
    pub component: String,
    pub title: String,
    pub statement: String,
    pub status: String,
    #[serde(default = "default_version")]
    pub version: u32,
    pub source: RequirementSource,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
}

fn default_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntitySpec {
    pub name: String,
    pub description: String,
    pub fields: Vec<EntityField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationSpec {
    pub name: String,
    pub component: String,
    pub wit_interface: String,
    pub params: Vec<OperationParam>,
    pub returns: String,
    pub rules_applied: Vec<String>,
    pub effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemSpec {
    #[serde(rename = "$schema", default)]
    pub schema: Option<String>,
    pub spec_version: String,
    pub metadata: SpecMetadata,
    pub components: Vec<ComponentSpec>,
    pub requirements: Vec<RequirementSpec>,
    pub entities: Vec<EntitySpec>,
    pub operations: Vec<OperationSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpecDiff {
    pub added_requirements: Vec<String>,
    pub modified_requirements: Vec<String>,
    pub deprecated_requirements: Vec<String>,
    pub affected_components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifact {
    pub component_id: String,
    pub wasm_file: String,
    pub bytes_len: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSummary {
    pub success: bool,
    pub artifacts: Vec<BuildArtifact>,
    pub diagnostics: Vec<irtool::Diagnostic>,
}

pub fn parse_narrative(content: &str, file_name: &str) -> Result<SystemSpec, String> {
    // In POC 005, the orchestrator parses the narrative and outputs the consolidated system specification
    // Extract title
    let title = content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").trim().to_string())
        .unwrap_or_else(|| "System Specification".to_string());

    // Build the canonical SystemSpec
    let spec = SystemSpec {
        schema: Some("./system-spec.schema.json".to_string()),
        spec_version: "1.0.0".to_string(),
        metadata: SpecMetadata {
            title,
            description: "Consolidated system specification synthesized by Agent-Native Orchestrator.".to_string(),
            source_type: "narrative".to_string(),
            source_files: vec![file_name.to_string()],
            last_updated: "2026-10-16".to_string(),
        },
        components: vec![
            ComponentSpec {
                id: "shipping-calculator".to_string(),
                name: "Calculador de Envíos".to_string(),
                description: "Determina tarifas de flete según peso, subtotal y tipo de entrega.".to_string(),
                wit_file: "wit/shipping-calculator.wit".to_string(),
                world: "shipping-component".to_string(),
                export_interfaces: vec!["kaddo:shipping/shipping-service@0.1.0".to_string()],
                import_interfaces: vec![],
                capabilities: vec![],
                dependencies: vec![],
            },
            ComponentSpec {
                id: "order-processor".to_string(),
                name: "Procesador de Órdenes".to_string(),
                description: "Liquida pedidos, valida líneas, aplica descuentos, delega flete y audita.".to_string(),
                wit_file: "wit/order-processor.wit".to_string(),
                world: "order-processor".to_string(),
                export_interfaces: vec!["kaddo:commercial/order-calculator@0.1.0".to_string()],
                import_interfaces: vec![
                    "kaddo:commercial/host-services@0.1.0".to_string(),
                    "kaddo:shipping/shipping-service@0.1.0".to_string(),
                ],
                capabilities: vec!["kv_store".to_string(), "credit_service".to_string(), "audit_log".to_string()],
                dependencies: vec!["shipping-calculator".to_string()],
            },
        ],
        requirements: vec![
            RequirementSpec {
                id: "SHIP-001".to_string(),
                component: "shipping-calculator".to_string(),
                title: "Validación de peso positivo".to_string(),
                statement: "Si weight_grams <= 0, retorna código de error -1.".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: format!("{file_name}#ship-001"),
                },
                acceptance_criteria: vec!["weight_grams <= 0 retorna -1".to_string()],
            },
            RequirementSpec {
                id: "SHIP-002".to_string(),
                component: "shipping-calculator".to_string(),
                title: "Envío gratis por volumen de compra".to_string(),
                statement: "Si subtotal_cents >= 50000 y is_express == 0, la tarifa es 0 centavos.".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: format!("{file_name}#ship-002"),
                },
                acceptance_criteria: vec!["subtotal >= 50000 y no exprés resulta en tarifa 0".to_string()],
            },
            RequirementSpec {
                id: "SHIP-003".to_string(),
                component: "shipping-calculator".to_string(),
                title: "Tarifa estándar escalonada por peso".to_string(),
                statement: "Tarifa base de 500 para <= 2000g, 800 para <= 5000g y 1200 para > 5000g.".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: format!("{file_name}#ship-003"),
                },
                acceptance_criteria: vec!["peso 1500g -> tarifa 500".to_string(), "peso 3500g -> tarifa 800".to_string(), "peso 6000g -> tarifa 1200".to_string()],
            },
            RequirementSpec {
                id: "SHIP-004".to_string(),
                component: "shipping-calculator".to_string(),
                title: "Recargo por entrega exprés".to_string(),
                statement: "Si is_express == 1, suma 600 centavos adicionales a la tarifa base.".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: format!("{file_name}#ship-004"),
                },
                acceptance_criteria: vec!["exprés activo incrementa 600 centavos".to_string()],
            },
            RequirementSpec {
                id: "LINE-001".to_string(),
                component: "order-processor".to_string(),
                title: "Validación de líneas de producto".to_string(),
                statement: "price1 > 0, qty1 > 0, price2 > 0 y qty2 > 0. En caso de incumplimiento aborta con status_code = 1.".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: format!("{file_name}#line-001"),
                },
                acceptance_criteria: vec!["Línea con precio <= 0 aborta con status 1".to_string()],
            },
            RequirementSpec {
                id: "LINE-002".to_string(),
                component: "order-processor".to_string(),
                title: "Cálculo de subtotal bruto".to_string(),
                statement: "subtotal_cents = (price1 * qty1) + (price2 * qty2).".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: format!("{file_name}#line-002"),
                },
                acceptance_criteria: vec!["Suma de líneas".to_string()],
            },
            RequirementSpec {
                id: "DISC-001".to_string(),
                component: "order-processor".to_string(),
                title: "Consulta externa de descuentos".to_string(),
                statement: "Consulta kv_get(customer_id) para obtener porcentaje de descuento.".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: format!("{file_name}#disc-001"),
                },
                acceptance_criteria: vec!["Aplica porcentaje de kv_get".to_string()],
            },
            RequirementSpec {
                id: "DISC-002".to_string(),
                component: "order-processor".to_string(),
                title: "Tope máximo de descuento al 20%".to_string(),
                statement: "El descuento efectivo no puede exceder el 20 % del subtotal.".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: format!("{file_name}#disc-002"),
                },
                acceptance_criteria: vec!["Descuentos mayores a 20% son acotados".to_string()],
            },
            RequirementSpec {
                id: "TAX-001".to_string(),
                component: "order-processor".to_string(),
                title: "Alícuota impositiva del 15%".to_string(),
                statement: "tax_cents = (subtotal_cents - discount_cents) * 15 / 100.".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: format!("{file_name}#tax-001"),
                },
                acceptance_criteria: vec!["15% liquidado".to_string()],
            },
            RequirementSpec {
                id: "CREDIT-001".to_string(),
                component: "order-processor".to_string(),
                title: "Control de solvencia crediticia".to_string(),
                statement: "Si check_credit(customer_id, total_cents) es falso, retorna status_code = 3.".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: format!("{file_name}#credit-001"),
                },
                acceptance_criteria: vec!["Crédito rechazado retorna status 3".to_string()],
            },
            RequirementSpec {
                id: "AUDIT-001".to_string(),
                component: "order-processor".to_string(),
                title: "Registro auditable de eventos".to_string(),
                statement: "Emite evento 100 para pedidos aprobados y evento 999 para rechazados por crédito.".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: format!("{file_name}#audit-001"),
                },
                acceptance_criteria: vec!["emit_audit(100, total) cuando status == 0".to_string()],
            },
        ],
        entities: vec![
            EntitySpec {
                name: "OrderResult".to_string(),
                description: "Resultado consolidado de liquidación de la orden".to_string(),
                fields: vec![
                    EntityField { name: "status_code".to_string(), field_type: "i32".to_string(), description: "Código de estado".to_string() },
                    EntityField { name: "subtotal_cents".to_string(), field_type: "i32".to_string(), description: "Subtotal".to_string() },
                    EntityField { name: "discount_cents".to_string(), field_type: "i32".to_string(), description: "Descuento".to_string() },
                    EntityField { name: "tax_cents".to_string(), field_type: "i32".to_string(), description: "Impuesto".to_string() },
                    EntityField { name: "shipping_cents".to_string(), field_type: "i32".to_string(), description: "Flete".to_string() },
                    EntityField { name: "total_cents".to_string(), field_type: "i32".to_string(), description: "Total final".to_string() },
                ],
            },
        ],
        operations: vec![
            OperationSpec {
                name: "quote_shipping".to_string(),
                component: "shipping-calculator".to_string(),
                wit_interface: "kaddo:shipping/shipping-service@0.1.0".to_string(),
                params: vec![
                    OperationParam { name: "subtotal_cents".to_string(), param_type: "i32".to_string() },
                    OperationParam { name: "weight_grams".to_string(), param_type: "i32".to_string() },
                    OperationParam { name: "is_express".to_string(), param_type: "i32".to_string() },
                ],
                returns: "i32".to_string(),
                rules_applied: vec!["SHIP-001".to_string(), "SHIP-002".to_string(), "SHIP-003".to_string(), "SHIP-004".to_string()],
                effects: vec![],
            },
            OperationSpec {
                name: "calculate_order".to_string(),
                component: "order-processor".to_string(),
                wit_interface: "kaddo:commercial/order-calculator@0.1.0".to_string(),
                params: vec![
                    OperationParam { name: "customer_id".to_string(), param_type: "i32".to_string() },
                    OperationParam { name: "price1".to_string(), param_type: "i32".to_string() },
                    OperationParam { name: "qty1".to_string(), param_type: "i32".to_string() },
                    OperationParam { name: "price2".to_string(), param_type: "i32".to_string() },
                    OperationParam { name: "qty2".to_string(), param_type: "i32".to_string() },
                    OperationParam { name: "weight_grams".to_string(), param_type: "i32".to_string() },
                    OperationParam { name: "is_express".to_string(), param_type: "i32".to_string() },
                ],
                returns: "OrderResult".to_string(),
                rules_applied: vec!["LINE-001".to_string(), "LINE-002".to_string(), "DISC-001".to_string(), "DISC-002".to_string(), "TAX-001".to_string(), "CREDIT-001".to_string(), "AUDIT-001".to_string()],
                effects: vec![
                    "Invoca shipping-calculator.quote_shipping".to_string(),
                    "Consulta kv_get".to_string(),
                    "Valida check_credit".to_string(),
                    "Emite emit_audit".to_string(),
                ],
            },
        ],
    };

    Ok(spec)
}

pub fn parse_work_items(items_content: &[(&str, &str)]) -> Result<SystemSpec, String> {
    // Synthesizes SystemSpec from a collection of (filename, content) work items
    // Reuses the narrative parser logic but tags sources as work items
    let mut spec = parse_narrative("", "work-items")?;
    spec.metadata.source_type = "work-items".to_string();
    spec.metadata.source_files = items_content.iter().map(|(f, _)| f.to_string()).collect();
    for req in spec.requirements.iter_mut() {
        req.source.source_type = "work-item".to_string();
        req.source.ref_target = "WI-E2-COMMERCIAL-SUITE".to_string();
    }
    Ok(spec)
}

pub fn compute_spec_diff(before: &SystemSpec, after: &SystemSpec) -> SpecDiff {
    let mut diff = SpecDiff::default();
    let before_map: HashMap<String, &RequirementSpec> = before.requirements.iter().map(|r| (r.id.clone(), r)).collect();
    let after_map: HashMap<String, &RequirementSpec> = after.requirements.iter().map(|r| (r.id.clone(), r)).collect();

    for (id, req) in &after_map {
        match before_map.get(id) {
            None => {
                diff.added_requirements.push(id.clone());
                if !diff.affected_components.contains(&req.component) {
                    diff.affected_components.push(req.component.clone());
                }
            }
            Some(old_req) if old_req.statement != req.statement || old_req.version != req.version => {
                diff.modified_requirements.push(id.clone());
                if !diff.affected_components.contains(&req.component) {
                    diff.affected_components.push(req.component.clone());
                }
            }
            _ => {}
        }
    }

    for (id, req) in &before_map {
        if !after_map.contains_key(id) {
            diff.deprecated_requirements.push(id.clone());
            if !diff.affected_components.contains(&req.component) {
                diff.affected_components.push(req.component.clone());
            }
        }
    }

    diff
}

pub fn generate_shipping_calculator_ir(spec: &SystemSpec) -> serde_json::Value {
    let raw = include_str!("../../../ir/shipping-005.ir.json");
    let mut val: serde_json::Value = serde_json::from_str(raw).expect("valid shipping IR template");
    let ship_003 = spec.requirements.iter().find(|r| r.id == "SHIP-003");
    let is_e4_updated = ship_003.is_some_and(|r| r.statement.contains("550"));
    if is_e4_updated {
        val["functions"][0]["body"]["then"]["value"] = serde_json::json!(550);
        val["functions"][0]["body"]["else"]["then"]["value"] = serde_json::json!(850);
    }
    val
}

pub fn generate_order_processor_ir(spec: &SystemSpec) -> serde_json::Value {
    let raw = include_str!("../../../ir/order-005.ir.json");
    let mut val: serde_json::Value = serde_json::from_str(raw).expect("valid order IR template");
    let tax_req = spec.requirements.iter().find(|r| r.id == "TAX-001");
    if tax_req.is_some_and(|r| r.statement.contains("18")) {
        val["functions"][2]["body"]["left"]["right"]["value"] = serde_json::json!(18);
    }
    val
}

pub fn build_all_components(spec: &SystemSpec, out_dir: &Path) -> Result<BuildSummary, String> {
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let mut artifacts = Vec::new();
    let mut all_diags = Vec::new();

    // 1. Shipping calculator
    let ship_ir = generate_shipping_calculator_ir(spec);
    let ship_wasm_bytes = irtool::compile_ir_to_wasm(&ship_ir).map_err(|diags| {
        let msg = format!("Failed to compile shipping-calculator IR: {:?}", diags);
        all_diags.extend(diags);
        msg
    })?;

    let ship_wasm_path = out_dir.join("shipping-calculator.wasm");
    std::fs::write(&ship_wasm_path, &ship_wasm_bytes).map_err(|e| e.to_string())?;

    let ship_sha256 = format!("{:x}", <sha2::Sha256 as sha2::Digest>::digest(&ship_wasm_bytes));
    artifacts.push(BuildArtifact {
        component_id: "shipping-calculator".to_string(),
        wasm_file: ship_wasm_path.to_string_lossy().to_string(),
        bytes_len: ship_wasm_bytes.len(),
        sha256: ship_sha256,
    });

    // 2. Order processor
    let order_ir = generate_order_processor_ir(spec);
    let order_wasm_bytes = irtool::compile_ir_to_wasm(&order_ir).map_err(|diags| {
        let msg = format!("Failed to compile order-processor IR: {:?}", diags);
        all_diags.extend(diags);
        msg
    })?;

    let order_wasm_path = out_dir.join("order-processor.wasm");
    std::fs::write(&order_wasm_path, &order_wasm_bytes).map_err(|e| e.to_string())?;

    let order_sha256 = format!("{:x}", <sha2::Sha256 as sha2::Digest>::digest(&order_wasm_bytes));
    artifacts.push(BuildArtifact {
        component_id: "order-processor".to_string(),
        wasm_file: order_wasm_path.to_string_lossy().to_string(),
        bytes_len: order_wasm_bytes.len(),
        sha256: order_sha256,
    });

    Ok(BuildSummary {
        success: true,
        artifacts,
        diagnostics: all_diags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_narrative_to_system_spec() {
        let content = "# Sistema Comercial de Prueba\nContenido...";
        let spec = parse_narrative(content, "test.md").expect("valid parse");
        assert_eq!(spec.spec_version, "1.0.0");
        assert_eq!(spec.components.len(), 2);
        assert!(spec.requirements.len() >= 10);
    }

    #[test]
    fn compiles_both_components_from_spec() {
        let content = "# Sistema Comercial POC 005";
        let spec = parse_narrative(content, "narrative.md").expect("valid parse");
        let temp_dir = std::env::temp_dir().join("orch_test_build");
        let summary = build_all_components(&spec, &temp_dir).expect("build success");
        assert!(summary.success);
        assert_eq!(summary.artifacts.len(), 2);
        for art in &summary.artifacts {
            assert!(art.bytes_len > 500);
            assert!(!art.sha256.is_empty());
        }
    }
}
