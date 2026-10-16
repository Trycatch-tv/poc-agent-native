---
type: spike
id: WI-032
alias: KAN-POC4-WI-032
title: Baseline Rust cargo-component y benchmark pareado v0.4
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
  - crates/baseline-rust-004/**
  - runs/rust-baseline-004/**
  - BENCHMARK_REPORT_004.md
---

# WI-032 (KAN-POC4-WI-032) — Baseline Rust cargo-component y benchmark pareado v0.4

## Outcome

- **Actor:** Operador del experimento y agente evaluado en el benchmark pareado de la POC 004.
- **Completed behavior:**
  - Se creó el crate `crates/baseline-rust-004` implementando la lógica comercial de `spec/task-004.yaml` (v3: 12 % de impuestos, auditoría 999/100, `calculate_order` y `quote_shipping`) con la sección personalizada canónica `component-type:order-processor`.
  - Se compiló a WebAssembly (`wasm32-unknown-unknown`) produciendo un binario de 761 bytes.
  - Se ejecutaron las mismas suites de pruebas en Wasmtime mediante `runner`, obteniendo 100 % de aprobación (9/9 públicas, 12/12 reservadas; 21/21 total).
  - Se midieron los tiempos de ciclo deterministas: Agent-Native (`verify + compile` en 54.7 ms) superó a Rust (238.4 ms incremental, 513.3 ms clean) con una ventaja de 4.4x a 9.4x.
  - Se cuantificó la compacidad diagnóstica ante desalineación de interfaz WIT: Agent-Native (366 bytes con JSONPath) superó a Rust (~8.8 KB) por un factor de 24.0x (95.8 % menos ruido sintáctico).
  - Se publicó el reporte comparativo canónico en `BENCHMARK_REPORT_004.md`.
- **Observable verification:**
  - `crates/baseline-rust-004/src/lib.rs`
  - `runs/rust-baseline-004/baseline.wasm` (761 bytes)
  - `runs/rust-baseline-004/public-report.json` (9/9 passed)
  - `runs/rust-baseline-004/hidden-report.json` (12/12 passed)
  - `BENCHMARK_REPORT_004.md`
