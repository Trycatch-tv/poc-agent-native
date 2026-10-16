---
type: spike
id: WI-021
alias: KAN-POC3-WI-021
title: E3-003 Extensión funcional con nueva regla de host
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
  - spec/task-003.yaml
  - ir/program-003.ir.json
  - runs/poc003-e3-extend/**
---

# WI-021 (KAN-POC3-WI-021) — E3-003 Extensión funcional con nueva regla de host

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Current behavior:** El programa sólo consulta descuento y emite evento de auditoría.
- **Target behavior:**
  - Se publica `spec/task-003.yaml` (v2) incorporando una regla de verificación de crédito: el programa debe consultar el saldo de crédito del cliente vía host (`check_credit(customer_id)`) y discriminar un nuevo estado de error si el pedido excede el saldo disponible.
  - El agente extiende `ir/program-003.ir.json` agregando la nueva capability e import en <= 3 intentos.
  - El compilador congelado compila la nueva IR sin modificaciones.
  - Se valida el 100 % de las pruebas previas y ampliadas.
- **Observable completion:** Evidencia en `runs/poc003-e3-extend/attempt-01/`; 100 % pass rate; hashes de toolchain idénticos al freeze point.
