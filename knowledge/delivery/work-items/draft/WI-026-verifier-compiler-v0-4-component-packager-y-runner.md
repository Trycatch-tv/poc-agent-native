---
type: spike
id: WI-026
alias: KAN-POC4-WI-026
title: Verifier v0.4, Component Packaging / Canonical ABI y Runner Component Linker
knowledge_level: K2
status: draft
implementation_status: not-started
phase: now
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
domains:
  - semantic-ir
  - deterministic-toolchain
code:
  - crates/irtool/**
  - crates/runner/**
---

# WI-026 (KAN-POC4-WI-026) — Verifier v0.4, Component Packaging / Canonical ABI y Runner Component Linker

## Outcome

- **Actor:** Operador del micro-toolchain determinista.
- **Current behavior:** `irtool` emite módulos WebAssembly Core con `ImportSection` y `runner` sólo enlaza funciones escalares en `wasmtime::Linker`.
- **Target behavior:**
  1. Extender `crates/irtool` para verificar la coherencia entre `ir.json` y el contrato `wit` (emitiendo `E_WIT_TYPE_MISMATCH`, `E_UNKNOWN_EXPORT_INTERFACE`).
  2. Implementar en `irtool` la capacidad de compilar y empaquetar WebAssembly Components válidos para Wasmtime.
  3. Extender `crates/runner` para soportar instanciación y ejecución en `wasmtime::component::Component` y `wasmtime::component::Linker` con host mocks que implementan interfaces WIT.
- **Observable completion:** `cargo test --workspace` pasando 100 %, verificación y empaquetado de `ir/smoke-v4.ir.json` exitoso, y ejecución en `runner` de `tests/smoke-v4.yaml` pasando 100 %.
