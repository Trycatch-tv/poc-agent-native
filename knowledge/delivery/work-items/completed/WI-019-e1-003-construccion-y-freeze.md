---
type: spike
id: WI-019
alias: KAN-POC3-WI-019
title: E1-003 Construcción con Capabilities y Freeze
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
  - spec/task-003.yaml
  - ir/program-003.ir.json
  - runs/poc003-e1-build/**
  - runs/toolchain-freeze-poc003.sha256
---

# WI-019 (KAN-POC3-WI-019) — E1-003 Construcción con Capabilities y Freeze

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Completed behavior:**
  - El agente construyó de forma autónoma `ir/program-003.ir.json` a partir de `spec/task-003.yaml` (v1) en 1 intento (`attempt-01`).
  - El programa declara las capabilities `kv_store` y `audit_log`, importa `kv_get` y `emit_audit`, descompone la lógica en subfunciones (`calculate_line_item`, `calculate_tax`, `calculate_subtotal`, `calculate_discount`, `calculate_total`, `build_valid_order_result`, `process_valid_order`, `dispatch_order_calculation`, `evaluate_shipping_and_proceed`), retorna `OrderResult` en memoria lineal y emite el evento de auditoría `emit_audit(100, total_cents)`.
  - Pasó la verificación estática con 0 diagnósticos.
  - 100 % de casos de prueba aprobados (6/6 en suite pública, 9/9 en suite oculta; 15/15 total) con validación exacta de llamadas y efectos secundarios en host.
  - Reproducibilidad binaria exacta confirmada (3 builds sucesivos idénticos con SHA-256 `637ee58cf943616fa80db2d87abb53a22ec04755a7ba001b3fbd7e79ad003be7`).
  - Toolchain congelada y sellada en `runs/toolchain-freeze-poc003.sha256` con tag de freeze `poc-003-e1-freeze`:
    - `target/release/irtool.exe`: `1eff80daf7136e89766ccc55a2050b5c389e6b01ed89a2fbc633894b6ac99668`
    - `target/release/runner.exe`: `66700a4da713cb918ad722165a3b6bf26d3ebe57a7c483ee56f1ecb5460b9067`
- **Observable verification:**
  - `runs/poc003-e1-build/attempt-01/diagnostics.json` (0 diagnósticos).
  - `runs/poc003-e1-build/attempt-01/public-tests.json` (6/6 passed, 4 ms).
  - `runs/poc003-e1-build/attempt-01/hidden-tests.json` (9/9 passed, 3 ms).
  - `runs/toolchain-freeze-poc003.sha256` registrado y custodiado.
