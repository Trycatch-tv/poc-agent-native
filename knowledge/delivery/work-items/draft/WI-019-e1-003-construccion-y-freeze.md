---
type: spike
id: WI-019
alias: KAN-POC3-WI-019
title: E1-003 Construcción con Capabilities y Freeze
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
  - runs/poc003-e1-build/**
  - runs/toolchain-freeze-poc003.sha256
---

# WI-019 (KAN-POC3-WI-019) — E1-003 Construcción con Capabilities y Freeze

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Current behavior:** No existe programa en IR v0.3 con consumo de capabilities de host.
- **Target behavior:**
  - El agente construye `ir/program-003.ir.json` a partir de `spec/task-003.yaml` (v1).
  - El programa declara capabilities (`kv_store`, `audit_log`), consulta el descuento por cliente vía `kv_get(customer_id)`, calcula la orden con subfunciones internas (`calculate_line_item`, `calculate_tax`) y emite un evento de auditoría vía `emit_audit(event_id, total_cents)`.
  - Pasa verificación y pruebas públicas y reservadas al 100 %.
  - Se sella el congelamiento estricto de la toolchain en `runs/toolchain-freeze-poc003.sha256` y tag `poc-003-e1-freeze`.
- **Observable completion:** Evidencia completa en `runs/poc003-e1-build/attempt-01/`; 100 % pass rate; freeze sellado.
