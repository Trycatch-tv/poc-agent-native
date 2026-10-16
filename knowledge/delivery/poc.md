---
type: poc
status: active
generated_by: kaddo-init
template_version: 1
project_state: ai-assisted
refined_by: work-item-agent
initiative: KADDO-AGENT-NATIVE-POC-002
---

# Kaddo Agent-Native POC 002 — Composición funcional, tipos compuestos (records) y errores de dominio tipados en Semantic IR v0.2

> **Artefacto canónico:** `knowledge/delivery/poc.md`
>
> **Referencia histórica previa:** [POC 001 Validada (GO)](file:///c:/Users/julia/Documents/repos/poc-agent-native/knowledge/delivery/poc-001.md) | [Reporte Final POC 001](file:///c:/Users/julia/Documents/repos/poc-agent-native/knowledge/delivery/poc-report.md) | [Benchmark Report](file:///c:/Users/julia/Documents/repos/poc-agent-native/BENCHMARK_REPORT.md)
>
> **Pregunta de investigación:** ¿puede un agente construir, corregir, mantener y extender programas con composición funcional interna (`call`), tipos compuestos en memoria lineal (`records`/`structs`) y errores tipados discriminados (`Result<T, E>`), mientras herramientas deterministas verifican esa IR v0.2 y generan WebAssembly Core sin degradar la compacidad diagnóstica (`< 500 B`), el feedback ultra-rápido (`< 60 ms`) ni la reproducibilidad bit a bit?
>
> **Naturaleza:** POC de investigación técnica, empírica, falsable y reproducible.
>
> **Modalidad:** Kaddo POC Mode (Fase 2).

## Problem

En la POC 001 (`KADDO-AGENT-NATIVE-POC-001`) se demostró que un agente de IA puede construir y mantener reglas puras sobre tipos escalares (`i32`, `bool`) mediante una Semantic IR y un micro-compilador determinista, obteniendo ventajas de `16x` a `49x` en latencia de toolchain y diagnósticos `23.3x` más compactos que el baseline de Rust.

Sin embargo, el software de negocio real requiere modelar entidades complejas y modularizar la lógica:
1. **Composición de funciones:** dividir problemas en subfunciones auxiliares internas (`call`) sin exponerlas necesariamente como exports públicos.
2. **Tipos de datos compuestos:** agrupar datos de negocio en registros tipados (`records` / `structs`, e.g. líneas de orden comercial `Item { unit_price: i32, quantity: i32 }`).
3. **Manejo formal de errores:** sustituir convenciones primitivas ad-hoc (como retornar `-1` en montos inválidos) por errores de dominio tipados y discriminados (`Result<OrderSummary, OrderError>`).

El problema a resolver en esta POC 002 es determinar si la Semantic IR puede escalar para soportar composición funcional, layouts predecibles en memoria lineal de WebAssembly y variantes de error tipadas, **sin perder la estabilidad del backend, la precisión del type-checking por AST y la compacidad de los diagnósticos estructurados**.

## Hypothesis

### Hipótesis principal

> **H2.** Dada una especificación funcional con composición modular, estructuras de datos compuestas y variantes de error explícitas, un agente de IA puede construir, reparar, extender y mantener la Semantic IR v0.2 de un programa mediante ciclos sucesivos (`E1`–`E4`), mientras una toolchain determinista verifica tipos, firmas y layouts de memoria, emitiendo WebAssembly ejecutable que satisface el 100 % de las pruebas de aceptación y regresión, sin requerir modificaciones al backend tras el hito inicial `E1`.

### Subhipótesis

| ID | Hipótesis | Evidencia favorable | Evidencia desfavorable |
|---|---|---|---|
| **H2-A** | El agente puede construir programas que involucren subfunciones internas (`call`) y estructuras de datos (`records`) a partir de la especificación. | `E1-002` pasa verificación y el 100 % de pruebas públicas y reservadas en <= 3 intentos. | Requiere intervención manual, no resuelve el layout de registros o excede el límite de intentos. |
| **H2-B** | Diagnósticos semánticos con JSONPath son suficientes para reparar defectos en campos de records o firmas de funciones internas. | `E2-002` es reparado por el agente en máximo 2 intentos sin ambigüedad sintáctica. | El agente no logra identificar el campo o tipo desalineado con el JSONPath provisto. |
| **H2-C** | La IR soporta evolución hacia variantes tipadas de error (`Result<T, E>`) con el backend congelado. | `E3-002` introduce manejo de error tipado sin tocar los binarios de la toolchain sellados en `E1`. | Cada nueva estructura o variante de error requiere parches en el compilador. |
| **H2-D** | La modularización interna permite mantenimiento localizado sin regresiones. | `E4-002` modifica una subfunción interna preservando el 100 % de las pruebas de integración existentes. | La modificación de una regla interna propaga fallos imprevistos en funciones dependientes. |
| **H2-E** | El pipeline con memoria lineal determinista es reproducible bit a bit. | 3 compilaciones sucesivas de la misma IR v0.2 producen exactamente el mismo SHA-256 en `.wasm`. | El orden de campos o asignación de memoria lineal introduce no-determinismo. |

### Non-Goals explícitos de POC 002

1. **No I/O ni Capabilities externas:** No se implementará comunicación de red, filesystem, base de datos ni llamadas asíncronas. La POC evalúa pura computación y memoria lineal en WebAssembly Core.
2. **No recolección de basura (GC) en runtime:** No se integrará un Garbage Collector en WASM; los layouts de records se resuelven de forma estática o mediante un allocator lineal linealizado determinista.
3. **No concurrencia:** Sin threads ni primitivas atómicas.

## Scenario

### Alcance técnico de la Semantic IR v0.2

| Categoría | Elementos soportados en v0.2 | Novedad frente a v0.1 |
|---|---|---|
| **Tipos escalares** | `i32`, `bool` | Heredado de v0.1 |
| **Tipos compuestos** | `record` (tupla nombrada con campos tipados) | **Nuevo en v0.2** |
| **Tipos resultado** | `Result<T, E>` / variantes discriminadas de error de dominio | **Nuevo en v0.2** |
| **Llamadas** | `call` a funciones internas declaradas con chequeo estricto de firma | **Nuevo en v0.2** |
| **Operadores de record** | `record.construct`, `record.get` (acceso a campo tipado) | **Nuevo en v0.2** |
| **Operadores de resultado** | `result.ok`, `result.err`, `result.is_err`, `result.unwrap` | **Nuevo en v0.2** |
| **Control & Aritmética** | `if`, `eq`, `lt_s`, `gte_s`, `add`, `sub`, `mul`, `div_s`, `and`, `or`, `not` | Heredado de v0.1 |
| **Trazabilidad** | IDs de requisitos por función y por variante | Reforzado |
| **Target de emisión** | WebAssembly Core Module (con sección `memory` determinista y exports) | Estructuras en memoria |

### Convenciones de la POC 002

- Los montos monetarios continúan en centavos enteros (`i32`).
- Las estructuras de datos se sitúan en memoria lineal con alineación determinista de 4 bytes (`i32`).
- El compilador emite directamente el binario `.wasm`; el agente nunca escribe ni edita WAT ni WASM.
- Al culminar exitosamente `E1-002`, la toolchain se congela de forma estricta (`poc-002-e1-freeze`), y su hash se custodia en `runs/toolchain-freeze-poc002.sha256`.

## Success Criteria

| ID | Criterio de Éxito | Métrica objetivo |
|---|---|---|
| **SC-01** | Construcción inicial `E1-002` completada por el agente | <= 3 intentos desde la especificación v1-002 |
| **SC-02** | Corrección funcional `E1-002` | 100 % de pruebas públicas y reservadas aprobadas |
| **SC-03** | Congelamiento de toolchain sellado | Hash SHA-256 de `irtool` y `runner` registrado |
| **SC-04** | Detección determinista de defecto sembrado en `E2-002` | Rechazo con exit code != 0 y diagnóstico JSONPath explícito |
| **SC-05** | Reparación controlada `E2-002` | Reparado por el agente en <= 2 intentos |
| **SC-06** | Extensión funcional `E3-002` con errores tipados | Completada en <= 3 intentos desde spec v2-002 |
| **SC-07** | Inmutabilidad del backend en `E3-002` | 0 cambios en el backend congelado (mismo SHA-256) |
| **SC-08** | Preservación de regresión en `E3-002` | 100 % de pruebas de suites previas aprobadas |
| **SC-09** | Inmutabilidad del backend en `E4-002` | 0 cambios en el backend congelado |
| **SC-10** | Aprobación de mantenimiento modular en `E4-002` | 100 % de pruebas aprobadas |
| **SC-11** | Reproducibilidad binaria exacta | 3/3 builds deterministas producen el mismo SHA-256 de `.wasm` |
| **SC-12** | Integridad de roles (agente) | 0 líneas de WAT o WASM editadas por el agente |
| **SC-13** | Integridad del estado de software | Semantic IR v0.2 es el único artefacto modificado por el agente |
| **SC-14** | Compacidad diagnóstica | Salida de diagnóstico JSON estructurada < 500 bytes |
| **SC-15** | Latencia del bucle determinista | `verify` + `compile` < 60 ms |
| **SC-16** | Validación estricta de WebAssembly | `.wasm` pasa `wasmparser` sin violaciones de tipo ni memoria |

## Work Items Plan (Fase 2)

| ID | Tipo | Título | Entregables principales |
|---|---|---|---|
| **`WI-009`** | spike | Contrato experimental e IR v0.2 | `ir/ir.schema.v2.json`, catálogo de tipos compuestos (`record`), variantes `Result<T, E>`, especificación funcional `spec/task-002.yaml` (v1) y suite de pruebas públicas `tests/public-002.yaml`. |
| **`WI-010`** | spike | Verifier v0.2 + Compiler IR v0.2 directo a WASM + Runner | Soporte en `crates/irtool` para type-checking de llamadas internas (`call`), layout determinista de records en memoria lineal, diagnósticos estructurados (`E_UNKNOWN_FIELD`, `E_SIGNATURE_MISMATCH`), y soporte en `runner` para pasar/leer estructuras en Wasmtime. |
| **`WI-011`** | spike | E1-002 Construcción y Freeze | Procesamiento de orden comercial con records (`OrderItem { unit_price, quantity }`, `OrderCalculation`) y llamadas internas, aprobación de pruebas públicas (`6/6`) y reservadas, sellado de freeze point `poc-002-e1-freeze`. |
| **`WI-012`** | spike | E2-002 Reparación controlada | Inyección controlada de defecto en campo de record o llamada interna, rechazo determinista con diagnóstico JSONPath (`< 500 B`), reparación en 1 intento y validación de regresión. |
| **`WI-013`** | spike | E3-002 Extensión funcional con errores tipados | Incorporación de variantes tipadas `OrderError` (`InvalidQuantity`, `ExceedsMaxAmount`) en `Result<OrderCalculation, OrderError>`, pruebas ampliadas, comprobación de backend congelado. |
| **`WI-014`** | spike | E4-002 Mantenimiento modular | Modificación localizada de regla de negocio en subfunción interna (diff conciso), verificación de no-regresión y reproducibilidad `3/3`. |
| **`WI-015`** | chore | Análisis, matriz SC y conclusión POC 002 | Evaluación de `SC-01` a `SC-16`, reporte consolidado `poc-report-v003.md`, decisión Kaddo (`GO / PIVOT / STOP`). |
| **`WI-016`** | spike opcional | Benchmark pareado POC-002 vs Rust | Comparación pareada de registros en memoria y llamadas internas: Semantic IR v0.2 vs `Rust` (`#![no_std]` con structs). |

## Conclusion Rule

- **GO (`validated`):** Se cumplen `SC-01` a `SC-16`. Justifica avanzar a la Fase 3 (**POC 003**: Capability ABI / Component Model para I/O externa).
- **PIVOT (`inconclusive`):** La composición funciona pero la gestión de memoria lineal o errores tipados resulta demasiado compleja para el agente o el verificador sin un runtime más pesado.
- **STOP (`rejected`):** El agente no logra razonar sobre campos o llamadas en la IR v0.2 o el backend no puede mantenerse congelado.
