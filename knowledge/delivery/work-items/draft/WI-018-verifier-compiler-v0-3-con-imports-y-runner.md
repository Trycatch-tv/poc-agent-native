---
type: spike
id: WI-018
alias: KAN-POC3-WI-018
title: Verifier v0.3, Compiler con Imports WASM y Runner con Host Linker
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

# WI-018 (KAN-POC3-WI-018) — Verifier v0.3, Compiler con Imports WASM y Runner con Host Linker

## Outcome

- **Actor:** Operador del micro-toolchain determinista.
- **Current behavior:** `irtool` compila módulos WASM sin sección `import`; `runner` no provee linker de host en Wasmtime.
- **Target behavior:**
  1. Extender `crates/irtool` para verificar autorizaciones de capabilities (`capabilities`), catálogo de `imports` y validar operadores `call_import` (emitiendo diagnósticos `E_UNDECLARED_CAPABILITY`, `E_UNKNOWN_IMPORT`, `E_IMPORT_SIGNATURE_MISMATCH`).
  2. Emitir la sección estándar `import` de WebAssembly (`wasm-encoder::ImportSection`) antes de las secciones de función y memoria.
  3. Extender `crates/runner` para registrar funciones de host en Wasmtime Linker (`kv_get`, `emit_audit`), inyectar estado inicial de host desde los casos de prueba YAML (`host_state.kv`) y registrar llamadas de host para aserciones (`assert_host_calls`).
- **Observable completion:** `cargo test` pasa, el smoke test con import y llamada al host compila y corre satisfactoriamente, y la suite previa de POC 001 y POC 002 mantiene regresión 100 %.
