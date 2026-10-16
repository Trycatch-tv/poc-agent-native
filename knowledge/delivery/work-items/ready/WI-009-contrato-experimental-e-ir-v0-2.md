---
type: spike
id: WI-009
alias: KAN-POC2-WI-009
title: Contrato experimental e IR v0.2
knowledge_level: K2
status: ready
implementation_status: not-started
phase: now
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
domains:
  - semantic-ir
  - commercial-rules-subject
  - experiment-evidence
code:
  - spec/**
  - ir/**
  - tests/**
  - runs/**
  - build/**
ready_at: '2026-10-16'
---

# WI-009 (KAN-POC2-WI-009) — Contrato experimental e IR v0.2

## Outcome

- **Actor:** Operador del experimento y agente evaluado en `KADDO-AGENT-NATIVE-POC-002`.
- **Current behavior:** El repositorio cuenta con la definición canónica de `KADDO-AGENT-NATIVE-POC-002` en `knowledge/delivery/poc.md`, pero aún no existen las especificaciones contractuales de la versión 0.2 de la Semantic IR (`ir/ir.schema.v2.json`), el contrato de tipos compuestos (`record`), llamadas internas (`call`), errores tipados (`Result<T, E>`), ni la especificación funcional v1 (`spec/task-002.yaml`) y casos de prueba base (`tests/public-002.yaml`).
- **Target behavior:** Existen en el repositorio el esquema formal JSON Draft 2020-12 de la IR v0.2, el smoke test neutral para registros y llamadas internas, la especificación de tarea v1 de POC 002 (cálculo de orden comercial con `OrderItem` y `OrderCalculation`) y las pruebas públicas de aceptación.
- **Observable completion:** `ir/ir.schema.v2.json` es válido, `spec/task-002.yaml` y `tests/public-002.yaml` definen los contratos funcionales de `E1-002`, y el smoke test valida la estructura sintáctica de records y llamadas sin implementar aún la lógica de negocio del ejercicio.

## Problem

Para iniciar la Fase 2 sin romper los artefactos congelados y auditados de la POC 001, se requiere un esquema v0.2 independiente que introduzca tipos estructurados (`record`), invocación de funciones internas (`call`) y variantes de resultado, junto con la especificación del dominio comercial de órdenes de compra.

## Scope

- Diseñar y crear `ir/ir.schema.v2.json` con soporte para:
  - Tipos escalares (`i32`, `bool`) y tipos compuestos (`record` con campos tipados).
  - Funciones exportadas y funciones auxiliares privadas.
  - Operador `call` con verificación estricta de parámetros y tipos.
  - Operadores de registro (`record.construct`, `record.get`).
- Crear `spec/task-002.yaml` (versión 1) con los requerimientos funcionales de procesamiento de orden comercial (`ORD-001`, `ORD-002`, `ORD-003`).
- Crear suite pública `tests/public-002.yaml` con casos representativos de prueba.
- Crear smoke test neutral `ir/smoke-v2.ir.json`.

## Out of scope

- Extensión de `crates/irtool` y `crates/runner` (pertenece a `WI-010`).
- Implementación de la orden comercial de negocio en `ir/program-002.ir.json` (pertenece a `WI-011`, ejercicio `E1-002`).

## Acceptance Criteria

- [ ] Existe `ir/ir.schema.v2.json` con JSON Schema Draft 2020-12 válido que define tipos compuestos (`record`), llamadas internas (`call`) y operadores de acceso/construcción.
- [ ] Existe `ir/smoke-v2.ir.json` con un programa neutral válido que ejercita llamadas internas y records sin contener lógica de negocio de `E1-002`.
- [ ] Existe `spec/task-002.yaml` (v1) declarando los requisitos de cálculo de orden comercial (`ORD-001`, `ORD-002`, `ORD-003`), tipos de record de entrada/salida y casos públicos de prueba.
- [ ] Existe `tests/public-002.yaml` con los vectores de prueba públicos de `E1-002`.
- [ ] Los artefactos de la POC 001 (`ir.schema.json`, `program.ir.json`, `spec/task.yaml`, `runs/`) permanecen intactos e inalterados.

## Validation

- Validar sintácticamente los archivos JSON mediante `node -e "JSON.parse(fs.readFileSync('ir/ir.schema.v2.json'))"` y `JSON.parse(fs.readFileSync('ir/smoke-v2.ir.json'))`.
- Validar formato de `spec/task-002.yaml` y `tests/public-002.yaml`.
- Ejecutar verificación de readiness mediante `kaddo_mark_work_item_ready`.
