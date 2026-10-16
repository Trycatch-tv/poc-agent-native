---
type: spike
id: WI-028
alias: KAN-POC4-WI-028
title: E2-004 Reparación controlada guiada por WIT diagnostics
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
  - experiment-evidence
code:
  - ir/program-004.ir.json
  - runs/poc004-e2-repair/**
---

# WI-028 (KAN-POC4-WI-028) — E2-004 Reparación controlada guiada por WIT diagnostics

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Current behavior:** No se ha comprobado la capacidad de diagnóstico ante desalineaciones de contrato WIT.
- **Target behavior:**
  - El operador inyecta un defecto en la IR v0.4 (e.g. tipo incompatible en un campo de la interfaz exportada WIT).
  - La toolchain congelada rechaza determinísticamente el programa con exit code != 0 y emite diagnóstico estructurado (`E_WIT_TYPE_MISMATCH` con JSONPath accionable < 600 B).
  - El agente repara el defecto en <= 2 intentos restaurando el 100 % pass rate y la paridad de hash SHA-256.
  - La toolchain permanece 100 % inmutable.
- **Observable completion:** Evidencia en `runs/poc004-e2-repair/` con diagnóstico conciso y paridad binaria restaurada.
