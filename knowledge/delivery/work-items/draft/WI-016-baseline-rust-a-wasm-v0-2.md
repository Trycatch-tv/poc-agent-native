---
type: spike
id: WI-016
alias: KAN-POC2-WI-016
title: Baseline Rust a WASM v0.2 y benchmark pareado
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
  - crates/baseline-rust-002/**
  - runs/rust-baseline-002/**
  - BENCHMARK_REPORT_002.md
---

# WI-016 (KAN-POC2-WI-016) — Baseline Rust a WASM v0.2 y benchmark pareado

## Outcome

- **Actor:** Operador del experimento y agente evaluado en el benchmark pareado de la POC 002.
- **Current behavior:** No existe implementación equivalente en Rust con structs y manejo de memoria en WASM para comparar la ruta Agent-Native v0.2.
- **Target behavior:** Se implementa en un crate Rust aislado (`crates/baseline-rust-002`, con `#![no_std]` y structs) el cálculo de la orden comercial con los mismos casos y especificación de `spec/task-002.yaml`. Se mide de forma pareada la latencia de feedback (`irtool verify + compile` vs `cargo check + build`), tamaño del binario `.wasm`, volumen de diagnósticos en bytes y complejidad de memoria. Se genera `BENCHMARK_REPORT_002.md`.
- **Observable completion:** Reporte comparativo pareado con mediciones reproducibles y análisis de trade-offs.
