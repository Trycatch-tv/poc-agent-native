---
type: spike
id: WI-027
alias: KAN-POC4-WI-027
title: E1-004 Construcción de Componente con Interfaces WIT y Freeze
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
  - experiment-evidence
code:
  - spec/task-004.yaml
  - wit/order.wit
  - tests/hidden-004.yaml
  - ir/program-004.ir.json
  - runs/poc004-e1-build/**
  - runs/toolchain-freeze-poc004.sha256
---

# WI-027 (KAN-POC4-WI-027) — E1-004 Construcción de Componente con Interfaces WIT y Freeze

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Completed behavior:**
  - El agente construyó de forma autónoma `ir/program-004.ir.json` a partir de `spec/task-004.yaml` (v1) y `wit/order.wit` en 1 intento (`attempt-01`).
  - El componente declara el contrato WIT (`kaddo:commercial@0.1.0`, world `order-processor`, `export_interface: order-calculator`, `import_interfaces: [host-services]`).
  - Importa de forma declarativa con enlace WIT las capacidades `kv_get`, `check_credit` y `emit_audit`.
  - Descompone la lógica en subfunciones modulares (`calculate_line_item`, `calculate_tax`, `calculate_subtotal`, `calculate_discount`, `calculate_total`, `build_valid_order_result`, `dispatch_order_calculation`, `process_valid_order`, `evaluate_shipping_and_proceed`), implementa y exporta `calculate_order` vinculado a la interfaz `order-calculator`.
  - Pasó la verificación estática del contrato WIT y semántica con 0 diagnósticos.
  - Generó el WebAssembly component ejecutable (869 bytes) con la sección personalizada canónica `component-type:order-processor` incrustada.
  - 100 % de aprobación en suites públicas y reservadas (7/7 públicas en 3 ms, 10/10 reservadas en 3 ms; 17/17 total) en Wasmtime Component Runtime con host mocks y verificación de llamadas.
  - Reproducibilidad binaria exacta demostrada (3/3 builds sucesivos idénticos con SHA-256 `b7c0a8c822b6f50f85c1151c1812f6f47b9d5c133a39ab32fb5dc01ddb1fab89`).
  - Toolchain release congelada en `runs/toolchain-freeze-poc004.sha256`:
    - `target/release/irtool.exe`: `8486e9fbeba08c5760bcad8fd6d712970f38e666baf90e5c66be5f1d75c93a38`
    - `target/release/runner.exe`: `045c5644bbad21a29644b9e21dd54cb465c2c4f6da60dc3cf996f587563ab5b3`
- **Observable verification:**
  - `runs/poc004-e1-build/attempt-01/diagnostics.json` (0 diagnósticos).
  - `runs/poc004-e1-build/attempt-01/public-report.json` (7/7 passed, 3 ms).
  - `runs/poc004-e1-build/attempt-01/hidden-report.json` (10/10 passed, 3 ms).
  - `runs/poc004-e1-build/attempt-01/build{1,2,3}.wasm` (idénticos a `program.wasm`).
  - `runs/toolchain-freeze-poc004.sha256` registrado y verificado.
