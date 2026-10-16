---
type: spike
id: WI-014
alias: KAN-POC2-WI-014
title: E4-002 mantenimiento modular
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
  - runs/poc002-e4-maintain/**
---

# WI-014 (KAN-POC2-WI-014) — E4-002 mantenimiento modular

## Outcome

- **Actor:** Operador del experimento y agente evaluado en `KADDO-AGENT-NATIVE-POC-002`.
- **Current behavior:** La subfunción de impuestos o subtotal utiliza una tasa/fórmula v2.
- **Target behavior:** Se publica `spec/task-002.yaml` (v3) modificando la política en una subfunción interna (e.g. nueva tasa de impuesto o umbral de recargo). El agente realiza el mantenimiento editando únicamente el cuerpo de la subfunción interna en `ir/program-002.ir.json` con un diff conciso (pocas líneas), sin tocar la función principal ni el backend congelado, y aprueba el 100 % de las pruebas de regresión. Se ejecutan 3 compilaciones deterministas consecutivas comprobando la igualdad exacta de SHA-256.
- **Observable completion:** Evidencia en `runs/poc002-e4-maintain/attempt-01/`; 3/3 builds deterministas con mismo hash; diff conciso localizado en la subfunción.
