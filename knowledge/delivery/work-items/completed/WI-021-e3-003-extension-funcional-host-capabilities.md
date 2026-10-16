---
type: spike
id: WI-021
alias: KAN-POC3-WI-021
title: E3-003 Extensión funcional con nueva regla de host
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
  - runs/poc003-e3-extend/**
---

# WI-021 (KAN-POC3-WI-021) — E3-003 Extensión funcional con nueva regla de host

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Completed behavior:**
  - Se publicó `spec/task-003.yaml` (v2) incorporando la capability `credit_service`, el import `check_credit(customer_id, amount) -> bool` y el requisito `ORD-007` para discriminación de crédito insuficiente (`status_code = 3`).
  - El agente extendió `ir/program-003.ir.json` en 1 solo intento incorporando la nueva capability, import y validación condicional en `build_valid_order_result`.
  - El compilador congelado (`target/release/irtool.exe`) verificó y compiló la nueva IR v0.3 con 0 diagnósticos generando un módulo WASM de 697 bytes sin ninguna modificación a la toolchain.
  - La suite de pruebas de host mock en Wasmtime alcanzó 100 % de aprobación (7/7 públicas, 10/10 ocultas; 17/17 total en POC 003).
  - Regresión confirmada: 100 % de casos pasados en POC 001 (12/12) y POC 002 (12/12 pub, 18/18 hid).
  - La toolchain permaneció 100 % inmutable (mismos hashes SHA-256 custodiados en `runs/toolchain-freeze-poc003.sha256`).
- **Observable verification:**
  - `runs/poc003-e3-extend/attempt-01/diagnostics.json` (0 diagnósticos).
  - `runs/poc003-e3-extend/attempt-01/public-tests.json` (7/7 passed, 3 ms).
  - `runs/poc003-e3-extend/attempt-01/hidden-tests.json` (10/10 passed, 3 ms).
  - `runs/poc003-e3-extend/attempt-01/poc001-regression.json` (12/12 passed, 1 ms).
  - `runs/poc003-e3-extend/attempt-01/poc002-pub-regression.json` (12/12 passed, 3 ms).
  - `runs/poc003-e3-extend/attempt-01/poc002-hid-regression.json` (18/18 passed, 2 ms).
  - Hashes de toolchain congelada validados intactos.
