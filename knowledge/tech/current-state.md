---
type: current-state
generated_by: kaddo-init
template_version: 1
project_state: ai-assisted
refined_by: architecture-agent
updated_at: 2026-10-16
---

# Current State

Generado a partir del Kaddo Context Pack y del artefacto canónico `knowledge/delivery/poc.md`.

## System Overview

`poc-agent-native` (`KADDO-AGENT-NATIVE-POC-001`) es un repositorio experimental en modalidad **Kaddo POC Mode** diseñado para investigar si un agente puede construir, reparar, extender y mantener reglas de negocio puras operando exclusivamente sobre una **Semantic IR (`v0.1`)**, mientras una toolchain determinista en Rust valida esa IR, la compila directamente a **WebAssembly (`.wasm`)** y ejecuta suites de pruebas públicas y reservadas en **Wasmtime**.

En el estado actual (`WI-001` a `WI-005` implementados), los ejercicios **`E1` (Construcción inicial)**, **`E2` (Diagnóstico y reparación controlada)** y **`E3` (Extensión funcional)** han sido completados con éxito. En `E3`, el agente extendió `ir/program.ir.json` a `spec_version: 2` añadiendo `calculate_discount(subtotal_cents: i32, premium: bool) -> i32` (`DISC-001`, `DISC-002`, `DISC-003`) sin modificar `calculate_shipping` ni el backend congelado (`runs/toolchain-freeze.sha256`, **SC-07**), aprobando `12/12` pruebas públicas y `18/18` pruebas reservadas en el primer intento (`.wasm` de `134 bytes` con `sha256=e9450f963e606588e4c01c7b2860e1735598dd0f040ed221a62f11de42211e02`, **SC-06**). La evidencia completa reside en `runs/e1-build/attempt-01/`, `runs/e2-repair/attempt-01/` y `runs/e3-extend/attempt-01/`, y el sistema está listo para el ejercicio **`E4` (`WI-006` — Mantenimiento)**.

## Modules

Módulos previstos dentro del monorepo experimental según `knowledge/delivery/poc.md`:

- **`spec/` (Especificación funcional):** Contiene `spec/task.yaml` (versionado de v1 a v3 por el operador) con firmas exportadas, requisitos (`SHIP-*`, `DISC-*`) y casos públicos de aceptación.
- **`ir/` (Semantic IR v0.1):** Contiene el esquema formal `ir/ir.schema.json` y el artefacto principal mantenido por el agente (`ir/program.ir.json`).
- **`irtool` (Verifier + Compiler en Rust):**
  - Subcomando `verify`: valida `ir/program.ir.json` contra `ir/ir.schema.json` y aplica las 11 reglas semánticas (`E_DUPLICATE_FUNCTION`, `E_DUPLICATE_PARAM`, `E_UNKNOWN_PARAM`, `E_TYPE_MISMATCH`, `E_IF_CONDITION`, `E_IF_BRANCH_TYPE`, `E_RETURN_TYPE`, `E_UNSUPPORTED_OP`, `E_CONST_TYPE`, `E_CONST_DIV_ZERO`, `E_MISSING_REQUIREMENT`), emitiendo `diagnostics.json`.
  - Subcomando `compile`: realiza el lowering determinista `IR v0.1 → build/program.wasm` sin pasar por código fuente tradicional.
- **`runner` (Test Runner en Rust + Wasmtime):** Carga `build/program.wasm`, invoca las funciones exportadas contra `tests/public.yaml` y la suite reservada, y produce `public-tests.json` y `hidden-tests.json`.
- **`runs/` (Evidencia reproducible):** Almacena por cada ejercicio (`e1-build`, `e2-repair`, `e3-extend`, `e4-maintain`) y cada intento (`attempt-XX`) los snapshots, diagnósticos, artefactos `.wasm`/`.wat`, hashes SHA-256 y `run.json`.

## Dependencies and Integrations

- **Toolchain prevista (observada en especificación de `poc.md`):**
  - **Rust / Cargo:** Implementación determinista de `irtool` (`verifier` + `compiler`, p. ej. con `wasm-encoder`) y `runner`.
  - **Wasmtime:** Runtime embebido en `runner` para ejecutar módulos WebAssembly Core sin WASI ni capacidades de host.
  - **`wasm-tools`:** Validación independiente del binario (`wasm-tools validate`) e inspección opcional en texto (`wasm-tools print`).
  - **Kaddo CLI & MCP (`@kaddo/cli`, `@kaddo/mcp`):** Gestión de estado de la POC, Work Items de experimento, evidencia y reporte final (`kaddo poc report`).
- **Integraciones externas:** Ninguna (`not-applicable` por restricción explícita de la POC: sin HTTP, bases de datos, colas ni servicios remotos).

## Data Stores

- **Basado en archivos en el repositorio:**
  - Estado del programa: `ir/program.ir.json` (JSON estructurado).
  - Evidencia experimental: `runs/<exercise>/<attempt>/run.json`, `diagnostics.json`, `public-tests.json`, `hidden-tests.json` y hashes SHA-256.
- No se utilizan bases de datos relacionales, NoSQL ni almacenamiento en nube.

## Infrastructure

- Ejecución 100 % local y determinista en estación de trabajo (Windows / multiplataforma) mediante CLI (`cargo`, `wasm-tools`, `kaddo`).
- Sin contenedores, servicios en nube ni demonios requeridos para `POC 001`.

## Implicit Decisions (candidates)

1. **[Candidata] Emisión binaria directa de WASM desde Rust (`wasm-encoder`) frente a generación vía WAT:** `poc.md` establece que el compilador emite `.wasm` directamente y reserva `IR → WAT → wasm-tools parse → WASM` únicamente como *PIVOT técnico explícito* de contingencia si surgiera un bloqueo de tooling ajeno a la hipótesis H1.
2. **[Candidata] Representación de `bool` en la frontera WASM como `i32` (`0 = false`, `1 = true`):** Permite mantener WebAssembly Core Module puro (`i32`) mientras el verificador de la IR exige tipado booleano estricto en condiciones (`E_IF_CONDITION`) y operadores lógicos (`and`, `or`, `not`).
3. **[Candidata] Convención de errores de dominio mediante centavos negativos (`-1`):** Simplifica la firma de retorno a un único `i32` en `IR v0.1` antes de investigar errores tipados en una eventual `POC 002`.

## Open Questions

- `[resuelta]` ¿El operador `eq` en `IR v0.1` acepta tanto `(i32, i32)` como `(bool, bool)`? **Resolución:** Sí, documentado en la tabla de tipado de `knowledge/delivery/poc.md` para soportar expresiones como `premium == false` en `E3` sin ambigüedad.
- `[asumida]` Los vectores exactos de las pruebas reservadas (`hidden-tests`) residirán fuera del directorio expuesto al contexto del agente durante `E1`–`E4`, y solo se registrará su resumen de aprobación en `runs/<exercise>/<attempt>/hidden-tests.json`.

## Areas Requiring Human Validation

- Confirmar si se desea materializar ahora el primer Work Item experimental (`KAN-POC-WI-001` — Contrato experimental e IR v0.1) mediante `kaddo create spike` para iniciar formalmente la fase de experimentación en Kaddo.
