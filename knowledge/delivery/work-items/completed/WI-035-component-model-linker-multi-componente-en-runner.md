---
type: spike
id: WI-035
alias: KAN-POC5-WI-035
title: Component Model Linker multi-componente en Runner
knowledge_level: K3
status: completed
implementation_status: completed
phase: completed
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
closed_at: '2026-10-16'
domains:
  - deterministic-toolchain
  - component-model
code:
  - crates/runner/**
---

# WI-035 (KAN-POC5-WI-035) — Component Model Linker multi-componente en Runner

## Outcome

- **Actor:** Runtime Wasmtime y suites de integración de `poc-agent-native`.
- **Target behavior:**
  - Extender `crates/runner` para soportar la composición y enlace estático de múltiples componentes WebAssembly (`wasmtime::component::Linker`).
  - Permitir que un componente principal (`order-processor.wasm`) importe interfaces provistas por un segundo componente (`shipping-calculator.wasm`), además de las capacidades de host (`host-services`).
  - Soporte para ejecutar suites YAML multi-componente pasando fixtures y validando invocaciones cruzadas entre componentes sin violar el encapsulamiento de memoria.
- **Observable completion:** `runner test --wasm order-processor.wasm --link shipping-calculator.wasm --cases tests/...` ejecuta e integra ambos componentes en Wasmtime con 0 errores (9/9 casos passing en `tests/narrative-005.yaml`).

## Acceptance Criteria

- [x] Soporte para instanciación y enlace cruzado de múltiples componentes en `runner`.
- [x] Validación de interfaces WIT cruzadas entre componentes sin mocks de negocio en el host.
- [x] Preservación de retrocompatibilidad con suites de POC 001, 002, 003 y 004.
