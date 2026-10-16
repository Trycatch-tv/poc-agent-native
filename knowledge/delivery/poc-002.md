---
type: poc
status: completed
conclusion: validated
generated_by: kaddo-init
template_version: 1
project_state: ai-assisted
refined_by: work-item-agent
initiative: KADDO-AGENT-NATIVE-POC-002
---

# Kaddo Agent-Native POC 002 — Composición funcional, tipos compuestos (records) y errores de dominio tipados en Semantic IR v0.2

> **Artefacto canónico:** `knowledge/delivery/poc.md`
>
> **Referencia histórica previa:** [POC 001 Validada (GO)](knowledge/delivery/poc-001.md) | [Reporte Final POC 001](knowledge/delivery/poc-report.md) | [Benchmark Report](BENCHMARK_REPORT.md)
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
2. **Tipos de datos compuestos:** agrupar datos de negocio en registros tipados (`records` / `structs`, e.g. líneas de orden comercial `LineItem { unit_price: i32, quantity: i32 }`).
3. **Manejo formal de errores:** sustituir convenciones primitivas ad-hoc (como retornar `-1` en montos inválidos) por errores de dominio tipados y discriminados (`status_code` o `Result<OrderSummary, OrderError>`).

El problema a resolver en esta POC 002 es determinar si la Semantic IR puede escalar para soportar composición funcional, layouts predecibles en memoria lineal de WebAssembly y variantes de error tipadas, **sin perder la estabilidad del backend, la precisión del type-checking por AST y la compacidad de los diagnósticos estructurados**.

## Hypothesis

### Hipótesis principal

> **H2.** Dada una especificación funcional con composición modular, estructuras de datos compuestas y variantes de error explícitas, un agente de IA puede construir, reparar, extender y mantener la Semantic IR v0.2 de un programa mediante ciclos sucesivos (`E1`–`E4`), mientras una toolchain determinista verifica tipos, firmas y layouts de memoria, emitiendo WebAssembly ejecutable que satisface el 100 % de las pruebas de aceptación y regresión, sin requerir modificaciones al backend tras el hito inicial `E1`.

### Subhipótesis

| ID | Hipótesis | Evidencia favorable | Evidencia desfavorable |
|---|---|---|---|
| **H2-A** | El agente puede construir programas que involucren subfunciones internas (`call`) y estructuras de datos (`records`) a partir de la especificación. | `E1-002` pasa verificación y el 100 % de pruebas públicas y reservadas en <= 3 intentos. | Requiere intervención manual, no resuelve el layout de registros o excede el límite de intentos. |
| **H2-B** | Diagnósticos semánticos con JSONPath son suficientes para reparar defectos en campos de records o firmas de funciones internas. | `E2-002` es reparado por el agente en máximo 2 intentos sin ambigüedad sintáctica. | El agente no logra identificar el campo o tipo desalineado con el JSONPath provisto. |
| **H2-C** | La IR soporta evolución hacia variantes tipadas de error (`status_code` / discriminantes) con el backend congelado. | `E3-002` introduce manejo de error tipado sin tocar los binarios de la toolchain sellados en `E1`. | Cada nueva estructura o variante de error requiere parches en el compilador. |
| **H2-D** | La modularización interna permite mantenimiento localizado sin regresiones. | `E4-002` modifica una subfunción interna preservando el 100 % de las pruebas de integración existentes. | La modificación de una regla interna propaga fallos imprevistos en funciones dependientes. |
| **H2-E** | El pipeline con memoria lineal determinista es reproducible bit a bit. | 3 compilaciones sucesivas de la misma IR v0.2 producen exactamente el mismo SHA-256 en `.wasm`. | El orden de campos o asignación de memoria lineal introduce no-determinismo. |

## Constraints

- Los montos monetarios continúan en centavos enteros (`i32`).
- Las estructuras de datos se sitúan en memoria lineal con alineación determinista de 4 bytes (`i32`).
- El compilador emite directamente el binario `.wasm`; el agente nunca escribe ni edita WAT ni WASM.
- Al culminar exitosamente `E1-002`, la toolchain se congela de forma estricta (`poc-002-e1-freeze`), y sus hashes se custodian en `runs/toolchain-freeze-poc002.sha256`.
- El agente no introduce mutaciones no controladas ni I/O no tipada.

## Non-goals

1. **No I/O ni Capabilities externas:** No se implementará comunicación de red, filesystem, base de datos ni llamadas asíncronas. La POC evalúa pura computación y memoria lineal en WebAssembly Core.
2. **No recolección de basura (GC) en runtime:** No se integrará un Garbage Collector en WASM; los layouts de records se resuelven mediante un allocator determinista lineal bump en memoria.
3. **No concurrencia:** Sin threads ni primitivas atómicas.

## Scenario

### Alcance técnico de la Semantic IR v0.2

| Categoría | Elementos soportados en v0.2 | Novedad frente a v0.1 |
|---|---|---|
| **Tipos escalares** | `i32`, `bool` | Heredado de v0.1 |
| **Tipos compuestos** | `record` (tupla nombrada con campos tipados) | **Nuevo en v0.2** |
| **Errores de dominio** | Códigos de error tipados y variantes de estado en records | **Nuevo en v0.2** |
| **Llamadas** | `call` a funciones internas declaradas con chequeo estricto de firma | **Nuevo en v0.2** |
| **Operadores de record** | `record.construct`, `record.get` (acceso a campo tipado en memoria) | **Nuevo en v0.2** |
| **Control & Aritmética** | `if`, `eq`, `lt_s`, `gte_s`, `add`, `sub`, `mul`, `div_s`, `and`, `or`, `not` | Heredado de v0.1 |
| **Trazabilidad** | IDs de requisitos por función y por variante (`ORDER-*`, `TAX-*`, `LINE-*`) | Reforzado |
| **Target de emisión** | WebAssembly Core Module (con memoria lineal 1 página e i32 ptr) | Estructuras en memoria |

## Success Criteria

- [x] **SC-01:** Construcción inicial `E1-002` completada por el agente en <= 3 intentos desde la especificación v1-002. *(Aprobada en 1 intento)*.
- [x] **SC-02:** Corrección funcional `E1-002`: 100 % de pruebas públicas y reservadas aprobadas. *(6/6 públicas, 9/9 reservadas)*.
- [x] **SC-03:** Congelamiento de toolchain sellado: Hash SHA-256 de `irtool` y `runner` registrado en `runs/toolchain-freeze-poc002.sha256`. *(Tag git `poc-002-e1-freeze`)*.
- [x] **SC-04:** Detección determinista de defecto sembrado en `E2-002`: rechazo con exit code != 0 y diagnóstico JSONPath explícito. *(Exit code 1, `E_UNKNOWN_FIELD` en `$.functions[2].body.else.fields.subtotal`)*.
- [x] **SC-05:** Reparación controlada `E2-002`: reparado por el agente en <= 2 intentos. *(Reparado en 1 intento, 100 % pass rate recuperado)*.
- [x] **SC-06:** Extensión funcional `E3-002` con errores tipados: completada en <= 3 intentos desde spec v2-002. *(Completada en 1 intento)*.
- [x] **SC-07:** Inmutabilidad del backend en `E3-002`: 0 cambios en el backend congelado. *(Hashes de `irtool.exe` y `runner.exe` idénticos)*.
- [x] **SC-08:** Preservación de regresión en `E3-002`: 100 % de pruebas de suites previas y ampliadas aprobadas. *(12/12 públicas, 18/18 reservadas)*.
- [x] **SC-09:** Inmutabilidad del backend en `E4-002`: 0 cambios en el backend congelado. *(Hashes idénticos)*.
- [x] **SC-10:** Aprobación de mantenimiento modular en `E4-002`: 100 % de pruebas aprobadas con diff localizado. *(12/12 públicas, 18/18 reservadas)*.
- [x] **SC-11:** Reproducibilidad binaria exacta: 3/3 builds deterministas producen el mismo SHA-256 de `.wasm`. *(`0a2265bca6b1431f2ea15bb3da9831f5e1f8082230c36fd1659c613f9af88c85`)*.
- [x] **SC-12:** Integridad de roles (agente): 0 líneas de WAT o WASM editadas por el agente. *(100 % emitido por compilador)*.
- [x] **SC-13:** Integridad del estado de software: Semantic IR v0.2 es el único artefacto modificado por el agente. *(Modificaciones exclusivas a `ir/program-002.ir.json`)*.
- [x] **SC-14:** Compacidad diagnóstica: salida de diagnóstico JSON estructurada con JSONPath accionable. *(Diagnósticos en E2-002 con ruta exacta de fallo)*.
- [x] **SC-15:** Latencia del bucle determinista: `verify` + `compile` < 60 ms. *(`23 ms` a `28 ms` medidos en toolchain)*.
- [x] **SC-16:** Validación estricta de WebAssembly: `.wasm` pasa `wasmparser` sin violaciones de tipo ni memoria en todos los ejercicios. *(100 % de módulos validados)*.

## Evidence and Execution Matrix

| Ejercicio | Intento | Diagnósticos | Pruebas Públicas | Pruebas Reservadas | WASM Size | WASM SHA-256 | Toolchain Hash Match | Estado |
|---|---|---|---|---|---|---|---|---|
| **`E1-002` (Construcción)** | 1 de 3 | 0 | 6/6 (100 %) | 9/9 (100 %) | 308 B | `f42bcb6e03c616b2891a9c4d1e060c4e141d502b01f44a326bc749d65a0d771a` | Base freeze (`poc-002-e1-freeze`) | **PASSED** |
| **`E2-002` (Inyección)** | - | 2 (`E_UNKNOWN_FIELD`, `E_MISSING_FIELD`) | Rechazo (exit 1) | - | - | - | 100 % match | **REJECTED (OK)** |
| **`E2-002` (Reparación)** | 1 de 2 | 0 | 6/6 (100 %) | 9/9 (100 %) | 308 B | `f42bcb6e03c616b2891a9c4d1e060c4e141d502b01f44a326bc749d65a0d771a` | 100 % match | **PASSED** |
| **`E3-002` (Extensión Errores)** | 1 de 3 | 0 | 12/12 (100 %) | 18/18 (100 %) | 459 B | `8079c054444d0541f555780e104a5259d2519a5556103104e1f14d0b35232f0e` | 100 % match | **PASSED** |
| **`E4-002` (Mantenimiento)** | 1 de 3 | 0 | 12/12 (100 %) | 18/18 (100 %) | 459 B | `0a2265bca6b1431f2ea15bb3da9831f5e1f8082230c36fd1659c613f9af88c85` | 100 % match | **PASSED** |

### Toolchain Cryptographic Freeze

- `target/debug/irtool.exe`: `959ca4f87127d36799c1bddad2af2cf260006711d03c6a8fd4f58fdcaf581f2d`
- `target/debug/runner.exe`: `feda31350f81dcd8beca79f31a15b1e6932cd8fb9ab9a7266b775e8845795a5b`
- Registrado y sellado en: `runs/toolchain-freeze-poc002.sha256` y tag `poc-002-e1-freeze`.

## Evaluation and Answers to Core Questions

1. **¿Pudo el agente operar con composición funcional y tipos compuestos en memoria lineal?**
   Sí (`SC-01`, `SC-02`, `SC-12`, `SC-13`). El agente modeló `LineItem` y `OrderCalculation`, implementó llamadas internas con `call` (`calculate_line_item`, `calculate_tax`) y gestionó la instanciación de estructuras vía un bump allocator determinista en memoria lineal WASM sin intervención manual.
2. **¿Fueron los diagnósticos semánticos con JSONPath efectivos ante defectos en registros?**
   Sí (`SC-04`, `SC-05`). El defecto inyectado en `E2-002` (`subtotal` en lugar de `subtotal_cents`) fue detectado en 12 ms con JSONPath exacto `$.functions[2].body.else.fields.subtotal` y reparado en 1 solo intento recuperando la paridad bit a bit del binario WASM.
3. **¿Se mantuvo el backend congelado durante la extensión funcional y el mantenimiento?**
   Sí (`SC-07`, `SC-09`). Ni `E3-002` (códigos de error de dominio) ni `E4-002` (actualización de alícuota fiscal del 10 % al 15 %) requirieron modificar una sola línea de código en `crates/irtool` o `crates/runner`. Los hashes SHA-256 de los ejecutables se mantuvieron idénticos.
4. **¿Permitió la composición funcional un mantenimiento verdaderamente localizado?**
   Sí (`SC-10`). En `E4-002`, el cambio de política fiscal se resolvió alterando únicamente 2 líneas en la subfunción interna `calculate_tax`, sin tocar la función orquestadora `calculate_order` ni las estructuras de datos, logrando 100 % de aprobación en suites públicas y reservadas.
5. **¿El pipeline continuó siendo determinista y reproducible?**
   Sí (`SC-11`). 3 compilaciones sucesivas e independientes en `E4-002` produjeron exactamente el mismo hash SHA-256 (`0a2265bca6b1431f2ea15bb3da9831f5e1f8082230c36fd1659c613f9af88c85`).
6. **¿Qué tamaño y latencia exhibe el runtime de Semantic IR v0.2?**
   Los binarios WASM resultantes oscilaron entre `308 bytes` (v1) y `459 bytes` (v2/v3). La latencia de verificación y compilación se mantuvo por debajo de los `28 ms`, validando que la adición de memoria lineal y llamadas no degradó el rendimiento ultra-rápido frente a compiladores tradicionales.

## Conclusion

Status: validated

Decision: GO

Evidence summary:
- E1-002: Procesamiento de órdenes comerciales con composición de subfunciones y registros en memoria lineal completado en 1 intento (308 bytes WASM, 6/6 públicas, 9/9 reservadas). Freeze sellado en tag poc-002-e1-freeze.
- E2-002: Defecto inyectado E_UNKNOWN_FIELD en campo de record rechazado determinísticamente con JSONPath exacto y reparado en 1 intento, recuperando el hash SHA-256 idéntico.
- E3-002: Extensión funcional con códigos de error de dominio (OrderCalculation.status_code 0..3) aprobada en 1 intento sobre backend congelado (459 bytes WASM, 12/12 públicas, 18/18 reservadas).
- E4-002: Mantenimiento modularizado en subfunción interna calculate_tax (alícuota 15 %) realizado en un diff localizado de 2 líneas sin tocar calculate_order ni el backend congelado (12/12 públicas, 18/18 reservadas, 3/3 builds deterministas idénticos).
- Success criteria passed: 16/16 (SC-01 a SC-16).
- Success criteria failed: 0/16.
- Inmutabilidad de Toolchain: 100 % confirmada en runs/toolchain-freeze-poc002.sha256.

Recommendation:
Decisión GO (validated). Habiendo comprobado que Semantic IR v0.2 soporta composición funcional, estructuras en memoria lineal y discriminación de errores con backend congelado y latencias < 30 ms, se recomienda formalmente avanzar a la Fase 3: POC 003 — Capability ABI & WebAssembly Component Model (interacción controlada con el mundo exterior: I/O, almacenamiento y mensajería declarativa sin romper el determinismo de la lógica de negocio).
