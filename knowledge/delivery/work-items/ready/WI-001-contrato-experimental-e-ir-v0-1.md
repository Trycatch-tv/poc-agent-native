---
type: spike
id: WI-001
alias: KAN-POC-WI-001
title: Contrato experimental e IR v0.1
knowledge_level: K2
status: ready
phase: now
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
domains:
  - semantic-ir
  - commercial-rules-subject
  - experiment-evidence
code:
  - spec/**
  - ir/**
  - tests/**
  - runs/**
  - build/**
ready_at: '2026-10-16'
---

# WI-001 (KAN-POC-WI-001) — Contrato experimental e IR v0.1

## Outcome

- **Actor:** Operador del experimento y agente evaluado en `KADDO-AGENT-NATIVE-POC-001`.
- **Current behavior:** El repositorio cuenta únicamente con la definición documental en `knowledge/delivery/poc.md`, pero aún no existen en el árbol de archivos los directorios experimentales (`spec/`, `ir/`, `tests/`, `runs/`, `build/`) ni los artefactos base (`ir/ir.schema.json`, `spec/task.yaml`, `ir/program.ir.json` de smoke test, `tests/public.yaml`).
- **Target behavior:** Existen en el repositorio la estructura de directorios base, el esquema JSON Draft 2020-12 (`ir/ir.schema.json`), el smoke test neutral (`ir/program.ir.json` con `identity`), la especificación inicial (`spec/task.yaml` v1) y los casos públicos de prueba (`tests/public.yaml`), sin preimplementar la lógica de negocio de `E1`.
- **Observable completion:** `ir/ir.schema.json` e `ir/program.ir.json` son JSON válidos, `spec/task.yaml` y `tests/public.yaml` contienen los contratos de `E1`, y `ir/program.ir.json` contiene únicamente `identity` (`SMOKE-001`).

## Journey reconstruction

```text
knowledge/delivery/poc.md (definición canónica)
   ↓
Materialización de carpetas spec/, ir/, tests/, runs/, build/
   ↓
Creación de ir/ir.schema.json + ir/program.ir.json (smoke test identity)
   ↓
Creación de spec/task.yaml (v1) + tests/public.yaml (casos E1-P01..E1-P06)
   ↓
Base lista para construir el verifier/compiler/runner en WI-002 sin contaminar E1
```

## Problem

Sin los artefactos base del contrato experimental (`ir/ir.schema.json`, `spec/task.yaml` v1, `tests/public.yaml` y el smoke test `ir/program.ir.json`) materializados en el repositorio, no es posible construir ni validar de forma reproducible la toolchain determinista (`WI-002`) ni ejecutar el ejercicio `E1` (`WI-003`).

_Expected value:_ Proveer la base contractual verificable para la toolchain determinista (`WI-002`) y los ejercicios `E1`–`E4` sin violar la restricción de construcción en vivo (**SC-02**).

## Scope

- Crear los directorios `spec/`, `ir/`, `tests/`, `runs/` y `build/`.
- Crear `ir/ir.schema.json` con el JSON Schema Draft 2020-12 de la Semantic IR v0.1.
- Crear `ir/program.ir.json` con únicamente la función neutral `identity(value: i32) -> i32` (`SMOKE-001`).
- Crear `spec/task.yaml` (versión 1) especificando `calculate_shipping(subtotal_cents: i32) -> i32` y sus requisitos `SHIP-001` a `SHIP-003`.
- Crear `tests/public.yaml` con los 6 casos públicos de aceptación de `E1` (`SHIP-P01` a `SHIP-P06`).

## Out of scope

- Implementación en Rust de `irtool` (`verifier` + `compiler`) y `runner` → pertenece a **`WI-002` (`KAN-POC-WI-002`)**.
- Implementación de `calculate_shipping` en `ir/program.ir.json` → pertenece a **`WI-003` (`KAN-POC-WI-003`, ejercicio `E1`)**.
- Vectores secretos de pruebas reservadas dentro del contexto visible al agente.

## Surface review

- **Product/UI:** not-applicable (experimento CLI/WASM sin interfaz gráfica).
- **Frontend:** not-applicable.
- **Backend:** reviewed-not-affected (los binarios Rust `irtool` y `runner` pertenecen a `WI-002`).
- **Database:** not-applicable.
- **Configuration:** affected (`ir/ir.schema.json`, `spec/task.yaml`, `tests/public.yaml`).
- **Feature flags / Auth / Notifications / Analytics:** not-applicable.
- **Documentation:** reviewed (`knowledge/tech/current-state.md` actualizado).
- **Operations/release:** reviewed-not-affected.

## Acceptance Criteria

- [x] Existen en la raíz del proyecto los directorios `spec/`, `ir/`, `tests/`, `runs/` y `build/`.
- [x] Existe `ir/ir.schema.json` con JSON válido conforme a Draft 2020-12 y con las definiciones `valueType`, `param`, `function`, `expr`, `constExpr`, `paramExpr`, `unaryExpr`, `binaryExpr` e `ifExpr`.
- [x] Existe `ir/program.ir.json` con JSON válido que contiene exclusivamente la función de smoke test `identity` (`SMOKE-001`) y **no** incluye la solución de `calculate_shipping` (**SC-02** / restricción de construcción en vivo).
- [x] Existe `spec/task.yaml` (versión 1) declarando `calculate_shipping(subtotal_cents: i32) -> i32`, los requisitos `SHIP-001`..`SHIP-003` y los 6 casos públicos `SHIP-P01`..`SHIP-P06`.
- [x] Existe `tests/public.yaml` con los 6 vectores públicos de `E1` (`-1 -> -1`, `0 -> 500`, `4999 -> 500`, `5000 -> 0`, `5001 -> 0`, `2147483647 -> 0`).
- [x] **End-to-end:** `ir/ir.schema.json` e `ir/program.ir.json` se parsean sin errores como JSON estricto, y `spec/task.yaml` junto con `tests/public.yaml` son coherentes con las tablas de `E1` en `knowledge/delivery/poc.md`.

## Validation

- Validar que `ir/ir.schema.json` e `ir/program.ir.json` son JSON válidos ejecutando Node.js `JSON.parse` y comprobando que `functions[0].name === 'identity'`.
- Ejecutar `npx @kaddo/cli ready WI-001` y `npx @kaddo/cli explain` para verificar el estado del Work Item sin advertencias de secciones faltantes.

## Definition of Done

- Todos los criterios de aceptación están cumplidos y validados.
- Ninguna regla de negocio de `E1`–`E4` ha sido pre-resuelta en `ir/program.ir.json`.
- El estado del conocimiento en Kaddo (`knowledge/tech/current-state.md`) está sincronizado.

## Open questions

- `[resuelta]` ¿Dónde deben residir los casos públicos para el comando `cargo run -p runner -- test --cases tests/public.yaml`? En `tests/public.yaml`, espejo de la sección `acceptance.public` de `spec/task.yaml`.

## Dependencies

- Ninguna. Este Work Item es la base experimental para `WI-002` (`Verifier + compiler directo a WASM + runner`).
