---
type: spike
id: WI-017
alias: KAN-POC3-WI-017
title: Contrato experimental y Semantic IR v0.3
knowledge_level: K2
status: completed
implementation_status: completed
phase: now
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
ready_at: '2026-10-16'
completed_at: '2026-10-16'
domains:
  - semantic-ir
  - commercial-rules-subject
  - deterministic-toolchain
code:
  - ir/ir.schema.v3.json
  - ir/smoke-v3.ir.json
  - spec/task-003.yaml
  - tests/public-003.yaml
---

# WI-017 (KAN-POC3-WI-017) — Contrato experimental y Semantic IR v0.3

## Outcome

- **Actor:** Investigador principal y agente en `KADDO-AGENT-NATIVE-POC-003`.
- **Current behavior:** La IR v0.2 sólo soportaba funciones puras autocontenidas y memoria lineal local, sin posibilidad de invocar servicios externos o registrar efectos secundarios.
- **Target behavior:** Formalizar el contrato experimental de la Fase 3 mediante `ir/ir.schema.v3.json` incorporando:
  1. Declaración de capabilities requeridas a nivel de módulo (`capabilities: ["kv_store", "audit_log"]`).
  2. Declaración de imports de host (`imports: [{ module, name, params, returns, capability }]`).
  3. Operador `call_import` con chequeo estático de permisos y concordancia de firma.
  4. Especificación funcional `spec/task-003.yaml` (v1) definiendo el cálculo de orden con consulta a inventario/tasa de descuento en host y emisión de auditoría.
  5. Suite de pruebas inicial `tests/public-003.yaml` incluyendo fixtures de host mock y aserciones de eventos emitidos.
- **Observable completion:** `ir/ir.schema.v3.json` válido (Draft 2020-12), `ir/smoke-v3.ir.json` de prueba mínima, `spec/task-003.yaml` (v1) y `tests/public-003.yaml`.

## Acceptance Criteria

- [x] Esquema `ir/ir.schema.v3.json` creado y validado con soporte para `capabilities`, `imports` y `call_import`.
- [x] Programa mínimo de humo `ir/smoke-v3.ir.json` creado y conforme con el esquema v3.
- [x] Especificación comercial de Fase 3 publicada en `spec/task-003.yaml` (v1) con requerimientos `LINE-*`, `TAX-*` y `ORD-*`.
- [x] Suite de pruebas públicas de Fase 3 publicada en `tests/public-003.yaml` con estado de host mock y aserciones de eventos.
