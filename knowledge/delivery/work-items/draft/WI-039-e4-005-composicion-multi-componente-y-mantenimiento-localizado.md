---
type: spike
id: WI-039
alias: KAN-POC5-WI-039
title: E4-005 Composición multi-componente y mantenimiento localizado
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
  - component-model
  - deterministic-toolchain
code:
  - runs/poc005-e4-composition/**
---

# WI-039 (KAN-POC5-WI-039) — E4-005 Composición multi-componente y mantenimiento localizado

## Outcome

- **Actor:** Agente y orquestador en ejercicio E4-005.
- **Target behavior:**
  - Ejecutar la aplicación compuesta con ambos WebAssembly Components interactuando a través de contratos WIT.
  - Introducir un cambio de mantenimiento localizado en uno de los dos componentes (ej. modificación de política tarifaria de envíos en `shipping-calculator.wasm`).
  - El orquestador localiza y modifica únicamente la IR del componente afectado, recompila dicho componente y valida la integración global.
  - Verificar que el otro componente (`order-processor.wasm`) no experimenta cambios innecesarios y su hash SHA-256 permanece inalterado.
  - Validar reproducibilidad 3/3 del componente modificado.
- **Observable completion:** Artefactos en `runs/poc005-e4-composition/attempt-01/`, 100 % pass rate en pruebas de integración, inalterabilidad del componente dependiente y reproducibilidad 3/3.

## Acceptance Criteria

- [ ] Enlace cruzado de componentes funcionando de extremo a extremo.
- [ ] Cambio localizado en un único componente con 0 modificaciones en el otro componente.
- [ ] Reproducibilidad binaria exacta (3/3 builds deterministas) en el componente modificado.
- [ ] 100 % pass rate en suites de integración multi-componente.
