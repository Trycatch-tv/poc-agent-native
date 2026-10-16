---
type: spike
id: WI-009
alias: KAN-POC2-WI-009
title: Contrato experimental e IR v0.2
knowledge_level: K2
status: completed
implementation_status: completed
phase: now
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
ready_at: '2026-10-16'
completed_at: '2026-10-16'
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
---

# WI-009 (KAN-POC2-WI-009) — Contrato experimental e IR v0.2

## Outcome

- **Actor:** Operador del experimento y agente evaluado en `KADDO-AGENT-NATIVE-POC-002`.
- **Current behavior:** El repositorio contaba únicamente con los contratos de la POC 001 (`ir/ir.schema.json` v0.1).
- **Target behavior:** Existen en el repositorio el esquema formal JSON Draft 2020-12 de la IR v0.2 (`ir/ir.schema.v2.json`), el smoke test neutral (`ir/smoke-v2.ir.json`), la especificación de tarea v1 de POC 002 (`spec/task-002.yaml`) y las pruebas públicas (`tests/public-002.yaml`).
- **Observable completion:** `ir/ir.schema.v2.json` es válido, `ir/smoke-v2.ir.json` ejercita records y llamadas internas sin implementar la lógica de orden de negocio, y `spec/task-002.yaml` junto con `tests/public-002.yaml` definen los contratos para `E1-002`.

## Problem

Para iniciar la Fase 2 sin romper los artefactos congelados y auditados de la POC 001, se requería un esquema v0.2 independiente que introdujera tipos estructurados (`record`), invocación de funciones internas (`call`) y operadores de acceso/construcción, junto con la especificación del dominio comercial de órdenes de compra.

## Scope

- Diseñar y crear `ir/ir.schema.v2.json` con soporte para:
  - Tipos escalares (`i32`, `bool`) y tipos compuestos (`record` con campos tipados).
  - Funciones exportadas y funciones auxiliares privadas.
  - Operador `call` con verificación estricta de parámetros y tipos.
  - Operadores de registro (`record.construct`, `record.get`).
- Crear `spec/task-002.yaml` (versión 1) con los requerimientos funcionales de procesamiento de orden comercial (`ORD-001`, `ORD-002`, `ORD-003`, `ORD-004`).
- Crear suite pública `tests/public-002.yaml` con casos representativos de prueba.
- Crear smoke test neutral `ir/smoke-v2.ir.json`.

## Out of scope

- Extensión de `crates/irtool` y `crates/runner` (pertenece a `WI-010`).
- Implementación de la orden comercial de negocio en `ir/program-002.ir.json` (pertenece a `WI-011`, ejercicio `E1-002`).

## Acceptance Criteria

- [x] Existe `ir/ir.schema.v2.json` con JSON Schema Draft 2020-12 válido que define tipos compuestos (`record`), llamadas internas (`call`) y operadores de acceso/construcción.
- [x] Existe `ir/smoke-v2.ir.json` con un programa neutral válido que ejercita llamadas internas y records sin contener lógica de negocio de `E1-002`.
- [x] Existe `spec/task-002.yaml` (v1) declarando los requisitos de cálculo de orden comercial (`ORD-001`..`ORD-004`), tipos de record de entrada/salida y casos públicos de prueba.
- [x] Existe `tests/public-002.yaml` con los vectores de prueba públicos de `E1-002`.
- [x] Los artefactos de la POC 001 (`ir.schema.json`, `program.ir.json`, `spec/task.yaml`, `runs/`) permanecen intactos e inalterados.

## Validation

- Validado sintácticamente `ir/ir.schema.v2.json` y `ir/smoke-v2.ir.json` mediante Node.js `JSON.parse`.
- Validados `spec/task-002.yaml` y `tests/public-002.yaml`.
- Verificada la preservación intacta de los contratos de POC 001.

## Learning

- **Qué se entregó:**
  1. `ir/ir.schema.v2.json`: Esquema JSON Draft 2020-12 que introduce formalmente `recordType`, operador `call`, operador `record.construct` y operador `record.get`.
  2. `ir/smoke-v2.ir.json`: Smoke test neutral (`Point`, `PointSum`, `add_coords`, `make_point_sum`).
  3. `spec/task-002.yaml`: Especificación funcional v1 para el procesamiento de órdenes comerciales (`calculate_order`, `calculate_line_item`, `calculate_tax`).
  4. `tests/public-002.yaml`: Suite de pruebas públicas de aceptación con 6 vectores para `E1-002`.
- **Decisiones técnicas:**
  1. Se aisló completamente la IR v0.2 en archivos con sufijo `-002` o `v2` para garantizar que la reproducibilidad y auditoría de la POC 001 (`WI-001` a `WI-008`) permanezcan inalterables.
  2. Los campos de records se tipan con `valueType`, permitiendo componer registros o escalares.
