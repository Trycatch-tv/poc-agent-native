---
type: spike
id: WI-028
alias: KAN-POC4-WI-028
title: E2-004 Reparación controlada guiada por WIT diagnostics
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
  - deterministic-toolchain
  - experiment-evidence
code:
  - ir/program-004.ir.json
  - runs/poc004-e2-repair/**
---

# WI-028 (KAN-POC4-WI-028) — E2-004 Reparación controlada guiada por WIT diagnostics

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Completed behavior:**
  - El operador inyectó un defecto en la IR v0.4 (`runs/poc004-e2-repair/attempt-01-defect/program.ir.json`), asignando la interfaz exportada `wit_interface: "unregistered-commercial-api"` no coincidente con el contrato WIT `order-calculator`.
  - La toolchain release congelada rechazó determinísticamente el programa con exit code 1 y emitió un diagnóstico estructurado en `runs/poc004-e2-repair/attempt-01-defect/diagnostics.json` de 366 bytes (< 600 B) con código `E_UNKNOWN_EXPORT_INTERFACE` y JSONPath exacto `$.functions[9].wit_interface`.
  - El agente reparó de manera autónoma el defecto en 1 solo intento (`attempt-02-repaired`), restaurando la interfaz a `"order-calculator"`.
  - La verificación estática post-reparación pasó con 0 diagnósticos.
  - La compilación produjo el WebAssembly component de 869 bytes con paridad de hash exacta al build de freeze E1-004 (`b7c0a8c822b6f50f85c1151c1812f6f47b9d5c133a39ab32fb5dc01ddb1fab89`).
  - 100 % de aprobación en tests (7/7 públicas en 3 ms, 10/10 reservadas en 3 ms; 17/17 total).
  - La toolchain release congelada (`target/release/irtool.exe` y `target/release/runner.exe`) permaneció 100 % inmutable contra `runs/toolchain-freeze-poc004.sha256`.
- **Observable verification:**
  - `runs/poc004-e2-repair/attempt-01-defect/diagnostics.json` (366 bytes, código `E_UNKNOWN_EXPORT_INTERFACE`).
  - `runs/poc004-e2-repair/attempt-02-repaired/diagnostics.json` (0 diagnósticos).
  - `runs/poc004-e2-repair/attempt-02-repaired/public-report.json` (7/7 passed).
  - `runs/poc004-e2-repair/attempt-02-repaired/hidden-report.json` (10/10 passed).
  - Paridad SHA-256 binaria restaurada: `b7c0a8c822b6f50f85c1151c1812f6f47b9d5c133a39ab32fb5dc01ddb1fab89`.
