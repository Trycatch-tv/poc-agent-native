---
type: spike
id: WI-030
alias: KAN-POC4-WI-030
title: E4-004 Mantenimiento modularizado y composición interoperable
knowledge_level: K2
status: draft
implementation_status: not-started
phase: now
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
domains:
  - semantic-ir
  - commercial-rules-subject
  - deterministic-toolchain
  - experiment-evidence
code:
  - spec/task-004.yaml
  - ir/program-004.ir.json
  - runs/poc004-e4-maintain/**
---

# WI-030 (KAN-POC4-WI-030) — E4-004 Mantenimiento modularizado y composición interoperable

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Current behavior:** No se ha evaluado el mantenimiento modularizado de un WebAssembly Component en Semantic IR v0.4.
- **Target behavior:**
  - Se publica una actualización de reglas comerciales en `spec/task-004.yaml` (v3).
  - El agente realiza el mantenimiento con un diff conciso y localizado sin romper la interfaz WIT externa ni tocar el backend congelado.
  - La suite de pruebas alcanza 100 % de aprobación en Wasmtime Component Runtime.
  - Se demuestra reproducibilidad binaria exacta (3/3 builds deterministas idénticos).
- **Observable completion:** Evidencia en `runs/poc004-e4-maintain/` con diff modular y paridad binaria 3/3.
