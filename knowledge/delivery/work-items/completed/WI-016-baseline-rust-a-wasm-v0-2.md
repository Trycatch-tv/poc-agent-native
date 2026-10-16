---
type: spike
id: WI-016
alias: KAN-POC2-WI-016
title: Baseline Rust a WASM v0.2 y benchmark comparativo pareado
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
  - commercial-rules-subject
  - experiment-evidence
code:
  - crates/baseline-rust-002/**
  - runs/rust-baseline-002/**
  - BENCHMARK_REPORT_002.md
---

# WI-016 (KAN-POC2-WI-016) — Baseline Rust a WASM v0.2 y benchmark comparativo pareado

## Outcome

- **Actor:** Operador del experimento y agente evaluado en el benchmark pareado de la POC 002.
- **Current behavior:** La ruta principal `Agent-Native` v0.2 (`Semantic IR v0.2 → irtool → program.wasm`) completó `E1-002`–`E4-002` con `16/16` criterios aprobados (`v1.0.0-poc002-validated`), pero no existía la comparación cuantitativa pareada frente a `Rust → wasm32-unknown-unknown` con estructuras de datos (`structs`), llamadas internas y memoria lineal.
- **Target behavior:**
  1. Crear un crate aislado (`crates/baseline-rust-002`, `cdylib`, `#![no_std]`, `opt-level = "z"`) que compile a `wasm32-unknown-unknown`.
  2. Implementar y medir de forma pareada los 4 ejercicios sobre el baseline (`B1-002`, `B2-002`, `B3-002`, `B4-002`) con las especificaciones (`v1`, `v2`, `v3`) y las suites públicas y reservadas, ejecutado contra el mismo `runner` de Wasmtime.
  3. Medir latencia de verificación (`irtool verify` vs `cargo check`), latencia de compilación (`irtool compile` vs `cargo build --release`), tamaño del `.wasm`, volumen de diagnósticos en bytes (`JSONPath` vs `rustc JSON`) y diffs de mantenimiento.
  4. Generar `runs/rust-baseline-002/comparison-summary.json`, `runs/rust-baseline-002/comparison-summary.md` y `BENCHMARK_REPORT_002.md`.
- **Observable completion:** Evidencia completa en `runs/rust-baseline-002/`; 100 % pass rate en runner para B1..B4; tabla comparativa cuantitativa demostrando ventajas de 10x–19x en velocidad de toolchain y 9.1x en compacidad diagnóstica; y `BENCHMARK_REPORT_002.md` publicado.

## Scope

- Configurar `crates/baseline-rust-002/Cargo.toml` y `src/lib.rs`.
- Ejecutar B1-002 (Construcción con `OrderCalculation` v1, 4 campos `i32`).
- Ejecutar B2-002 (Inyección de defecto en campo de struct, medición de bytes de diagnóstico de `cargo check`, y reparación en 1 intento).
- Ejecutar B3-002 (Extensión con `status_code` 0..3 y validaciones de dominio v2).
- Ejecutar B4-002 (Mantenimiento con alícuota 15 % en subfunción v3, 3 builds deterministas).
- Comparar métricas contra E1-002, E2-002, E3-002, E4-002.

## Acceptance Criteria

- [x] Crate aislado `crates/baseline-rust-002` configurado y funcional.
- [x] Ejercicios `B1-002` a `B4-002` ejecutados pareadamente y aprobados al 100 % con el mismo runner Wasmtime.
- [x] Mediciones registradas en `runs/rust-baseline-002/`:
  - `B1-002`: `232 ms` total toolchain vs `23 ms` en `E1-002` (`10.1x`).
  - `B2-002`: `5,418 bytes` de diagnóstico vs `593 bytes` en `E2-002` (`9.14x más compacto`).
  - `B3-002`: `424 ms` total toolchain vs `24 ms` en `E3-002` (`17.7x`).
  - `B4-002`: `432 ms` total toolchain vs `23 ms` en `E4-002` (`18.8x`). 3/3 builds deterministas idénticos.
- [x] Generados `runs/rust-baseline-002/comparison-summary.json` y `comparison-summary.md`.
- [x] Generado reporte canónico `BENCHMARK_REPORT_002.md`.
