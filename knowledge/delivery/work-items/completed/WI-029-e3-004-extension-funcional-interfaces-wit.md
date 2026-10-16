---
type: spike
id: WI-029
alias: KAN-POC4-WI-029
title: E3-004 Extensión funcional con interfaces y variantes ricas en WIT
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
  - tests/public-004.yaml
  - tests/hidden-004.yaml
  - ir/program-004.ir.json
  - runs/poc004-e3-extend/**
---

# WI-029 (KAN-POC4-WI-029) — E3-004 Extensión funcional con interfaces y variantes ricas en WIT

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Completed behavior:**
  - El contrato WIT en `wit/order.wit` y la especificación en `spec/task-004.yaml` fueron actualizados a `version: 2`, incorporando la nueva función exportada `quote-shipping: func(weight-grams: s32, is-express: s32) -> s32;` bajo la interfaz `order-calculator`.
  - Las suites de pruebas se ampliaron con 4 nuevos casos (2 públicos `TC-PUB-408`/`409`, 2 ocultos `E3-004-H11`/`H12`).
  - El agente extendió autónomamente `ir/program-004.ir.json` (spec_version: 2) implementando `quote_shipping` con enlace WIT a `order-calculator` en 1 solo intento (`attempt-01`).
  - La verificación estática del contrato ampliado pasó con 0 diagnósticos en la toolchain frozen.
  - La compilación en el backend congelado generó el WebAssembly component ampliado de 942 bytes (`sha256=af059e3a42582c5805c89c5d4dbd5fe8e6c4057564a823d884751f28bbf4b3b4`).
  - 100 % de aprobación en tests: 9/9 en suite pública (3 ms) y 12/12 en suite reservada (3 ms) para un total de 21/21 casos aprobados, sin regresión en `calculate_order`.
  - Reproducibilidad binaria demostrada (3/3 builds idénticos).
  - La toolchain release congelada permaneció 100 % inmutable contra `runs/toolchain-freeze-poc004.sha256`.
- **Observable verification:**
  - `runs/poc004-e3-extend/attempt-01/diagnostics.json` (0 diagnósticos).
  - `runs/poc004-e3-extend/attempt-01/public-report.json` (9/9 passed).
  - `runs/poc004-e3-extend/attempt-01/hidden-report.json` (12/12 passed).
  - `runs/poc004-e3-extend/attempt-01/build{1,2,3}.wasm` (idénticos a `program.wasm`).
