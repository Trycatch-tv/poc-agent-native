---
type: spike
id: WI-014
alias: KAN-POC2-WI-014
title: E4-002 mantenimiento modular
knowledge_level: K2
status: completed
implementation_status: completed
phase: now
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
completed_at: '2026-10-16'
domains:
  - semantic-ir
  - commercial-rules-subject
  - deterministic-toolchain
  - experiment-evidence
code:
  - spec/task-002.yaml
  - ir/program-002.ir.json
  - runs/poc002-e4-maintain/**
ready_at: '2026-10-16'
---

# WI-014 (KAN-POC2-WI-014) — E4-002 mantenimiento modular

## Outcome

- **Actor:** Operador del experimento y agente evaluado en `KADDO-AGENT-NATIVE-POC-002`.
- **Current behavior:** La subfunción interna `calculate_tax` aplica una tasa estándar del 10 %.
- **Target behavior:** Se publica `spec/task-002.yaml` (v3) modificando la tasa de impuesto al 15 % (`subtotal_cents * 15 / 100`). El agente realiza el mantenimiento editando únicamente el cuerpo de la subfunción interna en `ir/program-002.ir.json` con un diff conciso (2 líneas), sin alterar `calculate_order` ni el backend congelado, y aprueba el 100 % de las pruebas de regresión. Se ejecutan 3 compilaciones deterministas consecutivas comprobando la igualdad exacta de SHA-256.
- **Observable completion:** Evidencia en `runs/poc002-e4-maintain/attempt-01/`; 3/3 builds deterministas con mismo hash (`0a2265bca6b1431f2ea15bb3da9831f5e1f8082230c36fd1659c613f9af88c85`); diff conciso localizado en la subfunción; 12/12 pruebas públicas y 18/18 pruebas ocultas aprobadas; toolchain hashes inalterados.

## Scope

- Operador publica `spec/task-002.yaml` (v3), actualiza `tests/public-002.yaml` y `tests/hidden-002.yaml`.
- El agente aplica el cambio modular en `ir/program-002.ir.json` localizándolo exclusivamente en `calculate_tax`.
- Se ejecuta `irtool verify`, `irtool compile` y `runner`.
- Se realizan 3 builds deterministas verificando igualdad bit a bit de SHA-256 (`SC-11`).
- Se verifica que los binarios congelados en `runs/toolchain-freeze-poc002.sha256` coinciden al 100 % (`SC-09`).
- Se capturan todas las evidencias en `runs/poc002-e4-maintain/attempt-01/`.

## Acceptance Criteria

- [x] `spec/task-002.yaml` (v3) publicado con la tasa de impuesto del 15 % (`TAX-002-V3`).
- [x] El agente realiza el mantenimiento en un diff localizado de pocas líneas sin alterar `calculate_order` ni el backend (`SC-09`, `SC-10`).
- [x] `irtool verify` valida con 0 diagnósticos.
- [x] 3 builds consecutivos producen exactamente el mismo SHA-256 en WASM (`SC-11`).
- [x] 100 % de las pruebas públicas (12/12) y reservadas (18/18) aprobadas (`SC-10`).
- [x] Los hashes de la toolchain coinciden al 100 % con `runs/toolchain-freeze-poc002.sha256`.

## Validation

- Comprobar que `diagnostics.json` es `[]`.
- Comprobar que `public-tests.json` (12/12) y `hidden-tests.json` (18/18) pasan al 100 %.
- Comprobar que 3 hashes SHA-256 consecutivos del `.wasm` son idénticos.
- Comprobar SHA-256 de `irtool.exe` y `runner.exe` contra `toolchain-freeze-poc002.sha256`.
