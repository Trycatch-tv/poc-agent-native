---
type: spike
id: WI-011
alias: KAN-POC2-WI-011
title: E1-002 construcción y freeze
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
  - deterministic-toolchain
  - experiment-evidence
code:
  - ir/program-002.ir.json
  - runs/poc002-e1-build/**
  - runs/toolchain-freeze-poc002.sha256
---

# WI-011 (KAN-POC2-WI-011) — E1-002 construcción y freeze

## Outcome

- **Actor:** Agente evaluado y operador del experimento en `KADDO-AGENT-NATIVE-POC-002`.
- **Current behavior:** No existía implementación de la tarea de orden comercial v1 en IR v0.2 ni evidencia de ejecución de `E1-002`.
- **Target behavior:** El agente leyó `spec/task-002.yaml` (v1) e implementó en `ir/program-002.ir.json` la orden comercial (`calculate_order`), apoyándose en las subfunciones internas (`calculate_line_item`, `calculate_tax`) y usando records para agrupar campos de orden (`OrderCalculation { subtotal_cents, tax_cents, shipping_cents, total_cents }`). Aprobó verificación con 0 diagnósticos, compiló a `.wasm` (308 bytes), aprobó el 100 % de pruebas públicas (6/6) y reservadas (9/9), y selló el punto de congelamiento en `runs/toolchain-freeze-poc002.sha256` y tag git `poc-002-e1-freeze`.
- **Observable completion:** `program-002.wasm` generado y validado con `wasmparser`; 100 % de casos aprobados; artefactos completos en `runs/poc002-e1-build/attempt-01/`; y toolchain congelada.

## Scope

- Construcción de `ir/program-002.ir.json` conforme a `spec/task-002.yaml` (v1).
- Verificación con `irtool verify`, compilación con `irtool compile` y validación con `irtool validate`.
- Ejecución de pruebas públicas `tests/public-002.yaml` (`6/6`) y reservadas `tests/hidden-002.yaml` (`9/9`).
- Captura de artefactos de evidencia en `runs/poc002-e1-build/attempt-01/`.
- Congelamiento formal del toolchain en `runs/toolchain-freeze-poc002.sha256`.

## Acceptance Criteria

- [x] `ir/program-002.ir.json` es verificado por `irtool verify` con `0` diagnósticos (`exit 0`).
- [x] `irtool compile` emite `build/program-002.wasm` (308 bytes), que pasa validación estricta con `irtool validate`.
- [x] `runner` aprueba el 100 % de pruebas públicas (`6/6`) en `tests/public-002.yaml`.
- [x] `runner` aprueba el 100 % de pruebas reservadas (`9/9`) en `tests/hidden-002.yaml`.
- [x] Existen todos los artefactos de evidencia en `runs/poc002-e1-build/attempt-01/`.
- [x] Se registran los hashes del toolchain en `runs/toolchain-freeze-poc002.sha256`.

## Validation

- `irtool verify` ejecutado con 0 diagnósticos en 1 intento.
- `irtool compile` generó `build/program-002.wasm` (`308 bytes`, SHA-256 `f42bcb6e03c616b2891a9c4d1e060c4e141d502b01f44a326bc749d65a0d771a`).
- `runner test` sobre suite pública: 6/6 pasadas en 40 ms.
- `runner test` sobre suite reservada: 9/9 pasadas en 49 ms.
- Hashes del toolchain registrados en `runs/toolchain-freeze-poc002.sha256`.

## Learning

- **Qué se entregó:**
  1. `ir/program-002.ir.json`: Implementación completa de `calculate_line_item`, `calculate_tax` y `calculate_order` usando records y llamadas funcionales internas.
  2. `build/program-002.wasm`: Binario WebAssembly Core de solo 308 bytes con memoria lineal y llamadas internas.
  3. `runs/poc002-e1-build/attempt-01/`: Evidencia completa auditable (`run.json`, `spec.snapshot.yaml`, `ir.before.json`, `ir.after.json`, `diagnostics.json`, `public-tests.json`, `hidden-tests.json`, `artifact.sha256`, `program.wasm`, `program.wat`, `toolchain.json`, `agent-summary.md`).
  4. `runs/toolchain-freeze-poc002.sha256`: Hashes de `irtool.exe` y `runner.exe` sellados para los ejercicios `E2-002` a `E4-002`.
- **Decisiones técnicas:**
  1. La composición mediante llamadas internas puras (`call`) no requiere variables mutables locales; la IR se mantiene puramente expresiva y declarativa.
  2. La representación de estructuras en memoria lineal permite al runner inspeccionar directamente los offsets sin intermediarios pesados.
