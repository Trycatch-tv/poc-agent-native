---
type: spike
id: WI-029
alias: KAN-POC4-WI-029
title: E3-004 Extensión funcional con interfaces y variantes ricas en WIT
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
  - wit/order.wit
  - ir/program-004.ir.json
  - runs/poc004-e3-extend/**
---

# WI-029 (KAN-POC4-WI-029) — E3-004 Extensión funcional con interfaces y variantes ricas en WIT

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Current behavior:** El componente sólo implementa la interfaz base de la v1.
- **Target behavior:**
  - Se publica `spec/task-004.yaml` (v2) y `wit/order.wit` ampliando el contrato (e.g. nueva función de cotización o variantes de error detalladas).
  - El agente extiende `ir/program-004.ir.json` en <= 3 intentos.
  - La toolchain congelada verifica y compila el componente con 0 cambios al backend.
  - 100 % de aprobación en suites ampliadas y regresión completa de POCs previas.
  - Los hashes de `runs/toolchain-freeze-poc004.sha256` se mantienen idénticos.
- **Observable completion:** Evidencia completa en `runs/poc004-e3-extend/` con toolchain congelada.
