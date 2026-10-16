---
type: spike
id: WI-030
alias: KAN-POC4-WI-030
title: E4-004 Mantenimiento modularizado y composición interoperable
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
  - tests/public-004.yaml
  - tests/hidden-004.yaml
  - ir/program-004.ir.json
  - runs/poc004-e4-maintain/**
---

# WI-030 (KAN-POC4-WI-030) — E4-004 Mantenimiento modularizado y composición interoperable

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Completed behavior:**
  - Se actualizó `spec/task-004.yaml` a `version: 3` con incremento de la tasa impositiva base al 12 % y política de auditoría diferenciada (evento 999 para órdenes >= 10,000 centavos, evento 100 de lo contrario).
  - El agente realizó el mantenimiento de `ir/program-004.ir.json` (spec_version: 3) con un diff quirúrgico y conciso localizado exclusivamente en las subfunciones modulares `calculate_tax` y `build_valid_order_result`.
  - La interfaz externa WIT `order-calculator` y la función `quote_shipping` permanecieron intactas y 100 % interoperables.
  - La verificación estática con la toolchain frozen pasó con 0 diagnósticos en 1 solo intento (`attempt-01`).
  - La compilación en backend congelado generó el WebAssembly component de 956 bytes (`sha256=98fa9148b3ea155f914771f316da29f58bdf4f532c443d2c7111974916685904`).
  - 100 % de aprobación en tests: 9/9 en suite pública (4 ms) y 12/12 en suite reservada (4 ms), sumando 21/21 casos aprobados con verificación de estados y llamadas en el host.
  - Reproducibilidad binaria exacta demostrada (3/3 builds deterministas idénticos).
  - La toolchain release congelada permaneció 100 % inmutable contra `runs/toolchain-freeze-poc004.sha256`.
- **Observable verification:**
  - `runs/poc004-e4-maintain/attempt-01/diagnostics.json` (0 diagnósticos).
  - `runs/poc004-e4-maintain/attempt-01/public-report.json` (9/9 passed).
  - `runs/poc004-e4-maintain/attempt-01/hidden-report.json` (12/12 passed).
  - `runs/poc004-e4-maintain/attempt-01/build{1,2,3}.wasm` (idénticos a `program.wasm`).
