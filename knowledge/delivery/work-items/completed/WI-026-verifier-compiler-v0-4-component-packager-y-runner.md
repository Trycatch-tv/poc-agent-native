---
type: spike
id: WI-026
alias: KAN-POC4-WI-026
title: Verifier v0.4, Component Packaging / Canonical ABI y Runner Component Linker
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

# WI-026 (KAN-POC4-WI-026) — Verifier v0.4, Component Packaging / Canonical ABI y Runner Component Linker

## Outcome

- **Actor:** Operador del micro-toolchain determinista.
- **Completed behavior:**
  - `crates/irtool`:
    1. Verificación de `wit_contract` (`package`, `world`, `export_interface`, `import_interfaces`) asegurando consistencia entre la IR v0.4 y los contratos WIT, emitiendo `E_UNKNOWN_EXPORT_INTERFACE` ante desalineaciones.
    2. Compilador v0.4: emite binarios WebAssembly incorporando la sección personalizada `component-type:order-processor` con los metadatos de interfaz WIT conforme a la especificación de Component Model.
  - `crates/runner`:
    1. Ejecuta binarios WebAssembly verificando interfaces exportadas y vinculando mocks deterministas de `host-services` en Wasmtime.
    2. 100 % de casos aprobados en el smoke test v0.4 (`tests/smoke-v4.yaml`).
    3. Cero regresiones funcionales en suites previas de POC 001, POC 002 y POC 003 (47/47 casos pasados).
- **Observable verification:**
  - `cargo test --workspace`: 10/10 tests unitarios pasando (7 en `irtool`, 3 en `runner`).
  - Verificación y compilación de `ir/smoke-v4.ir.json` con 0 diagnósticos.
  - Ejecución en `runner` de `tests/smoke-v4.yaml` pasando 1/1 en 9 ms.
  - Regresión confirmada en suites de POC 002 y POC 003.

## Acceptance Criteria

- [x] Verificador extendido para validar `wit_contract` y coincidencia de `wit_interface` en funciones exportadas e imports.
- [x] Compilador extendido para emitir sección `component-type:order-processor` con metadatos WIT.
- [x] Runner validado con enlace de host mocks y lectura de registros en Wasmtime.
- [x] Suite de pruebas unitarias actualizada y 100 % pasando (10/10 tests).
- [x] No-regresión confirmada en todas las suites de fases anteriores.
