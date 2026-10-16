---
type: spike
id: WI-013
alias: KAN-POC2-WI-013
title: E3-002 extensión funcional con errores tipados
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
  - experiment-evidence
code:
  - spec/task-002.yaml
  - ir/program-002.ir.json
  - runs/poc002-e3-extend/**
---

# WI-013 (KAN-POC2-WI-013) — E3-002 extensión funcional con errores tipados

## Outcome

- **Actor:** Operador del experimento y agente evaluado en `KADDO-AGENT-NATIVE-POC-002`.
- **Current behavior:** `OrderCalculation` solo contenía campos monetarios y señalizaba errores con `-1`.
- **Target behavior:** Se publicó `spec/task-002.yaml` (v2) introduciendo `status_code` en `OrderCalculation` con códigos discriminados de dominio (`0 = OK`, `1 = ERR_INVALID_ITEM`, `2 = ERR_INVALID_SHIPPING`, `3 = ERR_LIMIT_EXCEEDED` si el subtotal con impuestos excede 1,000,000 centavos). El agente extendió `ir/program-002.ir.json` en 1 intento sin tocar el backend congelado (`SC-06`, `SC-07`), aprobando el 100 % de las pruebas nuevas y de regresión (`SC-08`).
- **Observable completion:** Artefactos completos en `runs/poc002-e3-extend/attempt-01/`; hashes de toolchain inalterados; y 100 % pass rate.

## Scope

- Operador publicó `spec/task-002.yaml` (v2), actualizó `tests/public-002.yaml` (12 casos) y `tests/hidden-002.yaml` (18 casos).
- El agente extendió `ir/program-002.ir.json` (versión 2) agregando `status_code` a `OrderCalculation` e implementando las reglas de validación de dominio.
- Se ejecutó `irtool verify`, `irtool compile` y `runner` sobre el binario generado.
- Se verificó que los binarios congelados en `runs/toolchain-freeze-poc002.sha256` coinciden al 100 %.
- Se capturaron todas las evidencias en `runs/poc002-e3-extend/attempt-01/`.

## Acceptance Criteria

- [x] `spec/task-002.yaml` (v2) publicado con los nuevos códigos de estado (`ORD-001-V2` a `ORD-004-V2`).
- [x] El agente extiende `ir/program-002.ir.json` en <= 3 intentos (`SC-06`).
- [x] `irtool verify` valida con 0 diagnósticos.
- [x] `irtool compile` emite `.wasm` válido sin tocar el compilador congelado (`SC-07`).
- [x] 100 % de las pruebas públicas (12/12) y reservadas (18/18) aprobadas (`SC-08`).
- [x] Los hashes de la toolchain coinciden exactamente con `runs/toolchain-freeze-poc002.sha256`.

## Validation

- `diagnostics.json` reportó `[]` (0 diagnósticos).
- `public-tests.json` (12/12) aprobado en 18 ms.
- `hidden-tests.json` (18/18) aprobado en 19 ms.
- Hashes de `target/debug/irtool.exe` y `target/debug/runner.exe` confirmados 100 % coincidentes con `runs/toolchain-freeze-poc002.sha256`.

## Learning

- **Qué se entregó:**
  1. `spec/task-002.yaml` v2: Formalización de estados discriminados de negocio (`status_code` 0 a 3).
  2. `ir/program-002.ir.json` v2: Extensión de la IR incorporando el nuevo campo de registro y las bifurcaciones de validación.
  3. `runs/poc002-e3-extend/attempt-01/`: Evidencia completa auditable con pass rate del 100 % (30/30 pruebas combinadas).
- **Decisiones técnicas:**
  1. Al modelar errores como códigos tipados en el propio registro de retorno, el cliente y el runner pueden evaluar la semántica exacta del fallo sin depender de señales externas no deterministas.
  2. La toolchain determinista congelada compiló directamente la IR v2 sin requerir un solo cambio en Rust.
