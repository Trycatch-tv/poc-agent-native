---
type: spike
id: WI-024
alias: KAN-POC3-WI-024
title: Baseline Rust a WASM v0.3 con Host Imports y benchmark pareado
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
  - crates/baseline-rust-003/**
  - runs/rust-baseline-003/**
  - BENCHMARK_REPORT_003.md
---

# WI-024 (KAN-POC3-WI-024) — Baseline Rust a WASM v0.3 con Host Imports y benchmark pareado

## Outcome

- **Actor:** Operador del experimento y agente evaluado en el benchmark pareado de la POC 003.
- **Current behavior:** No existe implementación equivalente en Rust con `extern "C"` imports de host para comparar la ruta Agent-Native v0.3.
- **Target behavior:**
  - Crear crate `crates/baseline-rust-003` (`cdylib`, `#![no_std]`) con imports `extern "C"` (`kv_get`, `emit_audit`).
  - Implementar y medir de forma pareada `B1-003` a `B4-003` contra el mismo `runner` Wasmtime con host linker.
  - Medir latencia de feedback, tamaño binario `.wasm`, volumen de diagnósticos en falla de link y trade-offs de seguridad.
  - Generar `BENCHMARK_REPORT_003.md`.
- **Observable completion:** Evidencia completa en `runs/rust-baseline-003/` y reporte comparativo pareado publicado.
