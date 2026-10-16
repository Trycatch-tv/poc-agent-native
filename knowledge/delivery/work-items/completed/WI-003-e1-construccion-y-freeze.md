---
type: spike
id: WI-003
alias: KAN-POC-WI-003
title: E1 construcción y freeze
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
  - ir/program.ir.json
  - runs/e1-build/**
  - runs/toolchain-freeze.sha256
---

# WI-003 (KAN-POC-WI-003) — E1 construcción y freeze

## Outcome

- **Actor:** Agente evaluado y operador del experimento en `KADDO-AGENT-NATIVE-POC-001`.
- **Current behavior:** `ir/program.ir.json` contiene únicamente el smoke test neutral `identity(value: i32) -> i32` (`SMOKE-001`), y no existe evidencia de ejecución de `E1` en `runs/e1-build/` ni punto de congelamiento del toolchain (`runs/toolchain-freeze.sha256`).
- **Target behavior:** El agente lee `spec/task.yaml` (v1) y el contrato `ir/ir.schema.json`, construye en `ir/program.ir.json` la función exportada `calculate_shipping(subtotal_cents: i32) -> i32` referenciando `SHIP-001`, `SHIP-002` y `SHIP-003`, aprueba la verificación (`irtool verify`), compila el binario (`irtool compile`), pasa la validación WASM (`irtool validate`) y el 100 % de las pruebas públicas (`6/6`) y reservadas, registra todos los artefactos de evidencia en `runs/e1-build/attempt-01/` y captura los hashes SHA-256 del toolchain congelado en `runs/toolchain-freeze.sha256`.
- **Observable completion:**
  1. `ir/program.ir.json` es verificado con `0` diagnósticos (`exit 0`) y compilado a `.wasm` válido.
  2. `tests/public.yaml` pasa `6/6` casos en Wasmtime y las pruebas reservadas pasan al `100 %`.
  3. Existe `runs/e1-build/attempt-01/` con todos los artefactos obligatorios (`run.json`, `spec.snapshot.yaml`, `ir.before.json`, `ir.after.json`, `diagnostics.json`, `public-tests.json`, `hidden-tests.json`, `artifact.sha256`, `program.wasm`, `program.wat`, `agent-summary.md`, `toolchain.json`).
  4. Existe `runs/toolchain-freeze.sha256` con los hashes SHA-256 de los binarios/fuentes de `irtool` y `runner` que quedarán congelados para `E2`–`E4`.

## Journey reconstruction

```text
spec/task.yaml (v1) + ir/ir.schema.json + ir/program.ir.json (before: identity)
   ↓
Agente construye ir/program.ir.json (calculate_shipping, requirements: SHIP-001..003)
   ↓
irtool verify → build/diagnostics.json = [] (máx. 2 reintentos si hubiera error)
   ↓
irtool compile → build/program.wasm + build/program.wat
   ↓
irtool validate → build/program.wasm válido
   ↓
runner test → 6/6 casos públicos + 100 % casos reservados aprobados
   ↓
Captura de evidencia en runs/e1-build/attempt-01/ + runs/toolchain-freeze.sha256
```

## Problem

Para validar las subhipótesis **H1-A** (construcción inicial de IR desde especificación) y establecer el **toolchain freeze point** requerido por **H1-C** (**SC-07** / **SC-09**), el agente debe transformar `spec/task.yaml` v1 en una `Semantic IR v0.1` válida sin ediciones manuales humanas (**SC-02**) ni implementación paralela en lenguajes tradicionales (**SC-13**).

_Expected value:_ Demostrar con evidencia auditable (`SC-01`, `SC-02`, `SC-03`, `SC-10`, `SC-14`, `SC-15`) que el agente puede sintetizar el programa inicial en Semantic IR v0.1 y dejar congelada la toolchain determinista para `E2`–`E4`.

## Scope

- Construir en `ir/program.ir.json` (`spec_version: 1`, `ir_version: "0.1"`) la función exportada `calculate_shipping(subtotal_cents: i32) -> i32` conforme a `spec/task.yaml` v1:
  - `subtotal_cents < 0` → `-1` (`SHIP-001`)
  - `subtotal_cents >= 5000` → `0` (`SHIP-002`)
  - en otro caso → `500` (`SHIP-003`)
- Ejecutar el pipeline determinista completo y registrar el paquete de evidencia en `runs/e1-build/attempt-01/`.
- Generar `runs/toolchain-freeze.sha256` con los hashes SHA-256 del toolchain compilado (`irtool` y `runner` y sus fuentes) como punto de congelamiento oficial tras `E1`.

## Out of scope

- Inyección de defectos y reparación guiada por diagnósticos → pertenece a **`WI-004` (`E2 — Diagnóstico y reparación`)**.
- Extensión con `calculate_discount` (spec v2) → pertenece a **`WI-005` (`E3 — Extensión funcional`)**.
- Cambio de política en `calculate_shipping` (spec v3) → pertenece a **`WI-006` (`E4 — Mantenimiento`)**.
- Cualquier modificación a `crates/irtool` o `crates/runner` una vez establecido el freeze point.

## Surface review

- **Product/UI:** not-applicable.
- **Frontend:** not-applicable.
- **Backend:** reviewed-not-affected (`crates/irtool` y `crates/runner` se usan tal como fueron entregados en `WI-002` y se congelan al cierre de `E1`).
- **Database:** not-applicable.
- **Configuration:** affected (`ir/program.ir.json` pasa del smoke test `identity` a la implementación v1 de `calculate_shipping`).
- **Feature flags / Auth / Notifications / Analytics:** not-applicable.
- **Documentation:** reviewed (`runs/e1-build/attempt-01/agent-summary.md` y actualización de estado en `knowledge/tech/current-state.md`).
- **Operations/release:** affected — generación de `runs/toolchain-freeze.sha256` (freeze point).

## Acceptance Criteria

- [x] `ir/program.ir.json` define `calculate_shipping(subtotal_cents: i32) -> i32` (`export: true`) con `requirements: ["SHIP-001", "SHIP-002", "SHIP-003"]` sin ediciones manuales humanas (**SC-02**, **SC-15**).
- [x] `cargo run -p irtool -- verify --schema ir/ir.schema.json --ir ir/program.ir.json --diagnostics build/diagnostics.json` aprueba con `0` diagnósticos (`exit 0`) dentro del límite de intentos (**SC-01**, **SC-03**).
- [x] `cargo run -p irtool -- compile` y `cargo run -p irtool -- validate` generan y validan `build/program.wasm` sin errores (**SC-10**, **SC-12**).
- [x] `cargo run -p runner -- test` aprueba `6/6` casos públicos (`SHIP-P01` a `SHIP-P06`) y `100 %` de los casos reservados de `E1` (**SC-01**).
- [x] Existen todos los artefactos de evidencia obligatorios en `runs/e1-build/attempt-01/` (`run.json`, `toolchain.json`, `spec.snapshot.yaml`, `ir.before.json`, `ir.after.json`, `diagnostics.json`, `public-tests.json`, `hidden-tests.json`, `artifact.sha256`, `program.wasm`, `program.wat`, `diff.patch`, `agent-summary.md`) (**SC-14**).
- [x] **End-to-end:** Existe `runs/toolchain-freeze.sha256` registrando los hashes SHA-256 de `irtool` y `runner` para auditar la inmutabilidad del backend en `E2`–`E4` (**SC-07**, **SC-09**).

## Validation

1. Ejecutar verificación, compilación y validación:
   - `cargo run -p irtool -- verify --schema ir/ir.schema.json --ir ir/program.ir.json --diagnostics runs/e1-build/attempt-01/diagnostics.json`
   - `cargo run -p irtool -- compile --ir ir/program.ir.json --output runs/e1-build/attempt-01/program.wasm --wat runs/e1-build/attempt-01/program.wat`
   - `cargo run -p irtool -- validate runs/e1-build/attempt-01/program.wasm`
2. Ejecutar pruebas públicas:
   - `cargo run -p runner -- test --wasm runs/e1-build/attempt-01/program.wasm --cases tests/public.yaml --output runs/e1-build/attempt-01/public-tests.json`
3. Verificar que `runs/e1-build/attempt-01/run.json` reporta `"outcome": "passed"`, `"ir_valid": true`, `"wasm_valid": true`, `public_tests.passed == 6` y `hidden_tests.failed == 0`.
4. Verificar que `runs/toolchain-freeze.sha256` existe y contiene los hashes SHA-256 de los binarios/fuentes del backend.

## Definition of Done

- Todos los criterios de aceptación de `WI-003` (`SC-01`, `SC-02`, `SC-03`, `SC-10`, `SC-12`, `SC-13`, `SC-14`, `SC-15`, `SC-16` en lo aplicable a `E1`) están cumplidos y respaldados por evidencia en `runs/e1-build/attempt-01/`.
- El freeze point del toolchain (`runs/toolchain-freeze.sha256`) está capturado.
- `knowledge/tech/current-state.md` y el contexto de Kaddo están sincronizados.

## Open questions

- Ninguna pregunta abierta bloqueante (`resolution_status = open`).

## Dependencies

- Depende de `WI-001` y `WI-002` (ambos completados).
- Bloquea `WI-004` (`E2 — Diagnóstico y reparación`), `WI-005` (`E3 — Extensión`) y `WI-006` (`E4 — Mantenimiento`).

## Learning

_Capturado al completar `WI-003` (2026-10-16):_

- **Qué se entregó:** Construcción autónoma de `calculate_shipping(subtotal_cents: i32) -> i32` en `ir/program.ir.json` a partir de `spec/task.yaml` v1, paquete completo de evidencia en `runs/e1-build/attempt-01/` (`76 bytes` `.wasm`, `sha256=026cbbc30f6252a535cab1b1608df265f7641ff0ef5aae1b22fd36e8e3dee505`, `6/6` pruebas públicas y `9/9` pruebas reservadas aprobadas en el intento 1 con `0` diagnósticos) y congelamiento del toolchain en `runs/toolchain-freeze.sha256` (tag `poc-e1-freeze`).
- **Qué se aprendió / decisiones emergentes:**
  1. La subhipótesis **H1-A** y los criterios **SC-01**, **SC-02** y **SC-03** quedaron validados en el primer intento sin requerir reintentos de reparación ni edición manual.
  2. El toolchain queda estrictamente congelado (`runs/toolchain-freeze.sha256`) a partir de este commit; cualquier cambio a `crates/irtool` o `crates/runner` durante `E2`–`E4` está prohibido por **SC-07** y **SC-09**.
- **Conocimiento actualizado:** `knowledge/delivery/poc.md` (`SC-01`..`SC-03` marcados), `knowledge/tech/current-state.md` y `.kaddo/`.
- **Pendientes:** Ejecutar `WI-004` (`E2 — Reparación controlada`).
