---
type: spike
id: WI-012
alias: KAN-POC2-WI-012
title: E2-002 reparación controlada
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
  - ir/program-002.ir.json
  - runs/poc002-e2-repair/**
---

# WI-012 (KAN-POC2-WI-012) — E2-002 reparación controlada

## Outcome

- **Actor:** Operador del experimento (fault injection) y agente evaluado en `KADDO-AGENT-NATIVE-POC-002`.
- **Current behavior:** `ir/program-002.ir.json` contenía la versión válida de `E1-002`.
- **Target behavior:** Se inyectó un defecto semántico controlado en un campo de record (`E_UNKNOWN_FIELD`). El verificador `irtool verify` rechazó la IR con `exit 1` y diagnóstico JSONPath explícito. El agente reparó la IR en 1 intento sin alterar el backend congelado y recuperó el 100 % de las pruebas produciendo exactamente el mismo binario bit a bit.
- **Observable completion:** Evidencia en `runs/poc002-e2-repair/attempt-01/`; diagnóstico de 593 bytes; reparación exitosa en 1 intento; y toolchain inalterada.

## Scope

- Operador guardó `runs/poc002-e2-repair/attempt-01/ir.before.json` e inyectó un defecto semántico controlado (`E_UNKNOWN_FIELD` en el constructor de `OrderCalculation`, usando `"subtotal"` en lugar de `"subtotal_cents"`).
- Se ejecutó `irtool verify` sobre la IR defectuosa, comprobando el rechazo determinista (`exit 1`) y la emisión de `diagnostics.injected.json` con JSONPath (`SC-04`).
- El agente reparó `ir/program-002.ir.json` en 1 intento guiado por el diagnóstico (`SC-05`).
- Se compiló y se probaron las suites pública (6/6) y reservada (9/9).
- Se verificó que los binarios congelados en `runs/toolchain-freeze-poc002.sha256` se mantuvieron inalterados.

## Acceptance Criteria

- [x] Defecto inyectado es rechazado determinísticamente por `irtool verify` con exit code != 0 y diagnóstico JSONPath explícito (`SC-04`).
- [x] El agente repara la IR en <= 2 intentos sin modificar el backend congelado (`SC-05`).
- [x] La IR reparada pasa `irtool verify` con 0 diagnósticos.
- [x] El binario compilado pasa `irtool validate` y aprueba el 100 % de pruebas públicas y reservadas.
- [x] Los hashes del toolchain coinciden exactamente con `runs/toolchain-freeze-poc002.sha256`.
- [x] Toda la evidencia auditable reside en `runs/poc002-e2-repair/attempt-01/`.

## Validation

- `diagnostics.injected.json` capturó `E_UNKNOWN_FIELD` con JSONPath `$.functions[2].body.else.fields.subtotal`.
- `diagnostics.json` post-reparación reportó `[]`.
- Suites de pruebas: 6/6 públicas (46 ms) y 9/9 reservadas (35 ms) aprobadas.
- Binario recuperado exactamente con SHA-256 `f42bcb6e03c616b2891a9c4d1e060c4e141d502b01f44a326bc749d65a0d771a`.
- Hashes del toolchain verificados idénticos a `runs/toolchain-freeze-poc002.sha256`.

## Learning

- **Qué se entregó:**
  1. `runs/poc002-e2-repair/attempt-01/ir.injected.json` y `diagnostics.injected.json`: Evidencia de rechazo determinista ante campos desconocidos en constructores de records.
  2. `runs/poc002-e2-repair/attempt-01/ir.after.json`: Reparación de 1 sola línea de diff guiada por el JSONPath.
  3. Recuperación exacta del SHA-256 de WASM sin ninguna mutación de código en el backend.
- **Decisiones técnicas:**
  1. La validación bidireccional en `record.construct` (campos desconocidos `E_UNKNOWN_FIELD` y campos faltantes `E_MISSING_FIELD`) guía al modelo de lenguaje directamente al punto de discrepancia del esquema sin ambigüedad sintáctica.
