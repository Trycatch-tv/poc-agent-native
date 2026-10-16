---
type: spike
id: WI-008
alias: KAN-POC-WI-008
title: Baseline Rust a WASM y benchmark comparativo pareado
knowledge_level: K2
status: completed
implementation_status: completed
phase: next
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
  - BENCHMARK_REPORT.md
  - Cargo.toml
  - crates/baseline-rust/**
  - runs/rust-baseline/**
  - knowledge/delivery/poc.md
  - knowledge/delivery/poc-report.md
  - knowledge/delivery/poc-report-v002.md
  - knowledge/tech/current-state.md
---

# WI-008 (KAN-POC-WI-008) — Baseline Rust a WASM y benchmark comparativo pareado

## Outcome

- **Actor:** Investigador principal y operador de `KADDO-AGENT-NATIVE-POC-001`.
- **Current behavior:** La ruta principal `Agent-Native` (`Semantic IR v0.1 → irtool → program.wasm`) completó `E1`–`E4` con `16/16` criterios aprobados (`v1.0.0-poc-validated`), pero el spike opcional `KAN-POC-WI-008` propuesto en `knowledge/delivery/poc.md` (*Baseline Rust→WASM*: implementar la misma secuencia `E1`–`E4` en Rust compilado a `wasm32-unknown-unknown` y evaluado con el mismo `runner` y las mismas suites públicas/reservadas) aún no se había ejecutado.
- **Target behavior:**
  1. Crear un crate aislado de baseline tradicional (`crates/baseline-rust`, `crate-type = ["cdylib"]`, `#![no_std]`) compilado con `cargo build --target wasm32-unknown-unknown --release` **fuera** del pipeline `irtool` (manteniendo intactos `ir/program.ir.json`, `crates/irtool` y `crates/runner`).
  2. Ejecutar de forma pareada los 4 ejercicios sobre el baseline Rust→WASM usando las mismas especificaciones (`v1`, `v2`, `v3`) y las mismas suites de pruebas públicas y reservadas (`B1`, `B2`, `B3`, `B4`).
  3. Registrar todas las mediciones pareadas en `runs/rust-baseline/` (`b1-build/`, `b2-repair/`, `b3-extend/`, `b4-maintain/` y `comparison-summary.json` / `comparison-summary.md`), generar el reporte standalone `BENCHMARK_REPORT.md`, actualizar `knowledge/delivery/poc.md` y emitir `knowledge/delivery/poc-report-v002.md`.
- **Observable completion:**
  1. Existen las 4 corridas del baseline Rust en `runs/rust-baseline/{b1-build,b2-repair,b3-extend,b4-maintain}/attempt-01/` con sus `.wasm`, `.wat`, `public-tests.json`, `hidden-tests.json` y `run.json`.
  2. Existen `BENCHMARK_REPORT.md` y `runs/rust-baseline/comparison-summary.md` con la comparación pareada (`Agent-Native IR → WASM` vs `Rust → wasm32-unknown-unknown`).
  3. `runs/toolchain-freeze.sha256` continúa cumpliéndose al `100 %` para `irtool.exe` y `runner.exe`.

## Journey reconstruction

```text
Misma spec (v1..v3) + mismos tests públicos y reservados + mismo runner (Wasmtime)
   ↓
Ruta A (ya medida en E1–E4): Agente → ir/program.ir.json → irtool verify/compile → program.wasm
   ↓
Ruta B (WI-008): Agente → crates/baseline-rust/src/lib.rs → cargo/rustc (wasm32-unknown-unknown) → baseline_rust.wasm
   ↓
Mismo validador (irtool validate) + mismo runner (runner test) sobre B1, B2, B3 y B4
   ↓
Comparación pareada en BENCHMARK_REPORT.md + runs/rust-baseline/comparison-summary.md + poc-report-v002.md
```

## Problem

Aunque `POC 001` validó que la ruta `Agent-Native` (`Semantic IR → WASM`) cumple los 16 criterios de éxito, tomar decisiones arquitectónicas sobre dónde aporta ventaja real frente a pedirle al agente escribir Rust directamente requiere un **benchmark pareado** bajo la misma especificación, las mismas suites públicas/reservadas y el mismo runtime (`Wasmtime`).

_Expected value:_ Cuantificar con datos medidos las diferencias entre `Semantic IR → WASM` y `Rust → wasm32-unknown-unknown` en tiempo de verificación/compilación, tamaño de binario `.wasm`, estructura y ruido de diagnósticos, localidad de diffs y fricción de ABI.

## Scope

- Asegurar que el target `wasm32-unknown-unknown` esté disponible en la toolchain Rust local.
- Crear `crates/baseline-rust` (`#![no_std]`, `cdylib`) aislado del pipeline Agent-Native.
- Ejecutar la serie pareada `B1` (`E1`), `B2` (`E2`), `B3` (`E3`) y `B4` (`E4`) capturando tiempos de `cargo check`/`cargo build --target wasm32-unknown-unknown --release`, tamaño de `.wasm`, validación con `irtool validate`, desensamblado `.wat` y resultados con `runner test`.
- Consolidar la tabla comparativa pareada en `BENCHMARK_REPORT.md` y `runs/rust-baseline/comparison-summary.md` (y `.json`), integrarla en `knowledge/delivery/poc.md` y persistir la versión `v2` del reporte final (`knowledge/delivery/poc-report-v002.md` y `knowledge/delivery/poc-report.md`).

## Out of scope

- Modificar `ir/program.ir.json`, `crates/irtool` o `crates/runner` (el pipeline Agent-Native y su freeze permanecen intactos).
- Cambiar la conclusión canónica (`validated` / `GO`) de la POC primaria.

## Surface review

- **Product/UI / Frontend / Database / Operations:** not-applicable.
- **Backend:** affected únicamente por la adición del crate aislado `crates/baseline-rust` para el benchmark comparativo; `crates/irtool` y `crates/runner` permanecen intactos (`reviewed-not-affected`).
- **Documentation:** affected (`BENCHMARK_REPORT.md`, `runs/rust-baseline/comparison-summary.md`, `knowledge/delivery/poc.md`, `knowledge/delivery/poc-report-v002.md`, `knowledge/delivery/poc-report.md`, `knowledge/tech/current-state.md`).

## Acceptance Criteria

- [x] Existe `crates/baseline-rust` compilable a `wasm32-unknown-unknown` sin alterar los hashes congelados de `target/debug/irtool.exe` ni `target/debug/runner.exe`.
- [x] Se ejecutan y registran los 4 ejercicios pareados (`B1`, `B2`, `B3`, `B4`) en `runs/rust-baseline/{b1-build,b2-repair,b3-extend,b4-maintain}/attempt-01/` pasando `irtool validate` y el `100 %` de las pruebas públicas y reservadas en el mismo `runner`.
- [x] Existen `BENCHMARK_REPORT.md` y `runs/rust-baseline/comparison-summary.md` (y `comparison-summary.json`) con la comparación cuantitativa pareada `Agent-Native IR → WASM` (`E1`–`E4`) vs `Rust → WASM` (`B1`–`B4`): corrección, intentos, latencia de verificación/compilación (`ms`), tiempo de test (`ms`), tamaño de `.wasm` (`bytes`), tamaño de fuente/diff y accionabilidad de diagnósticos.
- [x] **End-to-end:** `knowledge/delivery/poc.md` y `knowledge/delivery/poc-report-v002.md` (`Final report: current`) incluyen los resultados del benchmark comparativo pareado.

## Validation

1. Ejecutar `.\target\debug\irtool.exe validate` y `.\target\debug\runner.exe test` sobre los módulos `.wasm` generados en `B1`–`B4` (`100 %` aprobados).
2. Comprobar que los hashes de `target/debug/irtool.exe` y `target/debug/runner.exe` continúan coincidiendo con `runs/toolchain-freeze.sha256`.
3. Ejecutar `npx -y @kaddo/cli@latest poc report` y verificar `Final report: current` en `v2` (`knowledge/delivery/poc-report-v002.md`).

## Definition of Done

- Todos los criterios de aceptación de `WI-008` están verificados `[x]` con evidencia en `runs/rust-baseline/` y `BENCHMARK_REPORT.md`.
- `knowledge/delivery/poc.md`, `knowledge/delivery/poc-report-v002.md`, `knowledge/delivery/poc-report.md` y `knowledge/tech/current-state.md` están actualizados.

## Open questions

- Ninguna pregunta abierta bloqueante (`resolution_status = open`).

## Dependencies

- Depende de `WI-007` (`Análisis y conclusión`, completado).
