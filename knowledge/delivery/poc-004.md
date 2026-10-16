---
type: poc
status: active
generated_by: kaddo-init
template_version: 1
project_state: ai-assisted
refined_by: work-item-agent
initiative: KADDO-AGENT-NATIVE-POC-004
---

# Kaddo Agent-Native POC 004 — WebAssembly Component Model, Interfaces WIT Estandarizadas y Canonical ABI en Semantic IR v0.4

> **Artefacto canónico:** `knowledge/delivery/poc.md`
>
> **Referencias históricas previas:**
> - [POC 001 Validada (GO)](knowledge/delivery/poc-001.md) | [Reporte Final POC 001](knowledge/delivery/poc-report-v001.md) | [Benchmark 001](BENCHMARK_REPORT.md)
> - [POC 002 Validada (GO)](knowledge/delivery/poc-002.md) | [Reporte Final POC 002](knowledge/delivery/poc-report-v003.md) | [Benchmark 002](BENCHMARK_REPORT_002.md)
> - [POC 003 Validada (GO)](knowledge/delivery/poc-003.md) | [Reporte Final POC 003](knowledge/delivery/poc-report-v004.md) | [Benchmark 003](BENCHMARK_REPORT_003.md)
>
> **Pregunta de investigación:** ¿puede un agente de IA diseñar, evolucionar y componer lógica comercial en Semantic IR v0.4 a partir de especificaciones de interfaz estandarizadas en **WIT** (WebAssembly Interface Types), mientras una micro-toolchain determinista verifica la compatibilidad de firmas de interfaz, realiza el empaquetado como **WebAssembly Component** (conforme a Canonical ABI) y ejecuta pruebas en el runtime de componentes de Wasmtime, manteniendo el backend estrictamente congelado tras `E1-004` sin sacrificar latencias sub-60ms ni compacidad diagnóstica?
>
> **Naturaleza:** POC de investigación técnica, empírica, falsable y reproducible.
>
> **Modalidad:** Kaddo POC Mode (Fase 4).

## Problem

En las fases anteriores (`POC 001` a `POC 003`), se consolidó un micro-toolchain determinista capaz de compilar lógica pura, gestionar memoria lineal mediante bump allocation y enlazar imports declarativos con auditoría de efectos secundarios en WebAssembly Core.

No obstante, WebAssembly Core adolece de una limitación de diseño fundamental para ecosistemas agent-native y microservicios modernos:
1. **Límite de tipos primitivos planos:** Core WASM sólo transporta escalares (`i32`, `i64`, `f32`, `f64`). Cualquier estructura compuesta o variante de error exige pactar offsets ad-hoc en memoria lineal compartida.
2. **Falta de encapsulamiento de memoria:** Si dos módulos colaboran en Core WASM, deben compartir el mismo espacio de direccionamiento lineal o implementar copias manuales complejas y propensas a corrupción.
3. **Ausencia de un estándar universal de interfaces:** Sin un lenguaje de definición de interfaces agnóstico (IDL), el agente no puede integrarse limpiamente con componentes escritos en otros lenguajes (Rust, Go, Python) ni asegurar interoperabilidad estricta.

El **WebAssembly Component Model** y su lenguaje de interfaz **WIT** resuelven esta brecha:
- Las interfaces de imports y exports se declaran con tipos de alto nivel (`record`, `variant`, `enum`, `list`, `result<T, E>`).
- Los componentes poseen memoria lineal completamente aislada (encapsulamiento seguro).
- La comunicación se normaliza mediante el **Canonical ABI** (`canon lift` y `canon lower`).

El problema a resolver en esta **POC 004** es determinar si la Semantic IR puede elevar su semántica a **v0.4** para ligarse directamente a contratos **WIT**, verificando compatibilidad de tipos de interfaz y emitiendo un **WebAssembly Component** funcional que se ejecute en `wasmtime::component`, garantizando inmutabilidad post-freeze del compilador y preservando las ventajas de latencia ultra-baja y diagnósticos estructurados.

## Hypothesis

### Hipótesis principal

> **H4.** Dada una especificación de contrato comercial en interfaces WIT estandarizadas, un agente de IA puede generar, reparar, extender y mantener programas en Semantic IR v0.4 que implementen e importen dichas interfaces, mientras el micro-toolchain determinista verifica compatibilidad estática con el contrato WIT, empaqueta el artefacto como WebAssembly Component estándar y valida suites de integración en Wasmtime Component Runtime con backend congelado tras `E1-004`, manteniendo tiempos de ciclo < 60 ms y diagnósticos estructurados < 500 bytes.

### Subhipótesis

| ID | Hipótesis | Evidencia favorable | Evidencia desfavorable |
|---|---|---|---|
| **H4-A** | El agente puede implementar contratos de interfaz WIT (`export interface`) y consumir dependencias (`import interface`) en IR v0.4. | `E1-004` supera verificación estática y 100 % de pruebas en Wasmtime Component Runtime en <= 3 intentos. | El agente no logra mapear tipos WIT o excede el límite de intentos. |
| **H4-B** | Discordancias entre la IR v0.4 y el contrato WIT son detectadas determinísticamente por el verificador con JSONPath exacto. | `E2-004` detecta desalineación de tipo en campo de interfaz WIT (`E_WIT_TYPE_MISMATCH`), reportando ruta precisa y siendo reparada en <= 2 intentos. | El verificador omite la comprobación WIT o emite mensajes de error no accionables. |
| **H4-C** | Nuevas interfaces o tipos variantes en WIT pueden integrarse a la IR con el backend del compilador congelado. | `E3-004` incorpora un nuevo método de interfaz WIT (`quote-shipping`) sin requerir parches a los binarios de toolchain sellados en `E1-004`. | La incorporación de nuevas firmas WIT exige recompilar `irtool`. |
| **H4-D** | El empaquetado de componentes preserva la reproducibilidad binaria exacta bit a bit. | 3 compilaciones sucesivas de la misma IR v0.4 generan componentes WASM con idéntico hash SHA-256. | El proceso de Canonical ABI o encoding de componentes introduce metadatos no reproducibles. |
| **H4-E** | El Component Runtime de Wasmtime ejecuta componentes Agent-Native sin sobrecarga desmedida. | La latencia de ejecución en Wasmtime Component Runtime se mantiene en < 15 ms para suites de prueba completas. | La instanciación de componentes degrada severamente el tiempo de test. |

## Constraints

- Los contratos de interfaces de imports y exports se definen en archivos estándar `.wit`.
- La Semantic IR v0.4 referencia explícitamente los paquetes e interfaces WIT declarados.
- El micro-toolchain genera artefactos que cumplen con la especificación de WebAssembly Component Model (preámbulo de componente o empaquetado estándar).
- La toolchain (`irtool` y `runner`) se congela tras culminar `E1-004` y sus hashes criptográficos se custodian en `runs/toolchain-freeze-poc004.sha256`.
- El agente nunca edita archivos WAT ni binarios `.wasm` directamente.
- Todas las aserciones de prueba en `runner` se ejecutan contra la interfaz exportada del componente en Wasmtime.

## Non-goals

1. **No composición dinámica en runtime (no WASI dynamic plugin host):** El enlace de componentes se valida en tiempo de prueba sobre interfaces estáticas.
2. **No asincronía / WASI 0.3 async streams:** Todas las funciones de interfaz WIT son invocaciones síncronas en esta fase.
3. **No runtime de recolección de basura compleja (Wasm GC):** El modelo de memoria lineal interna encapsulada del componente sigue basándose en gestión lineal determinista.

## Scenario

### Alcance técnico de la Semantic IR v0.4

| Categoría | Elementos soportados en v0.4 | Novedad frente a v0.3 |
|---|---|---|
| **Definición de Interfaces WIT** | Archivos `wit/*.wit` con `package`, `interface`, `record`, `variant`, `func` | **Nuevo en v0.4** |
| **Vinculación a Contratos WIT** | Declaración `wit_interfaces: [...]` y mapeo de exportación a interfaz WIT | **Nuevo en v0.4** |
| **WebAssembly Component Packaging** | Emisión de binario componente WebAssembly ejecutable en Wasmtime Component Linker | **Nuevo en v0.4** |
| **Canonical ABI Mapping** | Normalización de pasaje de parámetros estructurados a través del límite del componente | **Nuevo en v0.4** |
| **Declaración de Capabilities** | Bloque raíz `capabilities` validado contra las interfaces importadas | Reforzado |
| **Composición modular interna** | `call` a subfunciones internas | Heredado de v0.2/v0.3 |
| **Errores de dominio tipados** | Variantes tipadas de error mapeadas a `variant` / `result<T, E>` en WIT | **Reforzado en v0.4** |

## Success Criteria

- [ ] **SC-01:** Construcción inicial `E1-004` completada por el agente en <= 3 intentos a partir de la especificación WIT v1-004.
- [ ] **SC-02:** Corrección funcional `E1-004`: 100 % de pruebas públicas y reservadas aprobadas en Wasmtime Component Runtime.
- [ ] **SC-03:** Congelamiento de toolchain sellado: Hashes SHA-256 de `irtool` y `runner` registrados en `runs/toolchain-freeze-poc004.sha256` y tag git `poc-004-e1-freeze`.
- [ ] **SC-04:** Detección determinista de desalineación con interfaz WIT en `E2-004`: rechazo con exit code != 0 y diagnóstico JSONPath explícito (`E_WIT_TYPE_MISMATCH` o `E_UNKNOWN_EXPORT_INTERFACE`).
- [ ] **SC-05:** Reparación controlada `E2-004`: reparado por el agente en <= 2 intentos recuperando 100 % pass rate y paridad SHA-256 del componente.
- [ ] **SC-06:** Extensión funcional `E3-004` con nueva interface o tipo variante en WIT completada en <= 3 intentos.
- [ ] **SC-07:** Inmutabilidad del backend en `E3-004`: 0 cambios en el backend congelado (mismos hashes de toolchain).
- [ ] **SC-08:** Preservación de regresión en `E3-004`: 100 % de pruebas de suites previas y ampliadas aprobadas.
- [ ] **SC-09:** Inmutabilidad del backend en `E4-004`: 0 cambios en el backend congelado.
- [ ] **SC-10:** Aprobación de mantenimiento modular en `E4-004`: 100 % de pruebas aprobadas con diff localizado.
- [ ] **SC-11:** Reproducibilidad binaria exacta: 3/3 builds deterministas producen el mismo SHA-256 del componente `.wasm`.
- [ ] **SC-12:** Integridad de roles (agente): 0 líneas de WAT o WASM editadas por el agente.
- [ ] **SC-13:** Integridad del estado de software: Semantic IR v0.4 es el único artefacto de código modificado por el agente.
- [ ] **SC-14:** Compacidad diagnóstica: salida de diagnóstico JSON estructurada con JSONPath accionable < 600 bytes.
- [ ] **SC-15:** Latencia del bucle determinista: `verify` + `compile/package` < 60 ms.
- [ ] **SC-16:** Validación estricta de WebAssembly Component: el artefacto pasa validación de componentes de Wasmtime sin errores de tipo ni de ABI.

## Work Items Plan (Fase 4)

| ID | Tipo | Título | Estado | Entregables principales |
|---|---|---|---|---|
| **`WI-025`** | spike | Contrato experimental, Interfaces WIT y Semantic IR v0.4 | `completed` | Definición de interfaz `wit/order.wit`, esquema `ir/ir.schema.v4.json`, especificación `spec/task-004.yaml` (v1) y suite inicial `tests/public-004.yaml`. |
| **`WI-026`** | spike | Verifier v0.4 + Component Packaging / Canonical ABI + Runner Component Linker | `draft` | Soporte en `irtool` para validar compatibilidad con WIT y empaquetar WebAssembly Component; soporte en `runner` para ejecutar componentes en Wasmtime Component Model. |
| **`WI-027`** | spike | E1-004 Construcción de Componente con Interfaces WIT y Freeze | `draft` | Construcción de `ir/program-004.ir.json`, verificación contra WIT, emisión de componente WASM, 100 % pass rate y sellado de `poc-004-e1-freeze`. |
| **`WI-028`** | spike | E2-004 Reparación controlada guiada por WIT diagnostics | `draft` | Inyección de desalineación contra la interfaz WIT (`E_WIT_TYPE_MISMATCH`), rechazo determinista y reparación en 1 intento. |
| **`WI-029`** | spike | E3-004 Extensión funcional con interfaces y variantes ricas en WIT | `draft` | Extensión de especificación WIT con nueva operación (`quote-shipping`) y validación sobre backend congelado. |
| **`WI-030`** | spike | E4-004 Mantenimiento modularizado y composición interoperable | `draft` | Mantenimiento localizado de reglas comerciales preservando la interfaz WIT y reproducibilidad 3/3. |
| **`WI-031`** | chore | Análisis, matriz SC y conclusión POC 004 | `draft` | Evaluación de `SC-01` a `SC-16`, reporte final `poc-report-v005.md` y decisión canónica Kaddo. |
| **`WI-032`** | spike opcional | Benchmark pareado POC-004 vs Rust `cargo component` | `draft` | Comparación cuantitativa: Semantic IR v0.4 vs `cargo-component` con bindings WIT automáticos. |

## Conclusion Rule

- **GO (`validated`):** Se cumplen `SC-01` a `SC-16`. Justifica avanzar a la Fase 5 (**POC 005**: Orquestación y composición multi-componente agent-native con Component Model Linker y registro federado).
- **PIVOT (`inconclusive`):** Los componentes WebAssembly son ejecutables pero la complejidad del Canonical ABI degrada la velocidad de feedback de la toolchain por encima de los 100 ms.
- **STOP (`rejected`):** El agente no logra mapear conceptos de interfaz WIT en Semantic IR v0.4 o la emisión de componentes no puede mantenerse determinista y congelada.
