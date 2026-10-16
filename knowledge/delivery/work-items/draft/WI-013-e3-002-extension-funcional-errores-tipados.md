---
type: spike
id: WI-013
alias: KAN-POC2-WI-013
title: E3-002 extensión funcional con errores tipados
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
  - spec/task-002.yaml
  - ir/program-002.ir.json
  - runs/poc002-e3-extend/**
---

# WI-013 (KAN-POC2-WI-013) — E3-002 extensión funcional con errores tipados

## Outcome

- **Actor:** Operador del experimento y agente evaluado en `KADDO-AGENT-NATIVE-POC-002`.
- **Current behavior:** La orden procesa casos válidos o usa retornos simples, sin errores de dominio tipados y discriminados.
- **Target behavior:** Se publica `spec/task-002.yaml` (v2) introduciendo validaciones de negocio que retornan un tipo resultado tipado `Result<OrderCalculation, OrderError>` con variantes explícitas (`InvalidQuantity`, `ExceedsMaxAmount`). El agente extiende `ir/program-002.ir.json` en <= 3 intentos sin modificar el backend congelado, y aprueba tanto las nuevas pruebas de error como el 100 % de las pruebas previas de regresión.
- **Observable completion:** Artefactos completos en `runs/poc002-e3-extend/attempt-01/`; hashes de toolchain inalterados; 100 % pass rate.
