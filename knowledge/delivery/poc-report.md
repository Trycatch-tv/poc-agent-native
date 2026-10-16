---
type: poc-report
version: 1
conclusion: validated
source_fingerprint: 4a578887e54fc9997ada480e97335dc08c27b6672c306fdee049b32b32939866
created_at: '2026-10-16'
---

# Kaddo Agent-Native POC 001 â€” Final Report (`v001`)

> Reporte canÃ³nico sincronizado con [knowledge/delivery/poc-report-v001.md](knowledge/delivery/poc-report-v001.md).

## Executive Summary

| Atributo | Valor observado / verificado |
|---|---|
| **Experimento** | `KADDO-AGENT-NATIVE-POC-001` â€” Semantic IR v0.1 compilada directamente a WebAssembly |
| **ConclusiÃ³n canÃ³nica** | **`validated`** |
| **DecisiÃ³n recomendada** | **`GO`** |
| **Success Criteria cumplidos** | `16 / 16` (`SC-01` a `SC-16`) |
| **Ejercicios completados** | `E1` (ConstrucciÃ³n y freeze), `E2` (ReparaciÃ³n controlada), `E3` (ExtensiÃ³n funcional), `E4` (Mantenimiento localizado) |
| **Ediciones manuales humanas a la IR** | `0` (medido) |
| **Cambios al backend tras el freeze (`E1`)** | `0` (`backend_modified_since_freeze = false`, medido) |
| **Reproducibilidad binaria (`E4`)** | `3 / 3` compilaciones consecutivas idÃ©nticas (`sha256 = 947620fe23636397ba820b7cadf673e95fb3e09f68591bd0b54bfc8183d0a450`, medido) |

`KADDO-AGENT-NATIVE-POC-001` validÃ³ empÃ­ricamente que un agente puede construir (`E1`), reparar a partir de diagnÃ³sticos deterministas (`E2`), extender con nuevas funciones (`E3`) y mantener reglas comerciales existentes (`E4`) operando exclusivamente sobre una representaciÃ³n intermedia semÃ¡ntica (`ir/program.ir.json`, Semantic IR v0.1), mientras un micro-toolchain determinista en Rust (`irtool` + `runner` sobre Wasmtime) verifica la semÃ¡ntica y emite directamente mÃ³dulos binarios WebAssembly (`.wasm`) sin pasar por cÃ³digo fuente tradicional de negocio ni requerir modificaciones posteriores al congelamiento (`poc-e1-freeze`).

## Problem and Objective

En el desarrollo asistido por IA actual, los agentes operan sobre lenguajes fuente diseÃ±ados para humanos (Rust, TypeScript, Python, Go), donde una fracciÃ³n significativa del esfuerzo de inferencia y revisiÃ³n se consume en sintaxis accidental, boilerplate y diagnÃ³sticos orientados a ediciÃ³n manual.

El objetivo de `KADDO-AGENT-NATIVE-POC-001` fue comprobar, dentro de un dominio acotado de reglas comerciales puras sin I/O, si una **Semantic IR v0.1** explÃ­cita, pequeÃ±a y fuertemente tipada puede actuar como **artefacto primario de software mantenido por el agente**, delegando la verificaciÃ³n estructural/semÃ¡ntica, la compilaciÃ³n directa a `.wasm` y la ejecuciÃ³n de pruebas a una frontera 100 % determinista.

## Hypothesis and Success Criteria

- **HipÃ³tesis principal (`H1`):** Para funciones puras de reglas de negocio sin I/O, un agente puede operar sobre una Semantic IR v0.1 como artefacto principal de software y, apoyado en un verificador determinista, un compilador directo `IR â†’ WASM` y un runner de pruebas en Wasmtime, completar construcciÃ³n (`E1`), reparaciÃ³n controlada (`E2`), extensiÃ³n (`E3`) y mantenimiento (`E4`) sin editar cÃ³digo fuente tradicional ni modificar el compilador despuÃ©s del punto de congelamiento.

| ID | Criterio | Estado | Evidencia medida |
|---|---|---|---|
| `SC-01` | E1 correcciÃ³n (`100 %` pÃºblicas y reservadas) | Cumplido (`[x]`) | `6/6` pÃºblicas y `9/9` reservadas en `runs/e1-build/attempt-01/` |
| `SC-02` | E1 autonomÃ­a (`0` ediciones manuales humanas a IR) | Cumplido (`[x]`) | `manual_ir_edits = 0` en `runs/e1-build/attempt-01/run.json` |
| `SC-03` | LÃ­mite de reparaciÃ³n inicial en E1 (`<= 2` intentos) | Cumplido (`[x]`) | `repair_attempts = 0` (aprobado en el primer intento) |
| `SC-04` | E2 rechazo determinista (`exit != 0`, cÃ³digo y ruta) | Cumplido (`[x]`) | `exit code = 1`, `E_UNKNOWN_PARAM` en `$.functions[0].body.condition.left` (`diagnostics.injected.json`) |
| `SC-05` | E2 reparaciÃ³n (`<= 2` intentos, `100 %` pruebas E1) | Cumplido (`[x]`) | Reparado en `1` intento; `6/6` pÃºblicas + `9/9` reservadas (`runs/e2-repair/attempt-01/`) |
| `SC-06` | E3 extensiÃ³n (`calculate_discount` + regresiÃ³n) | Cumplido (`[x]`) | `12/12` pÃºblicas + `18/18` reservadas (`runs/e3-extend/attempt-01/`) |
| `SC-07` | E3 estabilidad de toolchain post-freeze | Cumplido (`[x]`) | Hashes de `irtool.exe` y `runner.exe` idÃ©nticos a `runs/toolchain-freeze.sha256` |
| `SC-08` | E4 mantenimiento (`calculate_shipping` v3 + regresiÃ³n) | Cumplido (`[x]`) | `12/12` pÃºblicas + `18/18` reservadas (`runs/e4-maintain/attempt-01/`) |
| `SC-09` | E4 estabilidad de toolchain post-freeze | Cumplido (`[x]`) | Hashes de `irtool.exe` y `runner.exe` idÃ©nticos a `runs/toolchain-freeze.sha256` |
| `SC-10` | Validez WASM (`wasmparser` / `irtool validate`) | Cumplido (`[x]`) | `wasm_valid = true` en `E1`, `E2`, `E3` y `E4` |
| `SC-11` | Reproducibilidad (`3/3` compilaciones idÃ©nticas en E4) | Cumplido (`[x]`) | `3/3` builds con `sha256 = 947620fe23636397ba820b7cadf673e95fb3e09f68591bd0b54bfc8183d0a450` |
| `SC-12` | Fuente de verdad (sin ediciones manuales a WAT/WASM) | Cumplido (`[x]`) | Todo `.wasm` y `.wat` fue emitido por `irtool compile` desde `ir/program.ir.json` |
| `SC-13` | Aislamiento de negocio (sin implementaciÃ³n paralela) | Cumplido (`[x]`) | `crates/irtool` y `crates/runner` son genÃ©ricos para IR v0.1 |
| `SC-14` | Evidencia completa en `runs/<exercise>/<attempt>/` | Cumplido (`[x]`) | Todos los artefactos requeridos existen para `E1`â€“`E4` |
| `SC-15` | Trazabilidad funciÃ³n/caso â†’ requirement ID | Cumplido (`[x]`) | `SHIP-001..003` y `DISC-001..003` trazados en IR y reportes de `runner` |
| `SC-16` | Cero bypass de pruebas, verifier o especificaciÃ³n | Cumplido (`[x]`) | NingÃºn criterio ni caso esperado fue relajado |

## Scope, Constraints and Non-goals

- **Alcance tÃ©cnico incluido (`IR v0.1`):** Tipos escalares `i32` y `bool`, funciones exportadas puras, literales (`const`), referencias a parÃ¡metros (`param`), condicional ternario (`if`), comparaciones (`eq`, `lt_s`, `gte_s`), aritmÃ©tica entera con signo (`add`, `sub`, `mul`, `div_s`), operadores booleanos (`and`, `or`, `not`), trazabilidad por `requirements` y compilaciÃ³n directa a WebAssembly Core Module.
- **Restricciones activas:** Cero ediciones manuales a `ir/program.ir.json`, `.wat` o `.wasm`; congelamiento estricto del backend (`irtool` y `runner`) tras `E1` (`poc-e1-freeze`); ejecuciÃ³n determinista sin I/O ni capacidades de host.
- **Non-goals respetados:** No se diseÃ±Ã³ un lenguaje generalista completo, no se incluyÃ³ memoria lineal dinÃ¡mica ni strings/colecciones, no se integraron servicios externos (HTTP, bases de datos, colas) ni WASI, y no se ejecutÃ³ aÃºn el benchmark comparativo pareado frente a Rust (`KAN-POC-WI-008`, diferido por diseÃ±o a la segunda fase).

## Technical Approach and Architecture

```mermaid
flowchart LR
    SPEC["spec/task.yaml (v1..v3)"] --> AGENT["Agente evaluado"]
    DIAG["diagnostics.json"] --> AGENT
    AGENT --> IR["ir/program.ir.json (IR v0.1)"]
    IR --> VERIFY["irtool verify (Schema + 11 reglas semÃ¡nticas)"]
    VERIFY -->|"falla (exit 1)"| DIAG
    VERIFY -->|"pasa (exit 0)"| COMPILE["irtool compile (wasm-encoder)"]
    COMPILE --> WASM["build/program.wasm"]
    WASM --> VALIDATE["irtool validate (wasmparser)"]
    VALIDATE --> RUNNER["runner test (Wasmtime 27.0.0)"]
    RUNNER --> PUB["public-tests.json"]
    RUNNER --> HID["hidden-tests.json"]
```

La arquitectura separa estrictamente la **zona probabilÃ­stica** (donde el agente traduce cambios de especificaciÃ³n o diagnÃ³sticos en mutaciones sobre el AST JSON de `ir/program.ir.json`) de la **zona determinista** (implementada en Rust mediante dos crates genÃ©ricos: `crates/irtool` con `wasm-encoder`, `wasmparser` y `wasmprinter`, y `crates/runner` con `wasmtime`).

## Infrastructure and Project Resources

| Recurso / Componente | Tipo | VersiÃ³n / Hash observado | Rol en la POC |
|---|---|---|---|
| `rustc` / `cargo` | Compilador host | `1.85.0 (4d91de4e4 2025-02-17)` | CompilaciÃ³n de la toolchain determinista (`irtool`, `runner`) |
| `wasmtime` | Runtime WASM embebido | `27.0.0` | EjecuciÃ³n aislada de funciones exportadas en `crates/runner` |
| `wasm-encoder` / `wasmparser` / `wasmprinter` | Bytecode Alliance crates | `0.221.3` | EmisiÃ³n binaria `.wasm`, validaciÃ³n independiente e inspecciÃ³n `.wat` |
| `target/debug/irtool.exe` | Binario congelado (`E1`) | `sha256: 05b0d4c4d8bd85b5fcd7cbcfafa92dccb9b57d0a240464d50a5c874452604d0e` | Verifier + Compiler directo `IR â†’ WASM` |
| `target/debug/runner.exe` | Binario congelado (`E1`) | `sha256: 808cf7f6a6b88aedb141a2d09d75d999d3f36b8824aa1837b19113fb9963782c` | Ejecutor determinista de suites YAML pÃºblicas y reservadas |

## Implementation and Relevant Technical Decisions

| Work Item | DecisiÃ³n tÃ©cnica relevante | JustificaciÃ³n |
|---|---|---|
| `WI-001` | FormalizaciÃ³n de `ir/ir.schema.json` (Draft 2020-12) y tabla de tipado con `eq` polimÃ³rfico `(i32, i32)` / `(bool, bool)` | Evitar ambigÃ¼edades entre la especificaciÃ³n de `E3` (`premium == false`) y el verificador semÃ¡ntico |
| `WI-002` | EmisiÃ³n directa a `.wasm` mediante `wasm-encoder` e integraciÃ³n de `wasmparser` (`irtool validate`) y `wasmprinter` (`irtool print`) | Cumplir la ruta principal sin pasar por cÃ³digo fuente ni depender de binarios externos fuera del `PATH` |
| `WI-003` | Sellado del freeze point en el tag `poc-e1-freeze` y registro en `runs/toolchain-freeze.sha256` | Garantizar auditabilidad criptogrÃ¡fica de que `E2`, `E3` y `E4` no alteran el backend |
| `WI-004`â€“`WI-006` | EvoluciÃ³n exclusiva sobre `ir/program.ir.json` con snapshots antes/despuÃ©s y `diff.patch` por intento | Demostrar localidad del cambio y trazabilidad por requisito |

## Experiments and Validation

| Ejercicio | DescripciÃ³n experimental | Resultado medido | Estado |
|---|---|---|---|
| **E1 â€” ConstrucciÃ³n y freeze** (`WI-003`) | ConstrucciÃ³n inicial de `calculate_shipping(subtotal_cents: i32) -> i32` desde `spec/task.yaml` v1 y congelamiento del toolchain | `0` diagnÃ³sticos en intento 1, `.wasm` de `76 B`, `6/6` pÃºblicas y `9/9` reservadas aprobadas | Validado |
| **E2 â€” ReparaciÃ³n controlada** (`WI-004`) | InyecciÃ³n del defecto `E_UNKNOWN_PARAM` (`param "subtotal"` en `$.functions[0].body.condition.left`) y reparaciÃ³n guiada por `diagnostics.injected.json` | Rechazo con `exit code = 1`; reparaciÃ³n en `1` intento (`<= 2`), `6/6` pÃºblicas, `9/9` reservadas y mismo SHA-256 `.wasm` que `E1` | Validado |
| **E3 â€” ExtensiÃ³n funcional** (`WI-005`) | PublicaciÃ³n de `spec/task.yaml` v2 aÃ±adiendo `calculate_discount(subtotal_cents: i32, premium: bool) -> i32` sin tocar `calculate_shipping` ni el backend | `0` diagnÃ³sticos en intento 1, `.wasm` de `134 B`, `12/12` pÃºblicas (`6` nuevas + `6` regresiÃ³n) y `18/18` reservadas aprobadas; hashes del backend intactos | Validado |
| **E4 â€” Mantenimiento** (`WI-006`) | PublicaciÃ³n de `spec/task.yaml` v3 cambiando `calculate_shipping` (`>= 4000 -> 0`, `< 4000 -> 600`) con `calculate_discount` intacta | Diff localizado de 3 lÃ­neas en IR, `0` diagnÃ³sticos, `12/12` pÃºblicas, `18/18` reservadas, `3/3` builds con idÃ©ntico SHA-256; hashes del backend intactos | Validado |

## Results and Measurements

| ID | MÃ©trica | Tipo | E1 (`WI-003`) | E2 (`WI-004`) | E3 (`WI-005`) | E4 (`WI-006`) | Gate |
|---|---|---|---|---|---|---|---|
| `M01` | Public test pass rate | Medido | `6/6` (`100 %`) | `6/6` (`100 %`) | `12/12` (`100 %`) | `12/12` (`100 %`) | `100 %` (Cumplido) |
| `M02` | Hidden test pass rate | Medido | `9/9` (`100 %`) | `9/9` (`100 %`) | `18/18` (`100 %`) | `18/18` (`100 %`) | `100 %` (Cumplido) |
| `M03` | IR verifier result (final) | Medido | `true` | `true` (tras rechazar `ir.injected.json`) | `true` | `true` | `true` (Cumplido) |
| `M04` | WASM validation result | Medido | `true` | `true` | `true` | `true` | `true` (Cumplido) |
| `M05` | Repair attempts | Medido | `0` | `1` | `0` | `0` | `<= 2` (Cumplido) |
| `M06` | Manual IR edits | Medido | `0` | `0` | `0` | `0` | `0` (Cumplido) |
| `M07` | Backend changes after freeze | Medido | N/A (freeze) | `0` | `0` | `0` | `0` (Cumplido) |
| `M08` | Reproducible builds (`E4`) | Medido | â€” | â€” | â€” | `3/3` idÃ©nticos | `3/3` (Cumplido) |
| `M13` | Verify time (`verify_ms`) | Medido | `12 ms` | `11 ms` | `11 ms` | `11 ms` | ObservaciÃ³n |
| `M14` | Compile time (`compile_ms`) | Medido | `15 ms` | `14 ms` | `14 ms` | `14 ms` | ObservaciÃ³n |
| `M15` | Test execution time (`test_ms`) | Medido | `16 ms` | `17 ms` | `28 ms` | `28 ms` | ObservaciÃ³n |
| `M16` | WASM size (`wasm_size_bytes`) | Medido | `76 B` | `76 B` | `134 B` | `134 B` | ObservaciÃ³n |
| `M17` | IR size (lÃ­neas / bytes) | Medido | `67 L` / `1336 B` | `67 L` / `1336 B` | `138 L` / `2652 B` | `138 L` / `2652 B` | ObservaciÃ³n |
| `M18` | IR change size (`diff.patch`) | Calculado | ConstrucciÃ³n inicial | `1` lÃ­nea modificada | `+71` lÃ­neas (`1` funciÃ³n aÃ±adida) | `3` lÃ­neas modificadas | ObservaciÃ³n |
| `M19` | Human clarifications | Medido | `0` | `0` | `0` | `0` | ObservaciÃ³n |
| `M20` | Diagnostic count | Medido | `0` | `1` (`E_UNKNOWN_PARAM` inyectado) â†’ `0` | `0` | `0` | ObservaciÃ³n |

## Cost and Efficiency Analysis

- **MÃ©tricas de tokens y costo monetario del proveedor (`M09`, `M10`, `M11`):** `No evidence available.` (El entorno de ejecuciÃ³n del agente no expone contadores desagregados de tokens ni facturaciÃ³n por llamada individual en `run.json`, registrÃ¡ndose como `null` conforme al esquema permitido en `poc.md`).
- **Eficiencia operativa del ciclo determinista (`M13`â€“`M16`):** El ciclo completo `verify â†’ compile â†’ validate â†’ test` se ejecutÃ³ en menos de `60 ms` totales por intento (medido: `11â€“12 ms` de verificaciÃ³n, `14â€“15 ms` de compilaciÃ³n directa a `.wasm` y `16â€“33 ms` de ejecuciÃ³n en Wasmtime), produciendo binarios extremadamente compactos (`76 bytes` para 1 funciÃ³n y `134 bytes` para 2 funciones) sin sobrecarga de compilaciÃ³n de un toolchain generalista durante la evoluciÃ³n de reglas (`E2`â€“`E4`).

## Observability and Operational Findings

- **Accionabilidad de diagnÃ³sticos estructurados:** El contrato de `diagnostics.json` (con `code`, `function`, `path` en notaciÃ³n JSONPath, `expected` y `found`) permitiÃ³ localizar y corregir en `1` intento el defecto sembrado en `E2` sin inspecciÃ³n heurÃ­stica.
- **Auditabilidad por corrida:** La estructura `runs/<exercise>/attempt-01/` con snapshots de entrada/salida, `diff.patch`, desensamblado `.wat` derivado y hashes SHA-256 permitiÃ³ verificar de forma independiente tanto la correcciÃ³n funcional como la inmutabilidad del backend congelado en `runs/toolchain-freeze.sha256`.

## Findings, Learnings, Limitations and Risks

| CategorÃ­a | DescripciÃ³n fundamentada en la evidencia |
|---|---|
| **Hallazgo principal** | Una Semantic IR pequeÃ±a y fuertemente tipada permite al agente construir, reparar, extender y mantener reglas puras con `0` errores sintÃ¡cticos y diffs mÃ­nimos (`1` a `3` lÃ­neas en reparaciÃ³n y mantenimiento). |
| **Aprendizaje de ingenierÃ­a** | La inversiÃ³n de complejidad ocurre una sola vez al construir el verifier y el lowering `IR â†’ WASM` (`WI-002`); una vez congelado el toolchain (`E1`), la evoluciÃ³n funcional (`E3`, `E4`) es inmediata y determinista. |
| **LimitaciÃ³n actual** | `IR v0.1` estÃ¡ deliberadamente restringida a funciones escalares puras (`i32`, `bool`) sin estructuras compuestas, llamadas entre funciones, memoria lineal ni I/O. |
| **Riesgo para fases siguientes** | Al escalar a tipos compuestos, mÃºltiples mÃ³dulos o capacidades externas (`POC 002`), un diseÃ±o excesivamente expresivo de la IR podrÃ­a reintroducir la complejidad accidental de un lenguaje fuente tradicional si no se mantiene acotado por dominio. |

## Conclusion

- **Status:** **`validated`**
- **Decision:** **`GO`**

Los `16/16` Success Criteria (`SC-01`â€“`SC-16`) fueron satisfechos sin excepciones ni desviaciones respecto al protocolo experimental de `knowledge/delivery/poc.md`. Queda validada la hipÃ³tesis `H1` para el alcance de funciones puras de reglas comerciales compiladas directamente desde Semantic IR v0.1 a WebAssembly.

## Recommendation and Next Steps

1. **Benchmark comparativo pareado `Agent-Native IR â†’ WASM` vs `Rust â†’ wasm32-unknown-unknown` (`KAN-POC-WI-008`):** Ejecutar la misma secuencia `E1`â€“`E4` con las mismas especificaciones y suites pÃºblicas/reservadas pidiendo al agente implementar en Rust, comparando intentos de reparaciÃ³n, tiempo de compilaciÃ³n, tamaÃ±o binario y localidad del diff.
2. **DiseÃ±o de `KADDO-AGENT-NATIVE-POC-002` (Complejidad incremental sin I/O):** Extender el contrato de la Semantic IR a `v0.2` para evaluar llamadas entre funciones internas, tipos compuestos (records/structs de entrada) y errores de dominio explÃ­citamente tipados en lugar de la convenciÃ³n `-1`.
3. **ExploraciÃ³n posterior de Capability ABI:** SÃ³lo tras validar composiciÃ³n y tipos estructurados en `POC 002`, investigar un ABI de capacidades declarativas para efectos externos controlados.

## Traceability and Sources

| Fuente / Artefacto | Tipo | Ruta relativa en el repositorio |
|---|---|---|
| DefiniciÃ³n y conclusiÃ³n canÃ³nica de la POC | POC Artifact | [knowledge/delivery/poc.md](knowledge/delivery/poc.md) |
| Contrato experimental e IR v0.1 | Work Item (`completed`) | [knowledge/delivery/work-items/completed/WI-001-contrato-experimental-e-ir-v0-1.md](knowledge/delivery/work-items/completed/WI-001-contrato-experimental-e-ir-v0-1.md) |
| Verifier + compiler directo a WASM + runner | Work Item (`completed`) | [knowledge/delivery/work-items/completed/WI-002-verifier-compiler-directo-a-wasm-y-runner.md](knowledge/delivery/work-items/completed/WI-002-verifier-compiler-directo-a-wasm-y-runner.md) |
| E1 construcciÃ³n y freeze | Work Item (`completed`) | [knowledge/delivery/work-items/completed/WI-003-e1-construccion-y-freeze.md](knowledge/delivery/work-items/completed/WI-003-e1-construccion-y-freeze.md) |
| E2 reparaciÃ³n controlada | Work Item (`completed`) | [knowledge/delivery/work-items/completed/WI-004-e2-reparacion-controlada.md](knowledge/delivery/work-items/completed/WI-004-e2-reparacion-controlada.md) |
| E3 extensiÃ³n funcional | Work Item (`completed`) | [knowledge/delivery/work-items/completed/WI-005-e3-extension-funcional.md](knowledge/delivery/work-items/completed/WI-005-e3-extension-funcional.md) |
| E4 mantenimiento | Work Item (`completed`) | [knowledge/delivery/work-items/completed/WI-006-e4-mantenimiento.md](knowledge/delivery/work-items/completed/WI-006-e4-mantenimiento.md) |
| AnÃ¡lisis y conclusiÃ³n | Work Item (`completed`) | [knowledge/delivery/work-items/completed/WI-007-analisis-y-conclusion.md](knowledge/delivery/work-items/completed/WI-007-analisis-y-conclusion.md) |
| Evidencia reproducible E1 | Run Evidence | [runs/e1-build/attempt-01/run.json](runs/e1-build/attempt-01/run.json) |
| Evidencia reproducible E2 | Run Evidence | [runs/e2-repair/attempt-01/run.json](runs/e2-repair/attempt-01/run.json) |
| Evidencia reproducible E3 | Run Evidence | [runs/e3-extend/attempt-01/run.json](runs/e3-extend/attempt-01/run.json) |
| Evidencia reproducible E4 | Run Evidence | [runs/e4-maintain/attempt-01/run.json](runs/e4-maintain/attempt-01/run.json) |
| Freeze de toolchain | SHA-256 Manifest | [runs/toolchain-freeze.sha256](runs/toolchain-freeze.sha256) |
