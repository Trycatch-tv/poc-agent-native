---
type: spike
id: WI-005
alias: KAN-POC-WI-005
title: E3 extensión funcional
knowledge_level: K2
status: draft
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
  - spec/task.yaml
  - tests/public.yaml
  - ir/program.ir.json
  - runs/e3-extend/**
---

# WI-005 (KAN-POC-WI-005) — E3 extensión funcional

## Outcome

- **Actor:** Operador del experimento (publicación de `spec/task.yaml` v2 y pruebas de extensión) y agente evaluado (extensión de `ir/program.ir.json` a `spec_version: 2`) en `KADDO-AGENT-NATIVE-POC-001`.
- **Current behavior:** `spec/task.yaml` (`version: 1`) y `ir/program.ir.json` (`spec_version: 1`) sólo definen y exportan `calculate_shipping(subtotal_cents: i32) -> i32`. Aún no se ha evaluado la subhipótesis **H1-C** (**SC-06** y **SC-07**): extender el programa existente con una segunda función exportada (`calculate_discount(subtotal_cents: i32, premium: bool) -> i32`) preservando `calculate_shipping` y sin modificar el backend congelado en `poc-e1-freeze`.
- **Target behavior:**
  1. El operador actualiza `spec/task.yaml` a `version: 2` y `tests/public.yaml` (6 pruebas de regresión de `calculate_shipping` + 6 pruebas públicas nuevas de `calculate_discount`, total `12` pruebas públicas), y prepara el conjunto de pruebas reservadas de `E3` (9 pruebas reservadas de `calculate_shipping` + pruebas reservadas de `calculate_discount`).
  2. Se guarda `runs/e3-extend/attempt-01/ir.before.json` (`ir/program.ir.json` v1) y `runs/e3-extend/attempt-01/spec.snapshot.yaml` (spec v2).
  3. El agente extiende `ir/program.ir.json` a `spec_version: 2` añadiendo la función exportada `calculate_discount(subtotal_cents: i32, premium: bool) -> i32` con trazabilidad `["DISC-001", "DISC-002", "DISC-003"]`:
     - `subtotal_cents < 0` → `-1` (`DISC-001`)
     - `premium == false` (o rama `else` de `if (premium)`) → `0` (`DISC-002`)
     - `premium == true` → `subtotal_cents / 10` mediante `div_s` (`DISC-003`)
  4. `irtool verify` aprueba con `0` diagnósticos (`diagnostics.json = []`), `irtool compile` + `irtool validate` emiten un módulo `.wasm` válido con ambas funciones exportadas, `runner test` aprueba el `100 %` de pruebas públicas (`12/12`) y reservadas (**SC-06**), y los hashes de `irtool` y `runner` coinciden exactamente con `runs/toolchain-freeze.sha256` (**SC-07**).
- **Observable completion:**
  1. `diagnostics.json` contiene `[]` (`exit 0`).
  2. `public-tests.json` aprueba `12/12` casos (`6` de `calculate_shipping` + `6` de `calculate_discount`) y `hidden-tests.json` aprueba el `100 %` de los casos reservados.
  3. `runs/e3-extend/attempt-01/` contiene todos los artefactos obligatorios de `E3` y confirma `backend_modified_since_freeze: false`.

## Journey reconstruction

```text
Operador publica spec/task.yaml (v2) + tests/public.yaml (12 casos)
   ↓
Snapshot de ir/program.ir.json (v1) en runs/e3-extend/attempt-01/ir.before.json
   ↓
Agente extiende ir/program.ir.json (spec_version: 2, añade calculate_discount conservando calculate_shipping)
   ↓
irtool verify (exit 0, []) → irtool compile → irtool validate (SC-10)
   ↓
runner test (12/12 públicas + 100 % reservadas) (SC-06)
   ↓
Comprobación de hashes de irtool/runner contra runs/toolchain-freeze.sha256 (SC-07) + evidencia en runs/e3-extend/attempt-01/
```

## Problem

Para validar la subhipótesis **H1-C** (*Extensión funcional sin tocar el backend*), es necesario comprobar que una nueva regla de negocio con firma multi-parámetro (`(i32, bool) -> i32`) y división entera (`div_s`) puede expresarse íntegramente en la Semantic IR v0.1 y compilarse a `.wasm` mediante el backend congelado en `E1`, sin romper la función `calculate_shipping` existente (**SC-06**, **SC-07**).

_Expected value:_ Evidencia auditable para **H1-C**, **SC-06**, **SC-07**, **SC-10**, **SC-11**, **SC-12**, **SC-14** y **SC-16**.

## Scope

- Actualizar `spec/task.yaml` a `version: 2` añadiendo `calculate_discount(subtotal_cents: i32, premium: bool) -> i32` (`DISC-001`, `DISC-002`, `DISC-003`) y los 6 casos públicos `E3-P01` a `E3-P06`.
- Actualizar `tests/public.yaml` con los 6 casos de `calculate_shipping` (regresión de `E1`) y los 6 casos nuevos de `calculate_discount` (`12` casos en total).
- Preparar la suite reservada de `E3` fuera del directorio expuesto del repositorio (`scratch/e3-hidden.yaml`, con los 9 casos de `calculate_shipping` + casos reservados de borde de `calculate_discount`).
- Extender `ir/program.ir.json` a `spec_version: 2` agregando `calculate_discount` sin alterar la definición semántica de `calculate_shipping`.
- Ejecutar `verify → compile → validate → test` (públicas + reservadas) y comprobar que los hashes del toolchain coinciden con `runs/toolchain-freeze.sha256`.
- Registrar toda la evidencia en `runs/e3-extend/attempt-01/`.

## Out of scope

- Cambio de política en `calculate_shipping` (`>= 4000 -> 0`, `< 4000 -> 600`) → pertenece a **`WI-006` (`E4 — Mantenimiento`)**.
- Cualquier modificación a `crates/irtool` o `crates/runner` (congelados desde `poc-e1-freeze`).

## Surface review

- **Product/UI:** not-applicable.
- **Frontend:** not-applicable.
- **Backend:** reviewed-not-affected (congelado desde `poc-e1-freeze`).
- **Database:** not-applicable.
- **Configuration:** affected (`spec/task.yaml` v2, `tests/public.yaml`, `ir/program.ir.json` v2).
- **Feature flags / Auth / Notifications / Analytics:** not-applicable.
- **Documentation:** reviewed (`runs/e3-extend/attempt-01/agent-summary.md`, `knowledge/delivery/poc.md`, `knowledge/tech/current-state.md`).
- **Operations/release:** reviewed-not-affected.

## Acceptance Criteria

- [ ] `spec/task.yaml` está actualizado a `version: 2` con las funciones exportadas `calculate_shipping` y `calculate_discount` (`DISC-001`, `DISC-002`, `DISC-003`).
- [ ] `ir/program.ir.json` (`spec_version: 2`) incluye `calculate_discount(subtotal_cents: i32, premium: bool) -> i32` conservando intacta `calculate_shipping` y pasa `irtool verify` con `0` diagnósticos (`diagnostics.json = []`).
- [ ] El módulo `program.wasm` compilado pasa `irtool validate` y aprueba `12/12` pruebas públicas (`6` nuevas de `calculate_discount` + `6` de regresión de `calculate_shipping`) y `100 %` de las pruebas reservadas (**SC-06**, **SC-10**).
- [ ] Los hashes SHA-256 de `irtool` y `runner` coinciden exactamente con `runs/toolchain-freeze.sha256` (`backend_modified_since_freeze: false`, **SC-07**).
- [ ] **End-to-end:** Existen todos los artefactos obligatorios de `E3` en `runs/e3-extend/attempt-01/` (`run.json`, `toolchain.json`, `spec.snapshot.yaml`, `ir.before.json`, `ir.after.json`, `diagnostics.json`, `public-tests.json`, `hidden-tests.json`, `program.wasm`, `program.wat`, `artifact.sha256`, `diff.patch`, `agent-summary.md`) (**SC-14**).

## Validation

1. Ejecutar `.\target\debug\irtool.exe verify --schema ir/ir.schema.json --ir ir/program.ir.json --diagnostics runs/e3-extend/attempt-01/diagnostics.json` (`exit 0`).
2. Ejecutar `.\target\debug\irtool.exe compile --ir ir/program.ir.json --output build/program.wasm --wat build/program.wat` y `.\target\debug\irtool.exe validate build/program.wasm` (`exit 0`).
3. Ejecutar `.\target\debug\runner.exe test` contra `tests/public.yaml` (`12/12` aprobadas) y la suite reservada de `E3` (`100 %` aprobadas).
4. Verificar que los hashes SHA-256 de `target/debug/irtool.exe` y `target/debug/runner.exe` coinciden con `runs/toolchain-freeze.sha256`.

## Definition of Done

- Todos los criterios de aceptación de `WI-005` (**SC-06**, **SC-07**, **SC-10**, **SC-11**, **SC-12**, **SC-14**, **SC-16**) están cumplidos y respaldados en `runs/e3-extend/attempt-01/`.
- `SC-06` y `SC-07` están marcados como verificados `[x]` en `knowledge/delivery/poc.md`.
- `knowledge/tech/current-state.md` y `.kaddo/` están actualizados.

## Open questions

- Ninguna pregunta abierta bloqueante (`resolution_status = open`).

## Dependencies

- Depende de `WI-004` (`E2 reparación controlada`, completado).
- Bloquea `WI-006` (`E4 mantenimiento`).
