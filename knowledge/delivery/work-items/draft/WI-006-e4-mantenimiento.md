---
type: spike
id: WI-006
alias: KAN-POC-WI-006
title: E4 mantenimiento
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
  - runs/e4-maintain/**
---

# WI-006 (KAN-POC-WI-006) — E4 mantenimiento

## Outcome

- **Actor:** Operador del experimento (publicación de `spec/task.yaml` v3 y pruebas actualizadas) y agente evaluado (mantenimiento localizado de `ir/program.ir.json` a `spec_version: 3`) en `KADDO-AGENT-NATIVE-POC-001`.
- **Current behavior:** `spec/task.yaml` (`version: 2`) e `ir/program.ir.json` (`spec_version: 2`) implementan la política original de `calculate_shipping` (`>= 5000 -> 0`, `< 5000 -> 500`) junto con `calculate_discount`. Aún no se ha evaluado la subhipótesis **H1-D** (**SC-08** y **SC-09**): cambiar una política existente en `calculate_shipping` (`>= 4000 -> 0`, `< 4000 -> 600`) mediante un diff localizado en la IR sin alterar `calculate_discount` ni modificar el backend congelado en `poc-e1-freeze`.
- **Target behavior:**
  1. El operador actualiza `spec/task.yaml` a `version: 3` y `tests/public.yaml` con la nueva política de `calculate_shipping` (`SHIP-001`: `< 0 -> -1`, `SHIP-002`: `>= 4000 -> 0`, `SHIP-003`: `< 4000 -> 600`) más todas las pruebas de regresión de `calculate_discount`, y prepara el conjunto de pruebas reservadas de `E4`.
  2. Se guarda `runs/e4-maintain/attempt-01/ir.before.json` (`ir/program.ir.json` v2) y `runs/e4-maintain/attempt-01/spec.snapshot.yaml` (spec v3).
  3. El agente actualiza `ir/program.ir.json` a `spec_version: 3`, modificando únicamente los nodos afectados de `calculate_shipping` (`5000 -> 4000` y `500 -> 600`) y dejando `calculate_discount` intacta.
  4. `irtool verify` aprueba con `0` diagnósticos (`diagnostics.json = []`), `irtool compile` + `irtool validate` emiten un módulo `.wasm` válido, `runner test` aprueba el `100 %` de pruebas públicas actualizadas + regresión de descuento y el `100 %` de pruebas reservadas (**SC-08**), los hashes de `irtool` y `runner` coinciden exactamente con `runs/toolchain-freeze.sha256` (**SC-09**), y una segunda compilación del mismo IR produce exactamente el mismo hash SHA-256 (**SC-11**).
- **Observable completion:**
  1. `diagnostics.json` contiene `[]` (`exit 0`).
  2. `public-tests.json` y `hidden-tests.json` aprueban al `100 %`.
  3. `diff.patch` demuestra que el cambio en `ir/program.ir.json` está localizado exclusivamente en `spec_version` y en los literales `4000`/`600` de `calculate_shipping`.
  4. `runs/e4-maintain/attempt-01/` contiene todos los artefactos obligatorios de `E4` y confirma `backend_modified_since_freeze: false`.

## Journey reconstruction

```text
Operador publica spec/task.yaml (v3) + tests/public.yaml (shipping v3 + regresión discount)
   ↓
Snapshot de ir/program.ir.json (v2) en runs/e4-maintain/attempt-01/ir.before.json
   ↓
Agente actualiza ir/program.ir.json (spec_version: 3, umbral 5000→4000 y tarifa 500→600 en calculate_shipping; calculate_discount intacta)
   ↓
irtool verify (exit 0, []) → irtool compile → irtool validate (SC-10, SC-11)
   ↓
runner test (100 % públicas actualizadas + regresión discount + 100 % reservadas) (SC-08)
   ↓
Comprobación de hashes de irtool/runner contra runs/toolchain-freeze.sha256 (SC-09) + evidencia en runs/e4-maintain/attempt-01/
```

## Problem

Para validar la subhipótesis **H1-D** (*Mantenimiento localizado de reglas existentes*), es necesario demostrar que un cambio de política de negocio en `calculate_shipping` se traduce en una modificación mínima, trazable y aislada en `ir/program.ir.json`, sin regresiones en `calculate_discount` y sin tocar el backend congelado en `E1` (**SC-08**, **SC-09**).

_Expected value:_ Completar la serie experimental `E1`–`E4` con evidencia auditable para **H1-D**, **SC-08**, **SC-09**, **SC-10**, **SC-11**, **SC-12**, **SC-13**, **SC-14**, **SC-15** y **SC-16**.

## Scope

- Actualizar `spec/task.yaml` a `version: 3` con la nueva política de `calculate_shipping` (`>= 4000 -> 0`, `< 4000 -> 600`) y los casos públicos `E4-P01`..`E4-P06` más la regresión completa de `calculate_discount`.
- Actualizar `tests/public.yaml` (`spec_version: 3`) y preparar la suite reservada `scratch/e4-hidden.yaml`.
- Modificar `ir/program.ir.json` a `spec_version: 3` actualizando únicamente `calculate_shipping` (`5000 -> 4000`, `500 -> 600`) sin alterar `calculate_discount`.
- Ejecutar `verify → compile → validate → test` (públicas + reservadas), verificar determinismo de compilación (**SC-11**) y comprobar que los hashes del toolchain coinciden con `runs/toolchain-freeze.sha256` (**SC-09**).
- Registrar toda la evidencia en `runs/e4-maintain/attempt-01/`.

## Out of scope

- Consolidación final de la matriz `SC-01..SC-16`, cierre de las 10 preguntas obligatorias y generación del reporte final (`kaddo poc report`) → pertenecen a **`WI-007` (`KAN-POC-WI-007` — Análisis y conclusión)**.
- Cualquier modificación a `crates/irtool` o `crates/runner` (congelados desde `poc-e1-freeze`).

## Surface review

- **Product/UI:** not-applicable.
- **Frontend:** not-applicable.
- **Backend:** reviewed-not-affected (congelado desde `poc-e1-freeze`).
- **Database:** not-applicable.
- **Configuration:** affected (`spec/task.yaml` v3, `tests/public.yaml`, `ir/program.ir.json` v3).
- **Feature flags / Auth / Notifications / Analytics:** not-applicable.
- **Documentation:** reviewed (`runs/e4-maintain/attempt-01/agent-summary.md`, `knowledge/delivery/poc.md`, `knowledge/tech/current-state.md`).
- **Operations/release:** reviewed-not-affected.

## Acceptance Criteria

- [ ] `spec/task.yaml` está actualizado a `version: 3` con la nueva política de `calculate_shipping` (`>= 4000 -> 0`, `< 4000 -> 600`) y `calculate_discount` sin cambios.
- [ ] `ir/program.ir.json` (`spec_version: 3`) modifica únicamente `calculate_shipping` preservando intacta `calculate_discount` y pasa `irtool verify` con `0` diagnósticos (`diagnostics.json = []`).
- [ ] El módulo `program.wasm` compilado pasa `irtool validate` y aprueba el `100 %` de las pruebas públicas actualizadas + regresión de descuento y el `100 %` de las pruebas reservadas de `E4` (**SC-08**, **SC-10**).
- [ ] Compilar el mismo `ir/program.ir.json` dos veces en el mismo entorno produce exactamente el mismo hash SHA-256 (**SC-11**).
- [ ] Los hashes SHA-256 de `irtool` y `runner` coinciden exactamente con `runs/toolchain-freeze.sha256` (`backend_modified_since_freeze: false`, **SC-09**).
- [ ] **End-to-end:** Existen todos los artefactos obligatorios de `E4` en `runs/e4-maintain/attempt-01/` (`run.json`, `toolchain.json`, `spec.snapshot.yaml`, `ir.before.json`, `ir.after.json`, `diagnostics.json`, `public-tests.json`, `hidden-tests.json`, `program.wasm`, `program.wat`, `artifact.sha256`, `diff.patch`, `agent-summary.md`) (**SC-14**).

## Validation

1. Ejecutar `.\target\debug\irtool.exe verify --schema ir/ir.schema.json --ir ir/program.ir.json --diagnostics runs/e4-maintain/attempt-01/diagnostics.json` (`exit 0`).
2. Ejecutar `.\target\debug\irtool.exe compile --ir ir/program.ir.json --output build/program.wasm --wat build/program.wat` dos veces y comprobar que el hash SHA-256 de `build/program.wasm` es idéntico (**SC-11**), además de `.\target\debug\irtool.exe validate build/program.wasm` (`exit 0`).
3. Ejecutar `.\target\debug\runner.exe test` contra `tests/public.yaml` y la suite reservada de `E4` (`100 %` aprobadas).
4. Verificar que los hashes SHA-256 de `target/debug/irtool.exe` y `target/debug/runner.exe` coinciden con `runs/toolchain-freeze.sha256`.

## Definition of Done

- Todos los criterios de aceptación de `WI-006` (**SC-08** a **SC-16**) están cumplidos y respaldados en `runs/e4-maintain/attempt-01/`.
- `SC-08` a `SC-16` están marcados como verificados `[x]` en `knowledge/delivery/poc.md`.
- `knowledge/tech/current-state.md` y `.kaddo/` están actualizados.

## Open questions

- Ninguna pregunta abierta bloqueante (`resolution_status = open`).

## Dependencies

- Depende de `WI-005` (`E3 extensión funcional`, completado).
- Bloquea `WI-007` (`Análisis y conclusión`).
