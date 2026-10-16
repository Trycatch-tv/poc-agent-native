---
type: spike
id: WI-011
alias: KAN-POC2-WI-011
title: E1-002 construcción y freeze
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
  - experiment-evidence
code:
  - ir/program-002.ir.json
  - runs/poc002-e1-build/**
  - runs/toolchain-freeze-poc002.sha256
---

# WI-011 (KAN-POC2-WI-011) — E1-002 construcción y freeze

## Outcome

- **Actor:** Agente evaluado y operador del experimento en `KADDO-AGENT-NATIVE-POC-002`.
- **Current behavior:** No existe implementación de la tarea de orden comercial v1 en IR v0.2 ni evidencia de ejecución de `E1-002`.
- **Target behavior:** El agente lee `spec/task-002.yaml` (v1) e implementa en `ir/program-002.ir.json` la orden comercial (`calculate_order`), apoyándose en una subfunción interna (`calculate_item_total`) y usando records para agrupar campos de orden (`OrderCalculation { subtotal_cents, tax_cents, total_cents }`). Pasa verificación sin errores, compila a `.wasm`, aprueba el 100 % de pruebas públicas y reservadas, y se sella el punto de congelamiento en `runs/toolchain-freeze-poc002.sha256` y tag git `poc-002-e1-freeze`.
- **Observable completion:** `program-002.wasm` generado y validado con `wasmparser`; 100 % de casos aprobados; artefactos en `runs/poc002-e1-build/attempt-01/`; y toolchain congelada.
