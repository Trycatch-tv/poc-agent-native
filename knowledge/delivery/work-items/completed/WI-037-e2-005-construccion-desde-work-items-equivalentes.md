---
type: spike
id: WI-037
alias: KAN-POC5-WI-037
title: E2-005 Construcción desde Work Items equivalentes
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
  - runs/poc005-e2-workitems/**
---

# WI-037 (KAN-POC5-WI-037) — E2-005 Construcción desde Work Items equivalentes

## Outcome

- **Actor:** Agente y orquestador en ejercicio E2-005.
- **Target behavior:**
  - En un entorno de ejecución aislado, suministrar al orquestador una secuencia de Work Items (`spec/work-items-e2/`) que represente funcionalmente la misma aplicación construida en E1.
  - El orquestador consolida los Work Items en la System Specification, coordina la generación de IR y compila los componentes WASM.
  - Ejecutar la misma suite de pruebas externa y contratos WIT de E1.
  - Demostrar equivalencia funcional estricta (100 % pass rate: 9 públicas, 12 reservadas, 21/21 total) comprobando que dos formas de entrada (narrativa única vs secuencia de Work Items) convergen al mismo comportamiento verificado y al mismo hash binario reproducible.
- **Observable completion:** Artefactos completos en `runs/poc005-e2-workitems/attempt-01/`, 100 % pass rate y confirmación de SC-01 (`order-processor.wasm`: `a075570a...`, `shipping-calculator.wasm`: `b9ae1624...` idénticos bit a bit a E1).

## Acceptance Criteria

- [x] Secuencia de Work Items procesada por el orquestador consolidando la especificación.
- [x] 100 % pass rate en las mismas pruebas públicas y reservadas de E1 (21/21 passed).
- [x] Validación de SC-01 (equivalencia funcional confirmada e identidad binaria total).
