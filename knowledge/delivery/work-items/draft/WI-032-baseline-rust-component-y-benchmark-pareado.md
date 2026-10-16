---
type: spike
id: WI-032
alias: KAN-POC4-WI-032
title: Baseline Rust cargo-component y benchmark pareado v0.4
knowledge_level: K3
status: draft
implementation_status: not-started
phase: now
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
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
- **Current behavior:** No existe implementación de componente equivalente en Rust con bindings WIT automáticos para comparar la ruta Agent-Native v0.4.
- **Target behavior:**
  - Crear crate `crates/baseline-rust-004` con `cargo-component` o bindings `wit-bindgen`.
  - Implementar y medir de forma pareada contra el mismo runner Wasmtime Component Model.
  - Medir latencia de feedback (`verify + package` vs `cargo component build`), tamaño binario del componente `.wasm` y compacidad diagnóstica en desalineación de interfaz.
  - Publicar `BENCHMARK_REPORT_004.md`.
- **Observable completion:** Evidencia completa en `runs/rust-baseline-004/` y reporte comparativo pareado publicado.
