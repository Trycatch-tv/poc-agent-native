---
type: spike
id: WI-010
alias: KAN-POC2-WI-010
title: Verifier v0.2 + compiler IR v0.2 directo a WASM + runner
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
  - crates/irtool/**
  - crates/runner/**
  - tests/**
---

# WI-010 (KAN-POC2-WI-010) — Verifier v0.2 + compiler IR v0.2 directo a WASM + runner

## Outcome

- **Actor:** Operador del experimento y agente evaluado en `KADDO-AGENT-NATIVE-POC-002`.
- **Current behavior:** La toolchain previa (`irtool` y `runner`) solo soportaba Semantic IR v0.1 (escalares `i32`/`bool`).
- **Target behavior:** `crates/irtool` soporta verificación semántica estricta de IR v0.2 (validación de campos de record, firmas de `call`, type checking de llamadas y layouts de memoria) y lowering determinista directo a WebAssembly Core con sección `memory` y offsets precalculados; `crates/runner` soporta pasar y leer estructuras compuestas en memoria lineal de Wasmtime.
- **Observable completion:** `cargo run -p irtool -- verify --schema ir/ir.schema.v2.json --ir ir/smoke-v2.ir.json` verifica el smoke test con `0` diagnósticos; `cargo run -p irtool -- compile` emite un binario `.wasm` válido con layouts de record correctos; y `cargo run -p runner` ejecuta y aprueba los casos de smoke test compuestos.

## Scope

- Ampliar `crates/irtool`:
  - Verificador semántico para IR v0.2: validación de firmas de llamadas (`call`), verificación de acceso a campos (`record.get`), validación de constructores (`record.construct`), y emisión de diagnósticos JSON estructurados con códigos `E_UNKNOWN_FIELD`, `E_SIGNATURE_MISMATCH`, etc.
  - Compilador directo a WASM: emisión de WebAssembly Core con funciones privadas internas, sección `call`, emisión de `memory` estática para registros y lectura/escritura determinista de offsets.
- Ampliar `crates/runner`:
  - Soporte en el runner Wasmtime para mapear structs de entrada/salida desde/hacia memoria lineal de WASM según el esquema.
- Pruebas unitarias de verificación determinista en `irtool`.

## Out of scope

- Implementación de la tarea de negocio comercial de `E1-002` (pertenece a `WI-011`).

## Acceptance Criteria

- [x] `cargo run -p irtool -- verify --schema ir/ir.schema.v2.json --ir ir/smoke-v2.ir.json --diagnostics build/smoke-diag.json` valida exitosamente con `exit 0` y `diagnostics = []`.
- [x] `cargo run -p irtool -- compile --ir ir/smoke-v2.ir.json --output build/smoke-v2.wasm` compila el smoke test emitiendo WebAssembly Core válido con memoria lineal y llamadas internas.
- [x] `cargo run -p irtool -- validate build/smoke-v2.wasm` valida limpiamente contra `wasmparser`.
- [x] `cargo run -p runner -- test --wasm build/smoke-v2.wasm --cases tests/smoke-v2.yaml` ejecuta en Wasmtime y valida el retorno compuesto del smoke test (`PointSum { sum: 30 }`).
- [x] Mantiene compatibilidad regresiva completa con la IR v0.1 (`cargo test` pasa al 100 %).

## Validation

- Ejecutado `cargo test --workspace` con 5 pruebas unitarias e integración en verde.
- Ejecutado el pipeline completo sobre `ir/smoke-v2.ir.json`: `verify` (0 diagnósticos), `compile` (113 bytes), `validate` (válido con wasmparser) y `test` en runner (2/2 casos pasados).
- Ejecutado test de regresión sobre `build/program.wasm` de POC 001 con 12/12 casos aprobados.

## Learning

- **Qué se entregó:**
  1. `crates/irtool/src/lib.rs`: Soporte integral para Semantic IR v0.2 con registro de tipos de records (`RecordDef`), firmas de funciones internas/exportadas, type checking estricto de operadores `call`, `record.construct`, `record.get`, y lowering a WebAssembly Core con sección `memory` y puntero de heap global determinista.
  2. `crates/runner/src/main.rs`: Soporte para lectura de estructuras retornadas en memoria lineal WASM mapeadas automáticamente contra las expectativas de mapping en YAML/JSON.
  3. `tests/smoke-v2.yaml`: Suite de smoke test v0.2 ejecutada en Wasmtime aprobando 2/2 casos en 23 ms.
- **Decisiones técnicas:**
  1. Se implementó un bump allocator determinista sobre un global mutable `i32` (`$heap_ptr = 1024`) en WASM, asegurando que cada llamada y test case sea aislado y reproducible sin un runtime GC externo.
  2. El runner lee los campos ordenados de memoria lineal preservando compatibilidad 100 % con retornos escalares `i32`/`bool`.
