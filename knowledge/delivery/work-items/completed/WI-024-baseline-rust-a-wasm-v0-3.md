---
type: spike
id: WI-024
alias: KAN-POC3-WI-024
title: Baseline Rust a WASM v0.3 con Host Imports y benchmark pareado
knowledge_level: K3
status: completed
implementation_status: completed
phase: completed
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
  - crates/baseline-rust-003/**
  - runs/rust-baseline-003/**
  - BENCHMARK_REPORT_003.md
---

# WI-024 (KAN-POC3-WI-024) — Baseline Rust a WASM v0.3 con Host Imports y benchmark pareado

## Outcome

- **Actor:** Operador del experimento y agente evaluado en el benchmark pareado de la POC 003.
- **Completed behavior:**
  - Se implementó el crate `crates/baseline-rust-003` (`cdylib`, `#![no_std]`, `opt-level = "z"`) con imports declarados vía `#[link(wasm_import_module = "host")] extern "C"` (`kv_get`, `check_credit`, `emit_audit`).
  - Se compiló a `wasm32-unknown-unknown` y se ejecutaron las suites de pruebas con el mismo `target/release/runner.exe` sobre Wasmtime 27.0.0.
  - Se obtuvieron 7/7 casos aprobados en `tests/public-003.yaml` y 10/10 casos aprobados en `tests/hidden-003.yaml` (17/17 total, 100 % pass rate y 100 % de aserciones de auditoría superadas).
  - Se midieron los tiempos de ciclo: `verify + compile` en Semantic IR v0.3 (`27 ms`) vs `cargo check + cargo build` en Rust (`445 ms`), confirmando una ventaja de **16.5x**.
  - Se midió el volumen de diagnóstico en fallo: `343 bytes` en Semantic IR v0.3 vs `8,642 bytes` en `cargo check --message-format=json`, demostrando una compacidad diagnóstica de **25.1x** (96 % menos ruido).
  - Se publicó el reporte formal en `BENCHMARK_REPORT_003.md`.
- **Observable verification:**
  - `runs/rust-baseline-003/public-tests.json` (7/7 passed).
  - `runs/rust-baseline-003/hidden-tests.json` (10/10 passed).
  - `runs/rust-baseline-003/baseline_rust_003.wasm` (552 bytes).
  - `runs/rust-baseline-003/defect-diagnostics.json` (8,642 bytes).
  - `BENCHMARK_REPORT_003.md` publicado.

## Acceptance Criteria

- [x] Crate `crates/baseline-rust-003` creado y compilado a `wasm32-unknown-unknown` en release.
- [x] Imports enlazados a módulo `"host"` (`kv_get`, `check_credit`, `emit_audit`).
- [x] 100 % de pruebas públicas y reservadas aprobadas en `runner` (17/17).
- [x] Auditoría de efectos secundarios aprobada al 100 % en runner Wasmtime.
- [x] Métricas cuantitativas pareadas registradas en `BENCHMARK_REPORT_003.md`.
