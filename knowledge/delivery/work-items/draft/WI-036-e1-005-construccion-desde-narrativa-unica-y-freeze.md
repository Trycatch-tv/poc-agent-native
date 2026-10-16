---
type: spike
id: WI-036
alias: KAN-POC5-WI-036
title: E1-005 Construcción desde narrativa única y freeze
knowledge_level: K2
status: draft
implementation_status: not-started
phase: draft
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
domains:
  - orchestrator
  - system-spec
  - semantic-ir
  - deterministic-toolchain
code:
  - runs/poc005-e1-narrative/**
  - runs/toolchain-freeze-poc005.sha256
---

# WI-036 (KAN-POC5-WI-036) — E1-005 Construcción desde narrativa única y freeze

## Outcome

- **Actor:** Agente y orquestador en ejercicio E1-005.
- **Target behavior:**
  - El orquestador toma la narrativa funcional `spec/narrative-005.md`, sintetiza la System Specification inicial, genera las Semantic IRs para los componentes y compila los componentes WebAssembly nativos.
  - Validación del 100 % de las pruebas públicas y reservadas en Wasmtime.
  - Registro de evidencia reproducible en `runs/poc005-e1-narrative/` y sellado del congelamiento de toolchain en `runs/toolchain-freeze-poc005.sha256`.
- **Observable completion:** Artefactos completos en `runs/poc005-e1-narrative/attempt-01/`, 100 % pass rate y toolchain freeze sellado.

## Acceptance Criteria

- [ ] Generación autónoma de System Specification e IRs desde narrativa única.
- [ ] 100 % de pruebas aprobadas en el runtime de componentes de Wasmtime.
- [ ] Reproducibilidad binaria 3/3 confirmada con SHA-256 idéntico.
- [ ] Hashes de toolchain sellados en `runs/toolchain-freeze-poc005.sha256`.
