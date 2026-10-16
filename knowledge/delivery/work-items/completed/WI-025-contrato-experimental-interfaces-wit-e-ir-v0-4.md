---
type: spike
id: WI-025
alias: KAN-POC4-WI-025
title: Contrato experimental, Interfaces WIT y Semantic IR v0.4
knowledge_level: K2
status: completed
implementation_status: completed
phase: completed
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
  - wit/order.wit
  - ir/ir.schema.v4.json
  - ir/smoke-v4.ir.json
  - spec/task-004.yaml
  - tests/public-004.yaml
---

# WI-025 (KAN-POC4-WI-025) — Contrato experimental, Interfaces WIT y Semantic IR v0.4

## Outcome

- **Actor:** Investigador principal y agente en `KADDO-AGENT-NATIVE-POC-004`.
- **Completed behavior:**
  - Se formalizó el contrato de interfaces estandarizadas en WebAssembly Interface Types en `wit/order.wit`, definiendo el paquete `kaddo:commercial@0.1.0`, el world `order-processor`, la interfaz exportada `order-calculator` y las interfaces importadas `host-services` y `types`.
  - Se publicó el esquema de validación `ir/ir.schema.v4.json` (JSON Schema Draft 2020-12) extendiendo la IR con el bloque `wit_contract` (`package`, `world`, `export_interface`, `import_interfaces`).
  - Se creó el programa de humo `ir/smoke-v4.ir.json` validando la estructura de la IR v0.4.
  - Se publicó la especificación comercial `spec/task-004.yaml` (v1) incorporando los requisitos `WIT-001`, `LINE-*`, `TAX-*` y `ORD-001` a `ORD-007`.
  - Se formuló la suite de pruebas de aceptación pública en `tests/public-004.yaml` con 7 casos cubriendo validación de líneas, cálculo de descuentos vía KV, auditoría de eventos y control de límite de crédito.
- **Observable completion:** `wit/order.wit`, `ir/ir.schema.v4.json`, `ir/smoke-v4.ir.json`, `spec/task-004.yaml` y `tests/public-004.yaml` válidos y versionados.

## Acceptance Criteria

- [x] Contrato WIT `wit/order.wit` formalizado con package `kaddo:commercial@0.1.0`, world `order-processor` e interfaces `order-calculator`, `host-services` y `types`.
- [x] Esquema `ir/ir.schema.v4.json` creado y validado bajo JSON Schema Draft 2020-12 con soporte para `wit_contract`.
- [x] Programa mínimo de humo `ir/smoke-v4.ir.json` conforme al esquema v4.
- [x] Especificación de tarea `spec/task-004.yaml` (v1) publicada con trazabilidad a requerimientos WIT y reglas comerciales.
- [x] Suite de pruebas públicas `tests/public-004.yaml` formulada con aserciones de tipos y efectos secundarios de host.
