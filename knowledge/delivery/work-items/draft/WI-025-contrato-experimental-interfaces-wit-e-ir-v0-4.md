---
type: spike
id: WI-025
alias: KAN-POC4-WI-025
title: Contrato experimental, Interfaces WIT y Semantic IR v0.4
knowledge_level: K2
status: draft
implementation_status: not-started
phase: now
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
domains:
  - semantic-ir
  - commercial-rules-subject
  - deterministic-toolchain
code:
  - wit/order.wit
  - ir/ir.schema.v4.json
  - ir/smoke-v4.ir.json
  - spec/task-004.yaml
  - tests/public-004.yaml
---

# WI-025 (KAN-POC4-WI-025) — Contrato experimental, Interfaces WIT y Semantic IR v0.4

## Outcome

- **Actor:** Investigador principal y agente en `KADDO-AGENT-NATIVE-POC-004`.
- **Current behavior:** La IR v0.3 sólo soportaba WebAssembly Core con escalares y offsets calculados en memoria lineal interna, sin interfaces estandarizadas ni interoperabilidad mediante Component Model.
- **Target behavior:** Formalizar el contrato experimental de la Fase 4 mediante:
  1. Definición formal de la interfaz en WebAssembly Interface Types: `wit/order.wit` (`kaddo:commercial/order-processor`).
  2. Esquema `ir/ir.schema.v4.json` con soporte para vinculación a contratos WIT (`wit_package`, `export_interface`, `import_interfaces`).
  3. Programa de humo mínimo `ir/smoke-v4.ir.json`.
  4. Especificación funcional `spec/task-004.yaml` (v1) mapeada a la interfaz WIT.
  5. Suite de pruebas inicial `tests/public-004.yaml` formulada con inputs y outputs tipados conforme al contrato WIT.
- **Observable completion:** `wit/order.wit` válido, `ir/ir.schema.v4.json` conforme a Draft 2020-12, `ir/smoke-v4.ir.json`, `spec/task-004.yaml` y `tests/public-004.yaml`.
