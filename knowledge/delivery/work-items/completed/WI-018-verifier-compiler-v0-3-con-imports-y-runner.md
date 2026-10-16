---
type: spike
id: WI-018
alias: KAN-POC3-WI-018
title: Verifier v0.3, Compiler con Imports WASM y Runner con Host Linker
knowledge_level: K2
status: completed
implementation_status: completed
phase: completed
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
ready_at: '2026-10-16'
completed_at: '2026-10-16'
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
- **Completed behavior:**
  1. `crates/irtool` verifica autorizaciones de capabilities (`capabilities`), catálogo de `imports` y valida operadores `call_import` (emitiendo diagnósticos deterministas `E_UNDECLARED_CAPABILITY`, `E_UNKNOWN_IMPORT`, `E_CALL_ARITY`, `E_TYPE_MISMATCH`).
  2. `compile_ir_to_wasm` emite la sección estándar `import` de WebAssembly (`wasm-encoder::ImportSection`) antes de `FunctionSection` y `MemorySection`, indexando correctamente funciones importadas (`0 .. num_imports - 1`) y definidas (`num_imports .. num_imports + num_funcs - 1`).
  3. `crates/runner` registra funciones de host en el `Linker` de Wasmtime (`host.kv_get`, `host.emit_audit`, `host.check_credit`), inyecta el estado inicial del host desde los casos de prueba YAML (`host_state.kv`), registra llamadas y ejecuta aserciones deterministas de efectos secundarios auditables (`expected_host_calls` vs `actual_host_calls`).
- **Observable verification:**
  - `cargo test --workspace`: 8/8 tests unitarios pasando (5 en `irtool`, 3 en `runner`).
  - Verificación y compilación de `ir/smoke-v3.ir.json` con 0 diagnósticos y validación con `wasmparser` exitosa.
  - Ejecución end-to-end con `runner` en `tests/smoke-v3.yaml` pasando 1/1 con aserción de auditoría.
  - 100 % de regresión confirmada: POC 001 (12/12) y POC 002 (public 12/12, hidden 18/18, smoke 2/2).

## Scope

- Extensión completa de `irtool` (verificador y compilador) para soportar IR v0.3.
- Extensión de `runner` (Wasmtime Linker) para soportar mocks deterministas de host imports y auditoría.
- Validación de `ir/smoke-v3.ir.json` y `tests/smoke-v3.yaml`.
- Validación de no-regresión de suites previas de POC 001 y POC 002.
