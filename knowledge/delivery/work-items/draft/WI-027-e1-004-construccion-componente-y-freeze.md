---
type: spike
id: WI-027
alias: KAN-POC4-WI-027
title: E1-004 Construcción de Componente con Interfaces WIT y Freeze
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
  - runs/poc004-e1-build/**
  - runs/toolchain-freeze-poc004.sha256
---

# WI-027 (KAN-POC4-WI-027) — E1-004 Construcción de Componente con Interfaces WIT y Freeze

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Current behavior:** No existe implementación de la tarea 004 basada en componentes.
- **Target behavior:**
  - El agente construye de forma autónoma `ir/program-004.ir.json` a partir de `spec/task-004.yaml` (v1) y `wit/order.wit` en <= 3 intentos.
  - Verifica y emite el WebAssembly Component ejecutable con 0 diagnósticos.
  - 100 % de aprobación en suites públicas y reservadas en Wasmtime Component Runtime.
  - Reproducibilidad binaria exacta demostrada (3/3 builds deterministas idénticos).
  - Congelar la toolchain release en `runs/toolchain-freeze-poc004.sha256` y sellar con tag `poc-004-e1-freeze`.
- **Observable completion:** Artefactos completos en `runs/poc004-e1-build/attempt-01/`, freeze sellado y custodia criptográfica registrada.
