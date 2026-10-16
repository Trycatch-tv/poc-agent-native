---
type: spike
id: WI-010
alias: KAN-POC2-WI-010
title: Verifier v0.2 + compiler IR v0.2 directo a WASM + runner
knowledge_level: K3
status: draft
implementation_status: not-started
phase: now
project_state: ai-assisted
refined_by: work-item-agent
created_at: '2026-10-16'
domains:
  - deterministic-toolchain
  - semantic-ir
  - experiment-evidence
code:
  - crates/irtool/**
  - crates/runner/**
  - tests/**
---

# WI-010 (KAN-POC2-WI-010) — Verifier v0.2 + compiler IR v0.2 directo a WASM + runner

## Outcome

- **Actor:** Operador del experimento y agente evaluado en `KADDO-AGENT-NATIVE-POC-002`.
- **Current behavior:** La toolchain actual (`irtool` y `runner`) solo soporta Semantic IR v0.1 (funciones escalares `i32`/`bool` sin memoria lineal ni llamadas internas).
- **Target behavior:** `crates/irtool` soporta verificación semántica estricta de IR v0.2 (validación de campos de record, firmas de `call`, type checking de llamadas y layouts de memoria) y lowering determinista directo a WebAssembly Core con sección `memory` y offsets precalculados; `crates/runner` soporta pasar y leer estructuras compuestas en memoria lineal de Wasmtime.
- **Observable completion:** `cargo run -p irtool -- verify --schema ir/ir.schema.v2.json --ir ir/smoke-v2.ir.json` verifica el smoke test con `0` diagnósticos; `cargo run -p irtool -- compile` emite un binario `.wasm` válido con layouts de record correctos; y `cargo run -p runner` ejecuta y aprueba los casos de smoke test compuestos.

## Scope

- Ampliar `crates/irtool`:
  - Verificador semántico para IR v0.2: validación de firmas de llamadas (`call`), verificación de acceso a campos (`record.get`), validación de constructores (`record.construct`), y emisión de diagnósticos JSON estructurados con códigos `E_UNKNOWN_FIELD`, `E_SIGNATURE_MISMATCH`, etc.
  - Compilador directo a WASM: emisión de WebAssembly Core con funciones privadas internas, sección `call`, emisión de `memory` estática para registros y lectura/escritura determinista de offsets.
- Ampliar `crates/runner`:
  - Soporte en el runner Wasmtime para mapear structs de entrada/salida desde/hacia memoria lineal de WASM según el esquema.
- Pruebas unitarias de verificación determinista en `irtool`.
