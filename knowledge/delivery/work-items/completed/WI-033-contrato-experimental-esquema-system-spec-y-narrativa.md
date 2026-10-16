---
type: spike
id: WI-033
alias: KAN-POC5-WI-033
title: Contrato experimental, esquema System Specification y narrativa base
knowledge_level: K2
status: completed
implementation_status: completed
phase: completed
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
ready_at: '2026-10-16'
completed_at: '2026-10-16'
domains:
  - semantic-ir
  - system-spec
  - commercial-rules-subject
  - deterministic-toolchain
code:
  - spec/narrative-005.md
  - spec/system-spec.schema.json
  - spec/system-spec.json
  - wit/order-processor.wit
  - wit/shipping-calculator.wit
  - tests/narrative-005.yaml
---

# WI-033 (KAN-POC5-WI-033) — Contrato experimental, esquema System Specification y narrativa base

## Outcome

- **Actor:** Investigador principal y agente evaluado en `KADDO-AGENT-NATIVE-POC-005`.
- **Completed behavior:**
  - Se formalizó la narrativa funcional completa de la aplicación comercial en `spec/narrative-005.md` en lenguaje natural estructurado (Markdown), describiendo procesos, entidades, reglas de cálculo de orden y flete, interfaces y criterios de aceptación, con separación explícita en dos componentes autónomos (`order-calculator` y `shipping-calculator`).
  - Se diseñó y formalizó el esquema JSON Schema Draft 2020-12 de la **System Specification** (`spec/system-spec.schema.json`), capaz de representar requisitos funcionales con identificadores estables (`ORD-*`, `SHIP-*`, `LINE-*`, `TAX-*`, `DISC-*`, `CREDIT-*`, `AUDIT-*`), operaciones, entidades/tipos, contratos de interfaz y dependencias entre componentes.
  - Se generó la especificación consolidada inicial `spec/system-spec.json` derivada de la narrativa.
  - Se formalizaron los contratos WIT independientes para la aplicación multi-componente:
    - Componente de cálculo de fletes: `wit/shipping-calculator.wit` (world `shipping-component`, export `shipping-service`).
    - Componente de procesamiento de órdenes: `wit/order-processor.wit` (world `order-processor`, import `shipping-service`, import `host-services`, export `order-calculator`).
  - Se definió la suite de pruebas de referencia `tests/narrative-005.yaml` con 9 casos cubriendo validaciones de líneas, fletes, descuentos, impuestos, crédito y eventos de auditoría.
- **Observable completion:** `spec/narrative-005.md`, `spec/system-spec.schema.json`, `spec/system-spec.json`, `wit/shipping-calculator.wit`, `wit/order-processor.wit` y `tests/narrative-005.yaml` creados y validados.

## Acceptance Criteria

- [x] Narrativa funcional `spec/narrative-005.md` publicada en Markdown describiendo el dominio comercial completo y la separación en dos componentes (`order-calculator` y `shipping-calculator`).
- [x] Esquema `spec/system-spec.schema.json` formalizado en JSON Schema Draft 2020-12 soportando requisitos, entidades, operaciones y contratos.
- [x] Contratos WIT independientes formalizados para ambos componentes WebAssembly.
- [x] Suite de pruebas externa `tests/narrative-005.yaml` formulada con casos de cálculo, validación de líneas, impuestos, envíos y auditoría.
- [x] Instancia inicial `spec/system-spec.json` consolidada y validada.
