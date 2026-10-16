---
type: spike
id: WI-022
alias: KAN-POC3-WI-022
title: E4-003 Mantenimiento modularizado y auditabilidad
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
  - runs/poc003-e4-maintain/**
---

# WI-022 (KAN-POC3-WI-022) — E4-003 Mantenimiento modularizado y auditabilidad

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Current behavior:** El programa aplica la lógica de crédito y auditoría v2.
- **Target behavior:**
  - Se publica `spec/task-003.yaml` (v3) modificando el código de evento de auditoría para pedidos de alto valor (`HIGH_VALUE_AUDIT_EVENT = 999`) y actualizando un parámetro de cálculo interno.
  - El agente realiza el mantenimiento en un diff conciso localizado sin alterar otras funciones ni el backend.
  - Se verifica que tanto el resultado de negocio como los efectos secundarios registrados en el host mock coinciden al 100 %.
  - Se ejecutan 3 compilaciones deterministas consecutivas verificando igualdad bit a bit de SHA-256.
- **Observable completion:** Evidencia en `runs/poc003-e4-maintain/attempt-01/`; 3/3 builds con mismo hash; diff conciso; 100 % pass rate en runner.
