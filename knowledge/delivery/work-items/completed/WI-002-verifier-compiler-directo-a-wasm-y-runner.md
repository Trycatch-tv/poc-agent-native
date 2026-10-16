---
type: spike
id: WI-002
alias: KAN-POC-WI-002
title: Verifier + compiler directo a WASM + runner
knowledge_level: K3
status: completed
implementation_status: completed
phase: now
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
ready_at: '2026-10-16'
completed_at: '2026-10-16'
domains:
  - deterministic-toolchain
  - semantic-ir
  - experiment-evidence
code:
  - Cargo.toml
  - Cargo.lock
  - crates/**
  - compiler/**
  - runner/**
  - tests/**
---

# WI-002 (KAN-POC-WI-002) — Verifier + compiler directo a WASM + runner

## Outcome

- **Actor:** Operador del experimento y agente evaluado durante los ejercicios `E1`–`E4` de `KADDO-AGENT-NATIVE-POC-001`.
- **Current behavior:** El repositorio cuenta con los contratos declarativos de `WI-001` (`ir/ir.schema.json`, `ir/program.ir.json` con `identity`, `spec/task.yaml` v1 y `tests/public.yaml`), pero aún no existe el workspace de Rust ni los binarios deterministas (`irtool` y `runner`) capaces de verificar la IR, compilarla directamente a `.wasm` y ejecutar suites de pruebas en Wasmtime.
- **Target behavior:** Existe un workspace Cargo en Rust con los paquetes `irtool` (`verify` y `compile`) y `runner` (`test`) que validan estructural y semánticamente cualquier programa en `Semantic IR v0.1`, emiten diagnósticos JSON estructurados (`diagnostics.json`), generan directamente binarios WebAssembly Core (`.wasm`) deterministas y reproducibles sin pasar por código fuente tradicional ni contener reglas de negocio cableadas, y ejecutan suites YAML de casos de prueba usando Wasmtime.
- **Observable completion:**
  1. `cargo run -p irtool -- verify --schema ir/ir.schema.json --ir ir/program.ir.json --diagnostics build/diagnostics.json` valida con éxito (`exit 0`, `[]`) el smoke test `identity` y rechaza con `exit != 0` y códigos `E_*` cualquier IR inválida en las pruebas unitarias del verificador.
  2. `cargo run -p irtool -- compile --ir ir/program.ir.json --output build/program.wasm` emite un binario `.wasm` válido que pasa `wasm-tools validate` (`wasmparser`) y es ejecutable en Wasmtime.
  3. `cargo run -p runner -- test --wasm build/program.wasm --cases tests/smoke.yaml --output build/smoke-tests.json` ejecuta el módulo `identity` en Wasmtime y aprueba el 100 % de los casos de smoke test.

## Journey reconstruction

```text
ir/program.ir.json (smoke test: identity)
   ↓
cargo run -p irtool -- verify
   ├── Válida   → build/diagnostics.json = [] (exit 0)
   └── Inválida → build/diagnostics.json = [{ code, function, path, message, ... }] (exit != 0)
   ↓
cargo run -p irtool -- compile
   └── Lowering directo IR v0.1 → build/program.wasm (reproducible byte-a-byte)
   ↓
cargo run -p runner -- test (Wasmtime)
   └── Invoca exports WASM con casos YAML → build/smoke-tests.json (passed/failed/total)
```

## Problem

Para evaluar la hipótesis `H1` en `E1`–`E4`, se requiere una **Zona Determinista** completamente funcional antes de congelar el backend tras `E1`: un verificador semántico que produzca diagnósticos accionables (`H1-B`, `SC-04`), un compilador directo `IR v0.1 → WASM` reproducible (`H1-E`, `SC-10`, `SC-11`) sin lógica de negocio cableada (`SC-13`), y un runner sobre Wasmtime que evalúe casos públicos y reservados (`SC-01`, `SC-15`).

_Expected value:_ Habilitar el ciclo completo `verify → compile → validate → test` de la POC de manera determinista, auditable y libre de lógica de negocio específica de shipping/discount.

## Scope

- Crear el workspace Cargo en la raíz del repositorio (`Cargo.toml`) con dos binarios/paquetes:
  1. **`irtool` (`crates/irtool`):**
     - `verify --schema <path> --ir <path> --diagnostics <path>`
     - `compile --ir <path> --output <path> [--wat <path>]`
     - `validate <wasm-path>` (usando `wasmparser`, el validador de `wasm-tools validate`)
     - `print <wasm-path> [-o <wat-path>]` (usando `wasmprinter`, el generador WAT de `wasm-tools print`)
     - `sha256 <file>`
  2. **`runner` (`crates/runner`):**
     - `test --wasm <path> --cases <path> --output <path>` sobre **Wasmtime v27**.
- Crear `tests/smoke.yaml` para probar el módulo neutral `identity` de `ir/program.ir.json` sin tocar `calculate_shipping`.
- Incluir pruebas automatizadas (`cargo test`) que verifiquen cada uno de los 11 códigos `E_*` del verificador, la compilación + ejecución en Wasmtime de todos los operadores de `IR v0.1`, y la reproducibilidad SHA-256 en 3 compilaciones consecutivas.

## Out of scope

- Modificar `ir/program.ir.json` para implementar `calculate_shipping` (eso pertenece exclusivamente a **`WI-003` — Ejercicio `E1`**).
- Introducir cualquier constante, umbral (`5000`, `4000`, `500`, `600`) o regla de negocio de shipping/discount dentro del código Rust de `irtool` o `runner` (**SC-13**).
- Ejecutar los ejercicios `E1`, `E2`, `E3` o `E4`.

## Surface review

- **Product/UI:** not-applicable.
- **Frontend:** not-applicable.
- **Backend:** affected — implementación en Rust de los binarios CLI `irtool` y `runner`.
- **Database:** not-applicable.
- **Configuration:** affected — `Cargo.toml`, `Cargo.lock` y `tests/smoke.yaml`.
- **Feature flags / Auth / Notifications / Analytics:** not-applicable.
- **Documentation:** reviewed — actualizar `knowledge/tech/current-state.md` tras implementar el workspace Rust.
- **Operations/release:** reviewed-not-affected.

## Acceptance Criteria

- [x] Existe el workspace Cargo (`Cargo.toml`) con los paquetes `irtool` y `runner` que compilan sin errores (`cargo build --workspace`).
- [x] `cargo run -p irtool -- verify --schema ir/ir.schema.json --ir ir/program.ir.json --diagnostics build/diagnostics.json` finaliza con `exit 0` y escribe `[]` en `build/diagnostics.json`.
- [x] El verificador detecta y reporta en JSON estructurado (`code`, `function`, `path`, `message`) las 11 reglas semánticas (`E_DUPLICATE_FUNCTION`, `E_DUPLICATE_PARAM`, `E_UNKNOWN_PARAM`, `E_TYPE_MISMATCH`, `E_IF_CONDITION`, `E_IF_BRANCH_TYPE`, `E_RETURN_TYPE`, `E_UNSUPPORTED_OP`, `E_CONST_TYPE`, `E_CONST_DIV_ZERO`, `E_MISSING_REQUIREMENT`), verificado mediante `cargo test`.
- [x] `cargo run -p irtool -- compile --ir ir/program.ir.json --output build/program.wasm` genera directamente el binario WebAssembly válido sin usar reglas de negocio cableadas, y 3 compilaciones consecutivas de la misma IR producen exactamente el mismo hash SHA-256 (**SC-11**).
- [x] `cargo run -p runner -- test --wasm build/program.wasm --cases tests/smoke.yaml --output build/smoke-tests.json` ejecuta el `.wasm` en Wasmtime y pasa el 100 % de los casos del smoke test `identity`.
- [x] **End-to-end:** `cargo test --workspace` pasa al 100 %, el pipeline `verify → compile → test` funciona de punta a punta sobre `ir/program.ir.json` (`identity`), y `ir/program.ir.json` permanece intacto con únicamente la función `identity` (`SMOKE-001`).

## Validation

1. Ejecutar la suite de pruebas unitarias e integración del toolchain:
   `cargo test --workspace`
2. Ejecutar el flujo end-to-end sobre el smoke test neutral (`identity`):
   - `cargo run -p irtool -- verify --schema ir/ir.schema.json --ir ir/program.ir.json --diagnostics build/diagnostics.json`
   - `cargo run -p irtool -- compile --ir ir/program.ir.json --output build/program.wasm`
   - `cargo run -p runner -- test --wasm build/program.wasm --cases tests/smoke.yaml --output build/smoke-tests.json`
3. Comprobar reproducibilidad de 3 compilaciones seguidas comparando el SHA-256 de `build/program.wasm`.

## Definition of Done

- Todos los criterios de aceptación se cumplen y las validaciones pasan sin errores.
- Cero lógica de negocio de `E1`–`E4` en el código fuente de `irtool` o `runner` (**SC-13**).
- `ir/program.ir.json` sigue conteniendo únicamente el smoke test `identity`.
- `knowledge/tech/current-state.md` e inventario técnico de Kaddo actualizados (`kaddo context`, `kaddo explain`).

## Open questions

- Ninguna pregunta abierta bloqueante (`resolution_status = open`). Todas las reglas de tipado y el esquema de `diagnostics.json` quedaron formalizados en `knowledge/delivery/poc.md`.

## Dependencies

- Depende de `WI-001` (`KAN-POC-WI-001`, completado).
- Bloquea `WI-003` (`KAN-POC-WI-003` — Ejercicio `E1` construcción y freeze).

## Learning

_Capturado al completar `WI-002` (2026-10-16):_

- **Qué se entregó:** Workspace Cargo con `crates/irtool` (`verify`, `compile`, `validate`, `print`, `sha256`) y `crates/runner` (`test` sobre `wasmtime` v27.0.0), además de `tests/smoke.yaml` y la suite de pruebas `cargo test --workspace` que verifica los 11 códigos `E_*`, todos los operadores de `IR v0.1` en Wasmtime y la reproducibilidad byte-a-byte (`sha256=b1a8d1c8fb87aca50030479339fbbbcebe9031feb76607dad64966f9c2d80b04` para el smoke test `identity` de 42 bytes).
- **Qué se aprendió / decisiones emergentes:**
  1. Como los binarios globales `wasm-tools` y `wasmtime` no estaban instalados en el `PATH` de Windows, se integraron directamente las librerías oficiales de Bytecode Alliance (`wasm-encoder`, `wasmparser`, `wasmprinter` v0.221 y `wasmtime` v27.0.0 compatible con `rustc 1.85.0`) dentro de `irtool` y `runner`, exponiendo también `irtool validate`, `irtool print` e `irtool sha256`. Así el repositorio es 100 % autónomo y reproducible en cualquier máquina con Rust.
- **Conocimiento actualizado:** `knowledge/tech/current-state.md` y artefactos derivados en `.kaddo/`.
- **Pendientes:** Proceder con `WI-003` (`E1 — Construcción y freeze`).
