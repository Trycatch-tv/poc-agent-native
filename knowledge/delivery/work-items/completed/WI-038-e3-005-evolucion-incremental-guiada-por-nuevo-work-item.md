---
type: spike
id: WI-038
alias: KAN-POC5-WI-038
title: E3-005 Evolución incremental guiada por nuevo Work Item
knowledge_level: K2
status: completed
implementation_status: completed
phase: completed
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
closed_at: '2026-10-16'
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
  - Suministrar un nuevo Work Item (`spec/work-items-e3/WI-E3-01-actualizacion-tasa-iva-18.md`) que modifica una regla comercial existente (TAX-001: IVA al 18 %).
  - El orquestador actualiza la System Specification vigente identificando requisitos modificados (`TAX-001` v2) y componentes afectados (`order-processor`).
  - El orquestador localiza los nodos de IR afectados y coordina la modificación incremental sin afectar `shipping-calculator` (`sha256=b9ae1624...` idéntico a E1).
  - Compilación con la toolchain congelada y verificación de 100 % de pruebas nuevas y de regresión (9/9 passed en `tests/narrative-005-e3.yaml`).
- **Observable completion:** Artefactos en `runs/poc005-e3-evolution/attempt-01/`, diff quirúrgico de especificación (`spec-diff.json`), inmutabilidad de toolchain freeze y 100 % pass rate.

## Acceptance Criteria

- [x] System Specification actualizada versionando requisitos vigentes y modificados (`TAX-001` v2).
- [x] Modificación de IR localizada exclusivamente en las reglas afectadas (`order-processor` re-compilado, `shipping-calculator` 100 % intacto).
- [x] Inmutabilidad del backend congelado (hashes de toolchain idénticos).
- [x] 100 % de aprobación en pruebas nuevas y de regresión (9/9 passed).
