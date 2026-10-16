---
type: spike
id: WI-004
alias: KAN-POC-WI-004
title: E2 reparación controlada
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
  - deterministic-toolchain
  - experiment-evidence
code:
  - ir/program.ir.json
  - runs/e2-repair/**
---

# WI-004 (KAN-POC-WI-004) — E2 reparación controlada

## Outcome

- **Actor:** Operador del experimento (fault injection) y agente evaluado (reparación guiada por diagnósticos) en `KADDO-AGENT-NATIVE-POC-001`.
- **Current behavior:** `ir/program.ir.json` contiene la versión válida de `E1` (`calculate_shipping`), y aún no se ha puesto a prueba la subhipótesis **H1-B** (**SC-04** y **SC-05**): que un defecto sembrado en la IR sea rechazado determinísticamente por `irtool verify` con código y ruta lógica (`diagnostics.json`), y que el agente repare `ir/program.ir.json` en máximo 2 intentos recuperando el 100 % de las pruebas de `E1` sin tocar el backend congelado.
- **Target behavior:**
  1. Se guarda `runs/e2-repair/attempt-01/ir.before.json` (IR válida de `E1`) y se inyecta un defecto controlado de los candidatos de `poc.md` (`E_UNKNOWN_PARAM`: referencia al parámetro inexistente `subtotal` en lugar de `subtotal_cents`), guardando `runs/e2-repair/attempt-01/ir.injected.json`.
  2. Se ejecuta `irtool verify` sobre la IR defectuosa, confirmando rechazo con `exit != 0` y diagnóstico estructurado (`code: "E_UNKNOWN_PARAM"`, `function: "calculate_shipping"`, `path: "$.functions[0].body.condition.left"`) en `runs/e2-repair/attempt-01/diagnostics.injected.json` (**SC-04**).
  3. El agente recibe la IR defectuosa y el diagnóstico, repara `ir/program.ir.json` en `1` intento (`<= 2`, **SC-05**), vuelve a pasar `verify` (`diagnostics.json = []`), `compile`, `validate` y aprueba `6/6` pruebas públicas y `9/9` pruebas reservadas de `E1`, comprobando además que los hashes de `irtool` y `runner` coinciden exactamente con `runs/toolchain-freeze.sha256`.
- **Observable completion:**
  1. `diagnostics.injected.json` demuestra el rechazo determinista con `E_UNKNOWN_PARAM`.
  2. `diagnostics.json` posterior a la reparación es `[]` (`exit 0`).
  3. `public-tests.json` (`6/6`) y `hidden-tests.json` (`9/9`) aprueban al `100 %`, y `artifact.sha256` coincide con el `.wasm` válido de `E1`.
  4. Todos los artefactos de evidencia de `E2` existen en `runs/e2-repair/attempt-01/`.

## Journey reconstruction

```text
ir/program.ir.json (válida de E1) → copia en runs/e2-repair/attempt-01/ir.before.json
   ↓
Operador inyecta defecto controlado (E_UNKNOWN_PARAM: param "subtotal") → ir.injected.json
   ↓
irtool verify (rechaza con exit != 0) → diagnostics.injected.json (SC-04)
   ↓
Agente lee ir.injected.json + diagnostics.injected.json → repara ir/program.ir.json (SC-05)
   ↓
irtool verify (exit 0, []) → irtool compile → irtool validate → runner test (6/6 públicas + 9/9 reservadas)
   ↓
Verificación de hashes intactos respecto a runs/toolchain-freeze.sha256 + evidencia en runs/e2-repair/attempt-01/
```

## Problem

Para validar la subhipótesis **H1-B** (*Diagnósticos deterministas son suficientes para orientar una reparación*), es necesario demostrar empíricamente que el verificador rechaza una IR corrupta con ubicación lógica exacta (**SC-04**) y que el agente corrige el defecto en `ir/program.ir.json` dentro del límite de 2 intentos recuperando el 100 % del comportamiento de `E1` (**SC-05**).

_Expected value:_ Evidencia auditable para **H1-B**, **SC-04**, **SC-05** y **SC-14** sin alterar el backend congelado en `E1`.

## Scope

- Guardar `runs/e2-repair/attempt-01/ir.before.json` y `spec.snapshot.yaml`.
- Inyectar un único defecto controlado (`E_UNKNOWN_PARAM`: cambiar `"name": "subtotal_cents"` por `"name": "subtotal"` en `$.functions[0].body.condition.left`) y guardarlo en `runs/e2-repair/attempt-01/ir.injected.json`.
- Ejecutar `irtool verify` contra la IR inyectada, confirmar `exit != 0` y guardar el diagnóstico emitido en `runs/e2-repair/attempt-01/diagnostics.injected.json`.
- Reparar `ir/program.ir.json` a partir del diagnóstico, guardar `runs/e2-repair/attempt-01/ir.after.json` y `diff.patch` (diff entre `ir.injected.json` e `ir.after.json`).
- Ejecutar `verify → compile → validate → test` (públicas `6/6` + reservadas `9/9`) y comprobar que los hashes del toolchain coinciden con `runs/toolchain-freeze.sha256`.
- Registrar `run.json`, `toolchain.json`, `agent-summary.md` y demás artefactos en `runs/e2-repair/attempt-01/`.

## Out of scope

- Extensión funcional con `calculate_discount` (spec v2) → pertenece a **`WI-005` (`E3 — Extensión funcional`)**.
- Modificación de política de shipping (spec v3) → pertenece a **`WI-006` (`E4 — Mantenimiento`)**.
- Cualquier cambio al código o binarios de `irtool` y `runner` (congelados tras `E1`).

## Surface review

- **Product/UI:** not-applicable.
- **Frontend:** not-applicable.
- **Backend:** reviewed-not-affected (congelado desde `poc-e1-freeze`).
- **Database:** not-applicable.
- **Configuration:** affected (`ir/program.ir.json` durante inyección y reparación).
- **Feature flags / Auth / Notifications / Analytics:** not-applicable.
- **Documentation:** reviewed (`runs/e2-repair/attempt-01/agent-summary.md`, `knowledge/delivery/poc.md`, `knowledge/tech/current-state.md`).
- **Operations/release:** reviewed-not-affected.

## Acceptance Criteria

- [x] Existe `runs/e2-repair/attempt-01/ir.injected.json` con un único defecto sembrado (`E_UNKNOWN_PARAM`) sobre la IR de `E1`.
- [x] `irtool verify` rechaza `ir.injected.json` con `exit != 0` y emite en `diagnostics.injected.json` el código `E_UNKNOWN_PARAM`, la función `calculate_shipping` y la ruta lógica `$.functions[0].body.condition.left` (**SC-04**).
- [x] El agente repara `ir/program.ir.json` en máximo `2` intentos (`repair_attempts <= 2`) sin edición manual humana y `irtool verify` aprueba con `0` diagnósticos (**SC-05**).
- [x] Tras la reparación, `6/6` pruebas públicas de `E1` y `9/9` (`100 %`) pruebas reservadas vuelven a pasar en Wasmtime (**SC-05**, **SC-10**).
- [x] Los hashes SHA-256 de `irtool` y `runner` coinciden exactamente con `runs/toolchain-freeze.sha256` (`backend_modified_since_freeze: false`).
- [x] **End-to-end:** Existen todos los artefactos obligatorios de `E2` en `runs/e2-repair/attempt-01/` (`run.json`, `toolchain.json`, `spec.snapshot.yaml`, `ir.before.json`, `ir.injected.json`, `ir.after.json`, `diagnostics.injected.json`, `diagnostics.json`, `public-tests.json`, `hidden-tests.json`, `program.wasm`, `program.wat`, `artifact.sha256`, `diff.patch`, `agent-summary.md`) (**SC-14**).

## Validation

1. Verificar que `irtool verify` rechaza `runs/e2-repair/attempt-01/ir.injected.json` (`exit != 0`) con `E_UNKNOWN_PARAM`.
2. Verificar que `irtool verify` aprueba `ir/program.ir.json` reparada (`exit 0`, `diagnostics.json = []`).
3. Ejecutar `irtool compile`, `irtool validate` y `runner test` sobre `tests/public.yaml` (`6/6`) y las pruebas reservadas (`9/9`).
4. Comprobar que los hashes actuales de `irtool` y `runner` coinciden con `runs/toolchain-freeze.sha256`.

## Definition of Done

- Todos los criterios de aceptación de `WI-004` (**SC-04**, **SC-05**, **SC-10**, **SC-12**, **SC-14**, **SC-16**) están cumplidos y respaldados en `runs/e2-repair/attempt-01/`.
- `SC-04` y `SC-05` están marcados como verificados `[x]` en `knowledge/delivery/poc.md`.
- `knowledge/tech/current-state.md` y `.kaddo/` están actualizados.

## Open questions

- Ninguna pregunta abierta bloqueante (`resolution_status = open`).

## Dependencies

- Depende de `WI-003` (`E1 construcción y freeze`, completado).
- Bloquea `WI-005` (`E3 extensión funcional`).
