---
type: spike
id: WI-038
alias: KAN-POC5-WI-038
title: E3-005 Evolución incremental guiada por nuevo Work Item
knowledge_level: K2
status: draft
implementation_status: not-started
phase: draft
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
domains:
  - orchestrator
  - system-spec
  - semantic-ir
  - deterministic-toolchain
code:
  - runs/poc005-e3-evolution/**
---

# WI-038 (KAN-POC5-WI-038) — E3-005 Evolución incremental guiada por nuevo Work Item

## Outcome

- **Actor:** Agente y orquestador en ejercicio E3-005.
- **Target behavior:**
  - Suministrar un nuevo Work Item que modifica una regla comercial existente e incorpora una nueva capacidad en la aplicación.
  - El orquestador actualiza la System Specification vigente identificando requisitos obsoletos y nuevos.
  - El orquestador localiza los nodos de IR afectados y coordina la modificación incremental de la Semantic IR sin reconstrucción innecesaria del programa.
  - Compilación con la toolchain congelada y verificación de 100 % de pruebas nuevas y de regresión.
- **Observable completion:** Artefactos en `runs/poc005-e3-evolution/attempt-01/`, diff quirúrgico de IR, inmutabilidad de toolchain freeze y 100 % pass rate.

## Acceptance Criteria

- [ ] System Specification actualizada versionando requisitos vigentes y modificados.
- [ ] Modificación de IR localizada exclusivamente en las reglas afectadas.
- [ ] Inmutabilidad del backend congelado (hashes de toolchain idénticos).
- [ ] 100 % de aprobación en pruebas nuevas y de regresión.
