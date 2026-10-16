---
type: poc
status: validated
conclusion: validated
decision: GO
generated_by: kaddo-init
template_version: 1
project_state: ai-assisted
refined_by: work-item-agent
initiative: KADDO-AGENT-NATIVE-POC-005
---

# Kaddo Agent-Native POC 005 — De la narrativa al software ejecutable

> **Artefacto canónico activo:** `knowledge/delivery/poc.md` (espejado en `knowledge/delivery/poc-005.md`)
>
> **Referencias históricas previas:**
> - [POC 001 Validada (GO)](knowledge/delivery/poc-001.md) | [Reporte Final POC 001](knowledge/delivery/poc-report-v001.md) | [Benchmark 001](BENCHMARK_REPORT.md)
> - [POC 002 Validada (GO)](knowledge/delivery/poc-002.md) | [Reporte Final POC 002](knowledge/delivery/poc-report-v003.md) | [Benchmark 002](BENCHMARK_REPORT_002.md)
> - [POC 003 Validada (GO)](knowledge/delivery/poc-003.md) | [Reporte Final POC 003](knowledge/delivery/poc-report-v004.md) | [Benchmark 003](BENCHMARK_REPORT_003.md)
> - [POC 004 Validada (GO)](knowledge/delivery/poc-004.md) | [Reporte Final POC 004](knowledge/delivery/poc-report-v006.md) | [Benchmark 004](BENCHMARK_REPORT_004.md)
> - [Reporte Final POC 005](knowledge/delivery/poc-report-v007.md) | [Benchmark 005](BENCHMARK_REPORT_005.md)
>
> **Naturaleza:** POC de investigación técnica, empírica, falsable y reproducible.
>
> **Modalidad:** Kaddo POC Mode (Fase 5).

**Subtítulo:** Orquestación de desarrollo, especificación consolidada, evolución incremental y composición de componentes WebAssembly.

**Estado:** Validada (`validated`) — Decisión: **GO**.

## 1. Descripción general

La POC 005 busca validar si Agent-Native puede evolucionar desde un motor experimental de Semantic IR hacia un sistema capaz de construir y mantener software a partir de especificaciones narrativas, sin exigir que las personas escriban o modifiquen directamente la representación intermedia.

El experimento incorporará un **orquestador de desarrollo** que interprete una narrativa funcional completa o una secuencia de Work Items, consolide los requisitos vigentes del sistema y coordine a los agentes encargados de generar o modificar la Semantic IR. El resultado deberá verificarse y compilarse mediante la toolchain determinista desarrollada en las POC anteriores, produciendo componentes WebAssembly ejecutables y trazables.

La POC también deberá comprobar que Agent-Native funciona como tecnología independiente de Kaddo. Para ello, se validará un modo standalone que pueda consumir especificaciones directamente, junto con un mecanismo de integración opcional que permita aprovechar el conocimiento y los Work Items de Kaddo. Ambos caminos utilizarán el mismo motor y deberán preservar los contratos, pruebas y comportamiento funcional del software.

## 2. Pregunta de investigación

¿Puede un orquestador Agent-Native construir, modificar y mantener una aplicación a partir de una narrativa funcional completa o de múltiples Work Items incrementales, consolidando los requisitos en una especificación vigente y generando componentes WebAssembly interoperables, sin intervención humana directa sobre la Semantic IR ni dependencia obligatoria de Kaddo?

## 3. Hipótesis principal

Dada una descripción funcional suficientemente precisa de un sistema, un orquestador Agent-Native puede interpretar su intención, generar una especificación estructurada, coordinar la construcción de su Semantic IR y producir componentes WebAssembly ejecutables mediante herramientas deterministas.

Posteriormente, ante modificaciones narrativas o nuevos Work Items, el orquestador puede identificar los elementos afectados, actualizar la especificación vigente y modificar la IR preservando las funcionalidades anteriores mediante pruebas de regresión, trazabilidad y control de contratos.

Esta capacidad deberá funcionar tanto de manera independiente como mediante una integración opcional con Kaddo, sin introducir lógica comercial específica dentro del compilador ni requerir cambios en la toolchain para cada modificación de negocio.

## 4. Alcance funcional

### A. Construcción desde una narrativa única

El sistema deberá aceptar un documento Markdown que describa el comportamiento esperado de una aplicación. La narrativa podrá incluir procesos, operaciones, reglas comerciales, entidades, restricciones, dependencias y criterios de aceptación expresados en lenguaje natural.

El orquestador deberá interpretar el documento, identificar los requisitos funcionales y transformarlos en una especificación estructurada que pueda verificarse antes de generar la IR. Las decisiones ambiguas o incompletas deberán quedar identificadas explícitamente, evitando que el agente invente silenciosamente comportamientos no definidos.

La experiencia buscada es que una persona pueda describir qué debe hacer un sistema sin necesitar conocimientos de Semantic IR, WIT o WebAssembly.

### B. Construcción incremental desde múltiples Work Items

El sistema deberá aceptar una secuencia de Work Items que describan la construcción inicial y la evolución de una misma aplicación. Cada Work Item podrá incorporar funcionalidades, modificar reglas existentes, corregir defectos o retirar comportamientos que hayan dejado de ser válidos.

Los Work Items no se concatenarán directamente para producir el programa. El orquestador deberá interpretar sus relaciones, identificar requisitos vigentes y obsoletos, resolver contradicciones mediante decisiones explícitas y actualizar una **System Specification** versionada que represente el estado funcional esperado del software.

La Semantic IR deberá evolucionar a partir de esa especificación consolidada, preservando la trazabilidad de cada elemento hacia los requisitos y Work Items que justificaron su creación o modificación.

### C. Especificación consolidada del sistema

Se introducirá un artefacto denominado provisionalmente **System Specification**, cuya responsabilidad será representar los requisitos funcionales vigentes, las operaciones, los tipos, los contratos de interfaz, las dependencias y los criterios de aceptación del software.

El actual `task.yaml` podrá evolucionar hacia un contrato derivado de esta especificación. No deberá convertirse en una segunda fuente de verdad mantenida manualmente, sino en una representación estructurada y reproducible que el motor pueda consumir para implementar cambios específicos.

La especificación consolidada deberá mantener identificadores estables de requisitos, referencias a sus fuentes y versiones, permitiendo distinguir entre comportamiento vigente, modificado, reemplazado y retirado.

### D. Orquestador Agent-Native

Se desarrollará un orquestador independiente que coordine el ciclo entre la intención funcional y la generación del software. Su responsabilidad será preparar contexto, interpretar cambios, generar o actualizar especificaciones, solicitar la construcción de IR, ejecutar verificaciones y administrar los ciclos de corrección.

El orquestador deberá gestionar los diagnósticos producidos por el verificador y proporcionar al agente información suficiente para corregir errores sin modificar manualmente el código WebAssembly. También deberá identificar qué elementos han cambiado y ejecutar pruebas de aceptación y regresión sobre el comportamiento afectado y sus dependencias.

Se diferenciará explícitamente la orquestación del proceso de desarrollo de la composición de componentes durante la ejecución. El orquestador no deberá implementar internamente un nuevo compilador ni asumir las responsabilidades del runtime.

### E. Composición de componentes WebAssembly

Se validará la construcción y ejecución de una aplicación mínima compuesta por al menos dos componentes WebAssembly con responsabilidades independientes. Los componentes deberán comunicar sus capacidades mediante contratos WIT explícitos y utilizar el WebAssembly Component Model nativo validado en la POC 004.

El escenario deberá demostrar que una operación definida en un componente puede ser consumida por otro, preservando el contrato tipado y la separación de responsabilidades. El host y el runtime podrán proporcionar infraestructura y capacidades externas, pero no deberán incorporar implementaciones alternativas de las reglas comerciales como mecanismo para superar limitaciones del motor.

La composición deberá ser estática y reproducible durante esta fase. No se requiere implementar descubrimiento dinámico, registro federado ni una plataforma completa de microservicios.

### F. Modo standalone e integración opcional con Kaddo

Agent-Native deberá funcionar independientemente de Kaddo, aceptando una narrativa o especificación desde el sistema de archivos y produciendo los artefactos necesarios para verificar, compilar y ejecutar el software.

Adicionalmente, se validará un adaptador mínimo que permita consumir documentos de conocimiento y Work Items de un proyecto Kaddo. Esta integración deberá traducir el contexto necesario hacia los contratos del orquestador y recuperar la evidencia generada, sin introducir dependencias obligatorias del motor respecto del core de Kaddo.

La POC deberá comprobar que ambas modalidades utilizan la misma Semantic IR y la misma toolchain, diferenciándose únicamente en la procedencia y preparación del contexto. No forma parte de esta fase implementar la experiencia definitiva en el Admin, el CLI o el MCP de Kaddo.

### G. Trazabilidad y evidencia reproducible

Cada proceso de construcción y modificación deberá conservar la relación entre la narrativa o Work Items de origen, la System Specification, las modificaciones de Semantic IR, los contratos WIT, los resultados de verificación y los componentes WASM generados.

Se registrarán los diagnósticos, las pruebas públicas y reservadas, las modificaciones realizadas y los hashes de los artefactos. También se medirá el proceso completo de desarrollo, incluyendo intentos del agente, duración total y modificaciones del motor, diferenciándolo del tiempo de verificación y compilación determinista.

El objetivo es poder explicar qué cambió, por qué cambió, qué parte del software resultó afectada y qué evidencia demuestra que el comportamiento esperado se conserva.

## 5. Escenario experimental

Se utilizará una aplicación comercial acotada, aprovechando el dominio de órdenes de compra, descuentos, impuestos, crédito y auditoría desarrollado durante las POC anteriores.

La aplicación se dividirá en al menos dos componentes funcionales: uno encargado del cálculo comercial (`order-calculator`) y otro de las reglas de validación o cotización de envíos (`shipping-calculator`). Sus responsabilidades deberán quedar expresadas en contratos WIT independientes, de manera que el escenario permita comprobar una interacción real entre componentes.

Se ejecutarán cuatro ejercicios principales:

- **E1 — Construcción desde narrativa:** El agente recibe una descripción completa del sistema (`spec/narrative-005.md`) y construye la System Specification, la IR y los componentes WebAssembly necesarios. Se verifica el comportamiento funcional y se establece una versión base reproducible y freeze de toolchain.
- **E2 — Construcción desde Work Items:** En un entorno aislado, el agente recibe una secuencia equivalente de Work Items y construye la misma aplicación. Se compara el comportamiento mediante las mismas pruebas y contratos externos, sin exigir que la IR o los binarios resultantes sean idénticos.
- **E3 — Evolución incremental:** Se modifica una regla comercial y se incorpora una nueva capacidad mediante un Work Item. El orquestador debe actualizar la especificación vigente, localizar los elementos afectados, modificar la IR existente y comprobar que las funcionalidades no relacionadas continúan funcionando.
- **E4 — Composición y mantenimiento:** Se ejecuta la aplicación mediante al menos dos componentes WebAssembly conectados por contratos WIT. Posteriormente se modifica una funcionalidad de uno de ellos, verificando la compatibilidad de interfaces, las pruebas de integración, la reproducibilidad y la ausencia de cambios innecesarios en el otro componente.

Validaciones transversales:
- Ejecución en modo standalone puro y con adaptador Kaddo.
- Recuperación ante defectos controlados mediante diagnósticos deterministas.

## 6. Success Criteria (Criterios de éxito principales)

- [x] **SC-01:** Equivalencia funcional: La narrativa única (E1) y la secuencia equivalente de Work Items (E2) producen aplicaciones funcionalmente equivalentes, verificadas mediante contratos WIT y suites de pruebas comunes (21/21 passed, SHA-256 binario idéntico).
- [x] **SC-02:** Consolidación de especificación: El orquestador genera una System Specification consolidada sin contradicciones no resueltas, conservando procedencia y versiones de los requisitos (`system-spec.json`).
- [x] **SC-03:** Evolución sin intervención manual: Las modificaciones incrementales (E3/E4) se implementan sin edición manual humana de Semantic IR, WAT ni WASM, y sin reconstrucciones innecesarias del programa.
- [x] **SC-04:** WebAssembly Components nativos interoperables: Los componentes generados son WebAssembly Components nativos válidos (`0x0d` magic), ejecutables en Wasmtime y capaces de interoperar entre sí mediante interfaces WIT reales (`kaddo:shipping/shipping-service@0.1.0`).
- [x] **SC-05:** Aprobación de suites de pruebas: La solución supera el 100 % de las pruebas de aceptación, integración y regresión (60/60 ejecuciones exitosas en Wasmtime).
- [x] **SC-06:** Dualidad Standalone e Integración Kaddo: El motor ejecuta el mismo flujo en modo standalone y mediante la integración opcional con Kaddo con resultados y artefactos idénticos.
- [x] **SC-07:** Trazabilidad integral de extremo a extremo: Se conserva la trazabilidad desde la narrativa o Work Item hasta la System Specification, IR, componente ejecutable y resultados de prueba.
- [x] **SC-08:** Inmutabilidad de la Toolchain: Los cambios de negocio que utilicen capacidades ya soportadas no requieren modificar el verificador, compilador ni runtime tras el freeze de E1 (`runs/toolchain-freeze-poc005.sha256`).
- [x] **SC-09:** Medición del ciclo de desarrollo completo: Se registra evidencia suficiente para reproducir ejecuciones y comparar costos y tiempos del proceso completo de desarrollo en `BENCHMARK_REPORT_005.md`.
- [x] **SC-10:** Desacoplamiento dinámico de contratos WIT: Los contratos WIT se cargan y resuelven desde el proyecto experimental sin depender de interfaces comerciales incorporadas de forma fija en el código de `irtool` o `runner`.

## 7. Restricciones y exclusiones

La POC continuará dentro del repositorio experimental `poc-agent-native`, reutilizando las capacidades de las versiones anteriores y conservando sus pruebas de regresión. Su finalidad será validar la arquitectura y sus contratos antes de extraer un producto independiente.

No se incluye en esta fase desarrollar un lenguaje de propósito general, implementar un backend HTTP de producción, incorporar una base de datos real, crear un registro federado de componentes ni resolver todas las capacidades de networking, concurrencia, asincronía y seguridad operacional.

Tampoco se incluye trasladar el compilador al monorepo de Kaddo, publicar los paquetes definitivos de Agent-Native, desarrollar una interfaz visual completa ni consolidar el producto comercial. Esas actividades corresponderán a una fase posterior de extracción, estabilización e integración.

## 8. Arquitectura objetivo

El flujo conceptual será:

```text
                 FUENTES DE INTENCIÓN
              ┌──────────────────────┐
              │ Narrativa completa   │
              │        o             │
              │ Work Items de Kaddo  │
              └──────────┬───────────┘
                         │
                         ▼
              Agent-Native Orchestrator
                         │
                         ▼
                System Specification
                         │
                         ▼
               Agente de implementación
                         │
                         ▼
                    Semantic IR
                         │
                         ▼
              Verificador determinista
                         │
                         ▼
              Compilador / Component Model
                         │
                         ▼
               WebAssembly Components
                         │
                         ▼
               Runtime / Composición
                         │
                         ▼
               Pruebas y evidencias
                         │
                         └──► Retroalimentación
                              y trazabilidad
```

La intención y la narrativa permanecerán separadas de la representación implementada. El orquestador coordinará la evolución del conocimiento y del software, mientras la toolchain determinista será responsable de verificar y generar los componentes ejecutables.

## 9. Plan de Work Items (Fase 5)

| ID | Tipo | Título | Estado | Entregables principales |
|---|---|---|---|---|
| **`WI-033`** | spike | Contrato experimental, esquema System Specification y narrativa base | `completed` | Esquema `system-spec.schema.json`, narrativa funcional `spec/narrative-005.md`, contratos WIT multi-componente y suites de pruebas de referencia. |
| **`WI-034`** | spike | Orquestador Agent-Native y adaptador Kaddo | `completed` | Crate `crates/orchestrator` con ingestor narrativo, consolidador de System Specification, gestor de diagnósticos y adaptador standalone / Kaddo. |
| **`WI-035`** | spike | Component Model Linker multi-componente en Runner | `completed` | Soporte en `runner` para componer, enlazar e instanciar múltiples WebAssembly Components dependientes en Wasmtime. |
| **`WI-036`** | spike | E1-005 Construcción desde narrativa única y freeze | `completed` | Ejecución de E1: narrativa -> System Specification -> Semantic IR -> componentes WASM nativos -> 100 % pass rate y sellado de toolchain freeze. |
| **`WI-037`** | spike | E2-005 Construcción desde Work Items equivalentes | `completed` | Ejecución de E2: secuencia de Work Items -> System Specification -> componentes WASM -> validación de paridad funcional con E1. |
| **`WI-038`** | spike | E3-005 Evolución incremental guiada por nuevo Work Item | `completed` | Ejecución de E3: nuevo Work Item -> actualización de System Specification -> diff de IR -> regresión 100 % con backend congelado. |
| **`WI-039`** | spike | E4-005 Composición multi-componente y mantenimiento localizado | `completed` | Ejecución de E4: modificación en un componente manteniendo intacto el otro, compatibilidad WIT y reproducibilidad 3/3. |
| **`WI-040`** | chore | Validación dual (Standalone/Kaddo), análisis y conclusión POC 005 | `completed` | Validación transversal, matriz SC-01 a SC-10, reporte final de POC 005 (`poc-report-v007.md`) y decisión GO. |
| **`WI-041`** | spike | Benchmark del ciclo de vida completo: Agent-Native vs Tradicional | `completed` | Medición cuantitativa del tiempo y esfuerzo de ciclo completo de desarrollo en `BENCHMARK_REPORT_005.md`. |

## 10. Resultado esperado y transición posterior

Al finalizar la POC 005 se espera disponer de un prototipo reproducible de Agent-Native capaz de construir software desde una narrativa y evolucionarlo mediante cambios incrementales, utilizando componentes WebAssembly interoperables y un motor independiente de Kaddo.

La validación deberá permitir identificar los contratos reutilizables del compilador, la Semantic IR, el runtime y el orquestador, así como los límites entre las responsabilidades del motor y las del ecosistema Kaddo.

Si el resultado es favorable, se recomendará iniciar una fase de **extracción y estabilización**, creando un repositorio independiente para Agent-Native, con paquetes internos separados para IR, verificación, compilación, runtime y orquestación. Kaddo incorporará posteriormente un adaptador opcional que consuma esa capacidad mediante contratos versionados.
