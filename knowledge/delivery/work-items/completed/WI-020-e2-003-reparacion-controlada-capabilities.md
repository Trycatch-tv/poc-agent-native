---
type: spike
id: WI-020
alias: KAN-POC3-WI-020
title: E2-003 Reparación controlada guiada por Capability diagnostics
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
  - deterministic-toolchain
  - experiment-evidence
code:
  - ir/program-003.ir.json
  - runs/poc003-e2-repair/**
---

# WI-020 (KAN-POC3-WI-020) — E2-003 Reparación controlada guiada por Capability diagnostics

## Outcome

- **Actor:** Agente evaluado y operador del experimento.
- **Completed behavior:**
  - El operador inyectó un defecto de autorización en `runs/poc003-e2-repair/attempt-01/program-defective.ir.json`, eliminando la capability `audit_log` de las capacidades autorizadas en la raíz.
  - `irtool verify` (binario release congelado) rechazó determinísticamente el programa con exit code 1 y emitió el diagnóstico estructurado `E_UNDECLARED_CAPABILITY` señalando la ruta JSONPath exacta `$.imports[1].capability` y `$.functions[5].body.fields.status_code.condition.left.import` (tamaño del diagnóstico: 679 bytes formateado / < 500 bytes minificado).
  - El agente reparó el defecto en 1 solo intento en `runs/poc003-e2-repair/attempt-01/program-repaired.ir.json` restaurando la declaración de la capability `audit_log`.
  - La verificación pasó con 0 diagnósticos y compiló a WASM con paridad binaria exacta SHA-256 (`637ee58cf943616fa80db2d87abb53a22ec04755a7ba001b3fbd7e79ad003be7`).
  - Todas las pruebas (públicas y ocultas) alcanzaron 100 % de aprobación (15/15).
  - La toolchain permaneció 100 % inalterada (hashes idénticos a los custodiados en `runs/toolchain-freeze-poc003.sha256`).
- **Observable verification:**
  - `runs/poc003-e2-repair/attempt-01/defect-diagnostics.json` (exit code 1, `E_UNDECLARED_CAPABILITY`).
  - `runs/poc003-e2-repair/attempt-01/repaired-diagnostics.json` (0 diagnósticos).
  - `runs/poc003-e2-repair/attempt-01/public-tests.json` (6/6 passed, 5 ms).
  - `runs/poc003-e2-repair/attempt-01/hidden-tests.json` (9/9 passed, 6 ms).
  - Hashes de toolchain congelada verificados.
