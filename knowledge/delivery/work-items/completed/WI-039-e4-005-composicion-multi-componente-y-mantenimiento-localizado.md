---
type: spike
id: WI-039
alias: KAN-POC5-WI-039
title: E4-005 Composición multi-componente y mantenimiento localizado
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
  - component-model
  - deterministic-toolchain
code:
  - runs/poc005-e4-composition/**
---

# WI-039 (KAN-POC5-WI-039) — E4-005 Composición multi-componente y mantenimiento localizado

## Outcome

- **Actor:** Agente y orquestador en ejercicio E4-005.
- **Target behavior:**
  - Ejecutar la aplicación compuesta con ambos WebAssembly Components interactuando a través de contratos WIT (`kaddo:shipping/shipping-service@0.1.0`).
  - Introducir un cambio de mantenimiento localizado exclusivamente en `shipping-calculator` (actualización de tarifas base de peso a 550c y 850c vía `WI-E4-01`).
  - El orquestador localiza y modifica únicamente la IR del componente afectado, recompila dicho componente y valida la integración global.
  - Verificar que el otro componente (`order-processor.wasm`) no experimenta cambios innecesarios y su hash SHA-256 permanece inalterado (`a075570a554a67251a66d24c2fbcd9c46569e4a88cd69d15230b3abdae9a8751` idéntico a E1).
  - Validar reproducibilidad 3/3 del componente modificado (`31bba5071664f51e60e303e321d32dfa94a1854baf9e268895d05afa6f9b36ba`).
- **Observable completion:** Artefactos en `runs/poc005-e4-composition/attempt-01/`, 100 % pass rate (9/9) en pruebas de integración (`tests/narrative-005-e4.yaml`), inalterabilidad del componente dependiente y reproducibilidad 3/3.

## Acceptance Criteria

- [x] Enlace cruzado de componentes funcionando de extremo a extremo.
- [x] Cambio localizado en un único componente con 0 modificaciones en el otro componente (`order-processor.wasm` SHA-256 inalterado).
- [x] Reproducibilidad binaria exacta (3/3 builds deterministas) en el componente modificado (`shipping-calculator.wasm`: `31bba507...`).
- [x] 100 % pass rate en suites de integración multi-componente (9/9 passed).
