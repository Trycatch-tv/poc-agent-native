---
type: spike
id: WI-012
alias: KAN-POC2-WI-012
title: E2-002 reparación controlada
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
  - ir/program-002.ir.json
  - runs/poc002-e2-repair/**
---

# WI-012 (KAN-POC2-WI-012) — E2-002 reparación controlada

## Outcome

- **Actor:** Operador del experimento (fault injection) y agente evaluado en `KADDO-AGENT-NATIVE-POC-002`.
- **Current behavior:** `ir/program-002.ir.json` contiene la versión válida de `E1-002`.
- **Target behavior:** Se inyecta un defecto semántico controlado en un campo de record (`E_UNKNOWN_FIELD`) o en la firma de una llamada interna (`E_SIGNATURE_MISMATCH`). El verificador `irtool verify` rechaza la IR con exit code != 0 y emite diagnósticos estructurados en JSON con JSONPath explícito. El agente repara la IR en <= 2 intentos sin alterar el backend congelado y recuperando el 100 % de las pruebas.
- **Observable completion:** Evidencia en `runs/poc002-e2-repair/attempt-01/`; diagnóstico < 500 bytes; reparación exitosa en 1 intento.
