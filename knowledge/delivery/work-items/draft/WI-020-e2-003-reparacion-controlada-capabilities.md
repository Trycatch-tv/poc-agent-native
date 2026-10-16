---
type: spike
id: WI-020
alias: KAN-POC3-WI-020
title: E2-003 Reparación controlada guiada por Capability diagnostics
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
  - ir/program-003.ir.json
  - runs/poc003-e2-repair/**
---

# WI-020 (KAN-POC3-WI-020) — E2-003 Reparación controlada guiada por Capability diagnostics

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Current behavior:** El programa construido en E1-003 es correcto y la toolchain está congelada.
- **Target behavior:**
  - El operador inyecta un defecto de autorización: invocación de una función de capability no autorizada (`unauthorized_host_op` o llamada sin declarar la capability en la cabecera `capabilities`).
  - `irtool verify` rechaza determinísticamente el programa con exit code 1 y diagnóstico JSONPath exacto (`E_UNDECLARED_CAPABILITY`).
  - El agente repara el defecto en un solo intento guiándose exclusivamente por el diagnóstico.
  - Se valida no-regresión contra el backend congelado y se recupera la paridad binaria.
- **Observable completion:** Evidencia en `runs/poc003-e2-repair/attempt-01/`; diagnóstico compacto < 600 bytes; reparación en 1 intento; toolchain inalterada.
