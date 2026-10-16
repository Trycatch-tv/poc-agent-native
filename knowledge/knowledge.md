---
type: current-state
updated_at: 2026-10-16
project_state: ai-assisted
refined_by: architecture-agent
---

# poc-agent-native — Knowledge

> What is true about this product right now.

## Purpose

`poc-agent-native` (`KADDO-AGENT-NATIVE-POC-001`) es un proyecto experimental de investigación técnica en modalidad **Kaddo POC Mode** que evalúa si un agente puede construir, reparar, extender y mantener un programa usando una **Semantic IR (`v0.1`)** como artefacto principal de software, mientras una toolchain determinista (`verifier`, `compiler` y `runner` sobre Wasmtime) verifica esa representación y la compila directamente a **WebAssembly Core Module (`.wasm`)**.

Sirve al equipo de investigación de Kaddo para validar o refutar con evidencia reproducible (**E1 Construcción**, **E2 Reparación**, **E3 Extensión** y **E4 Mantenimiento**) la viabilidad de separar una zona probabilística (especificación → IR mantenida por el agente) de una zona determinista (verificación → lowering a WASM → validación → ejecución de pruebas).

## Architecture overview

El experimento se divide en tres subsistemas explícitos:

1. **Kaddo POC Mode (`knowledge/`, `.kaddo/`):** Mantiene la hipótesis canónica (`knowledge/delivery/poc.md`), los Work Items de experimentación (`spike`/`chore`), la trazabilidad y la conclusión final (`GO` / `PIVOT` / `STOP`).
2. **Zona probabilística (`spec/task.yaml` → Agente → `ir/program.ir.json`):** El agente interpreta la especificación funcional y los diagnósticos estructurados (`diagnostics.json`) para producir o evolucionar exclusivamente la Semantic IR v0.1 (`ir/program.ir.json`).
3. **Zona determinista (`irtool` + `wasm-tools` + `runner` + `runs/`):**
   - **Verifier (`irtool verify`):** Valida esquema JSON (`ir/ir.schema.json`) y reglas semánticas (`E_UNKNOWN_PARAM`, `E_TYPE_MISMATCH`, `E_RETURN_TYPE`, etc.), emitiendo `diagnostics.json`.
   - **Compiler (`irtool compile`):** Realiza el lowering determinista de IR v0.1 directamente a binario WebAssembly (`build/program.wasm`).
   - **Validator (`wasm-tools validate`):** Verifica la validez independiente del binario `.wasm` (y opcionalmente genera `build/program.wat` solo para inspección).
   - **Runner (`runner test` con Wasmtime):** Ejecuta las suites de pruebas públicas y reservadas contra los exports WASM y registra la evidencia auditable en `runs/<exercise>/<attempt>/`.

## Key domains

- **`semantic-ir`:** Contrato de representación intermedia (`ir/ir.schema.json`, `ir/program.ir.json`) para funciones puras con tipos `i32` y `bool`, control `if`, comparaciones (`eq`, `lt_s`, `gte_s`), aritmética (`add`, `sub`, `mul`, `div_s`), booleanos (`and`, `or`, `not`) y trazabilidad a IDs de requisitos.
- **`deterministic-toolchain`:** Verificador semántico (`verifier`), compilador directo `IR → WASM` (`compiler`) y ejecutor de pruebas embebido en Wasmtime (`runner`).
- **`commercial-rules-subject`:** Dominio sujeto del experimento (`spec/task.yaml` v1–v3) compuesto por reglas puras de negocio en centavos enteros (`calculate_shipping` y `calculate_discount`).
- **`experiment-evidence`:** Captura reproducible de ejecuciones, diagnósticos, diffs de IR, hashes SHA-256 (`runs/toolchain-freeze.sha256`, `artifact.sha256`) y métricas (`M01`–`M20`) para evaluar los criterios de éxito `SC-01` a `SC-16`.

## Active constraints

- **Fuente única de verdad del negocio:** Toda la lógica de negocio del experimento debe residir en `ir/program.ir.json`. Está prohibido implementar un fallback en Rust/JS/TS/Python/Go o editar manualmente `.wat`/`.wasm` (**SC-12**, **SC-13**).
- **Congelamiento de toolchain (`freeze point` tras E1):** Después de aprobar E1, los binarios/código de `verifier`, `compiler` y `runner` se congelan; E2, E3 y E4 deben completarse sin modificar el backend (**SC-07**, **SC-09**).
- **Autonomía y límite de intentos:** Cero ediciones manuales humanas a `ir/program.ir.json` desde el inicio de E1 (**SC-02**) y máximo 2 intentos de reparación tras rechazo del verificador en E1/E2 (**SC-03**, **SC-05**).
- **Alcance técnico acotado (IR v0.1):** Funciones puras sin I/O, sin imports de host, sin memoria lineal dinámica, sin WASI ni servicios externos (bases de datos, HTTP, Kafka, filesystem).
- **Reproducibilidad determinista:** Tres compilaciones consecutivas de la misma IR con la misma toolchain deben producir exactamente el mismo hash SHA-256 de `program.wasm` (**SC-11**).

## Validated outcome (`KADDO-AGENT-NATIVE-POC-001`)

- **Status:** `validated` (`Decision: GO`).
- **Success Criteria:** `16/16` (`SC-01`–`SC-16`) cumplidos y respaldados en `runs/e1-build/`, `runs/e2-repair/`, `runs/e3-extend/` y `runs/e4-maintain/`.
- **Evidencia clave:**
  - `E1`: Construcción de `calculate_shipping` en IR v0.1 en 1 intento (`0` diagnósticos, `76 B` `.wasm`, `6/6` públicas, `9/9` reservadas) y congelamiento de toolchain (`poc-e1-freeze`, `runs/toolchain-freeze.sha256`).
  - `E2`: Rechazo determinista de defecto inyectado (`E_UNKNOWN_PARAM` en `$.functions[0].body.condition.left`) y reparación autónoma en 1 intento recuperando el mismo `.wasm` bit a bit.
  - `E3`: Extensión funcional con `calculate_discount(i32, bool) -> i32` en 1 intento sin modificar el backend congelado (`134 B` `.wasm`, `12/12` públicas, `18/18` reservadas).
  - `E4`: Mantenimiento localizado de `calculate_shipping` (`>= 4000 -> 0`, `< 4000 -> 600`) en 3 líneas de diff sin tocar `calculate_discount` ni el backend (`12/12` públicas, `18/18` reservadas, `3/3` compilaciones consecutivas con idéntico SHA-256).

