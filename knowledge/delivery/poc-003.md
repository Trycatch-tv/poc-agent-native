---
type: poc
status: completed
conclusion: validated
generated_by: kaddo-init
template_version: 1
project_state: ai-assisted
refined_by: work-item-agent
initiative: KADDO-AGENT-NATIVE-POC-003
---

# Kaddo Agent-Native POC 003 — Capability ABI, Imports Declarativos y Efectos Secundarios Auditables en Semantic IR v0.3

> **Artefacto canónico:** `knowledge/delivery/poc.md`
>
> **Referencias históricas previas:**
> - [POC 001 Validada (GO)](knowledge/delivery/poc-001.md) | [Reporte Final POC 001](knowledge/delivery/poc-report-v001.md) | [Benchmark 001](BENCHMARK_REPORT.md)
> - [POC 002 Validada (GO)](knowledge/delivery/poc-002.md) | [Reporte Final POC 002](knowledge/delivery/poc-report-v003.md) | [Benchmark 002](BENCHMARK_REPORT_002.md)
>
> **Pregunta de investigación:** ¿puede un agente de IA interactuar con servicios externos (almacenamiento clave-valor, telemetría y auditoría de eventos) mediante un Capability ABI declarativo e imports tipados en Semantic IR v0.3, mientras herramientas deterministas verifican autorizaciones estáticas, generan la sección `import` de WebAssembly y ejecutan suites con host mocks en Wasmtime sin comprometer el determinismo de la lógica de negocio ni la inmutabilidad del compilador post-freeze?
>
> **Naturaleza:** POC de investigación técnica, empírica, falsable y reproducible.
>
> **Modalidad:** Kaddo POC Mode (Fase 3).

## Problem

En las fases anteriores, `POC 001` (funciones escalares puras) y `POC 002` (composición modular, memoria lineal determinista y discriminación de errores) demostraron ventajas contundentes de velocidad (`10x–19x`), tamaño de binario y compacidad diagnóstica (`9.1x–23x`) frente a Rust.

Sin embargo, en sistemas reales de producción, las reglas de negocio no existen en el vacío:
1. **Acceso a estado externo:** una orden comercial necesita consultar inventarios dinámicos o alícuotas fiscales almacenadas externamente (e.g. `kv_store.get(key)`).
2. **Efectos secundarios controlados:** transacciones exitosas o fallos críticos requieren emitir eventos de auditoría y telemetría estructurada (e.g. `audit_log.emit(event_id, payload)`).
3. **El dilema de la I/O no controlada:** si se permite al agente usar llamadas de sistema arbitrarias (syscalls/sockets) o dependencias opacas, se destruye el determinismo, se abre la puerta a vectores de inyección/alucinación de I/O y se pierde la capacidad de testeo puramente aislado.

El problema a resolver en esta **POC 003** es concebir y validar un **Capability ABI declarativo**: el módulo WebAssembly declara estrictamente qué capacidades de host requiere; el verificador valida que el agente no invoque operaciones de host no autorizadas; el compilador emite la sección `import` de WebAssembly Core; y el runner en Wasmtime enlaza mocks deterministas que permiten verificar los efectos secundarios y suministrar fixtures de host sin modificar el backend una vez congelado.

## Hypothesis

### Hipótesis principal

> **H3.** Mediante la especificación de un Capability ABI declarativo en la Semantic IR v0.3, un agente de IA puede construir, reparar, extender y mantener programas que consuman servicios de host (lectura de estado y emisión de eventos) respetando los límites de autorización estática, mientras el micro-toolchain determinista valida firmas de importación, emite bytecode WASM estándar y ejecuta pruebas con mocks de host en Wasmtime, manteniendo el backend congelado tras `E1-003` y preservando un tiempo de ciclo < 60 ms y diagnósticos compactos < 500 bytes.

### Subhipótesis

| ID | Hipótesis | Evidencia favorable | Evidencia desfavorable |
|---|---|---|---|
| **H3-A** | El agente puede declarar y consumir capacidades de host tipadas (`call_import`) a partir de la especificación. | `E1-003` supera verificación semántica y 100 % de pruebas con host mocks en <= 3 intentos. | El agente no logra enlazar parámetros de importación o excede los intentos permitidos. |
| **H3-B** | Violaciones de autorización o firmas de importación son diagnosticadas determinísticamente por JSONPath. | `E2-003` detecta uso de capability no declarada (`E_UNDECLARED_CAPABILITY`), reportando la ruta exacta y siendo reparada en <= 2 intentos. | El verificador omite la comprobación estática o emite diagnósticos no accionables. |
| **H3-C** | Nuevas capacidades de host pueden integrarse a la IR con el backend del compilador congelado. | `E3-003` incorpora validación de crédito externo sin requerir parches a los binarios de toolchain sellados en `E1-003`. | Cada nueva función importada obliga a recompilar `irtool`. |
| **H3-D** | Los efectos secundarios (eventos emitidos) son deterministas y auditables en runner. | Las aserciones de pruebas verifican tanto los valores de retorno como la secuencia exacta de llamadas al host. | Las llamadas al host producen comportamiento no determinista o estados no reproducibles. |
| **H3-E** | La reproducibilidad binaria exacta se conserva con secciones `import`. | 3 compilaciones sucesivas de la misma IR v0.3 con imports producen idéntico SHA-256 en WASM. | El ordenamiento de imports genera fluctuaciones en los índices de funciones del módulo. |

## Constraints

- La interacción con el exterior ocurre exclusivamente mediante funciones de importación declaradas en el módulo (`(import "host" <name> (func ...))`).
- No se permiten punteros a memoria arbitraria del host: los datos se intercambian como escalares (`i32`) o punteros a la memoria lineal del módulo WASM.
- La toolchain (`irtool` y `runner`) se congela tras culminar `E1-003` y sus hashes criptográficos se custodian en `runs/toolchain-freeze-poc003.sha256`.
- El agente nunca edita archivos WAT ni binarios WASM directamente.
- Todas las aserciones de prueba en `runner` deben validar tanto los retornos de las funciones como los efectos secundarios en el host mock.

## Non-goals

1. **No sockets de red directos ni filesystem POSIX:** No se implementará acceso no mediado al sistema operativo (no WASI preview 1 libc ni sockets TCP).
2. **No asincronía compleja / Promises en runtime:** Todas las llamadas a capabilities de host son síncronas en el límite de invocación WASM Core.
3. **No Garbage Collection:** La memoria se sigue gestionando mediante el modelo determinista bump en memoria lineal.

## Scenario

### Alcance técnico de la Semantic IR v0.3

| Categoría | Elementos soportados en v0.3 | Novedad frente a v0.2 |
|---|---|---|
| **Declaración de Capabilities** | Bloque raíz `capabilities: [...]` con permisos explícitos de host | **Nuevo en v0.3** |
| **Catálogo de Imports** | Declaración de funciones importadas del host (`module`, `name`, `params`, `returns`) | **Nuevo en v0.3** |
| **Invocación de Host** | Operador `call_import` con chequeo estricto de autorización y tipos | **Nuevo en v0.3** |
| **Host Mocking en Runner** | Fixtures de host (KV store inicial) y captura de llamadas de auditoría en Wasmtime Linker | **Nuevo en v0.3** |
| **Tipos escalares y compuestos** | `i32`, `bool`, `record` (memoria lineal Core WASM) | Heredado de v0.2 |
| **Composición modular interna** | `call` a subfunciones internas | Heredado de v0.2 |
| **Errores de dominio tipados** | Variantes y códigos de estado (`status_code`) | Heredado de v0.2 |
| **Trazabilidad** | IDs de requisitos trazados a funciones, variantes e imports | Reforzado |

## Success Criteria

- [x] **SC-01:** Construcción inicial `E1-003` completada por el agente en <= 3 intentos desde la especificación v1-003 (logrado en intento 1).
- [x] **SC-02:** Corrección funcional `E1-003`: 100 % de pruebas públicas y reservadas aprobadas (retorno y efectos secundarios en host; 15/15 passed).
- [x] **SC-03:** Congelamiento de toolchain sellado: Hashes SHA-256 de `irtool` y `runner` registrados en `runs/toolchain-freeze-poc003.sha256` y tag git `poc-003-e1-freeze`.
- [x] **SC-04:** Detección determinista de violación de Capability en `E2-003`: rechazo con exit code != 0 y diagnóstico JSONPath explícito (`E_UNDECLARED_CAPABILITY`).
- [x] **SC-05:** Reparación controlada `E2-003`: reparado por el agente en <= 2 intentos recuperando 100 % pass rate y paridad SHA-256 (reparado en 1 intento).
- [x] **SC-06:** Extensión funcional `E3-003` con consumo de nueva capability de host completada en <= 3 intentos (logrado en intento 1).
- [x] **SC-07:** Inmutabilidad del backend en `E3-003`: 0 cambios en el backend congelado (mismos hashes de toolchain).
- [x] **SC-08:** Preservación de regresión en `E3-003`: 100 % de pruebas de suites previas y ampliadas aprobadas (59/59 casos totales).
- [x] **SC-09:** Inmutabilidad del backend en `E4-003`: 0 cambios en el backend congelado.
- [x] **SC-10:** Aprobación de mantenimiento modular en `E4-003`: 100 % de pruebas aprobadas con diff localizado (17/17 casos aprobados).
- [x] **SC-11:** Reproducibilidad binaria exacta: 3/3 builds deterministas producen el mismo SHA-256 de `.wasm` (`637ee58cf943616fa80db2d87abb53a22ec04755a7ba001b3fbd7e79ad003be7` en E1/E2, `bf2021f69c6d122a3bb08630c21e7fd7471b64434f806fed5cdea4b2b99fb887` en E4).
- [x] **SC-12:** Integridad de roles (agente): 0 líneas de WAT o WASM editadas por el agente.
- [x] **SC-13:** Integridad del estado de software: Semantic IR v0.3 es el único artefacto modificado por el agente.
- [x] **SC-14:** Compacidad diagnóstica: salida de diagnóstico JSON estructurada con JSONPath accionable < 600 bytes.
- [x] **SC-15:** Latencia del bucle determinista: `verify` + `compile` < 60 ms (~27 ms).
- [x] **SC-16:** Validación estricta de WebAssembly: `.wasm` pasa `wasmparser` con sección `import` válida y sin violaciones de tipo ni memoria.

## Work Items Plan (Fase 3)

| ID | Tipo | Título | Estado | Entregables principales |
|---|---|---|---|---|
| **`WI-017`** | spike | Contrato experimental y Semantic IR v0.3 | `completed` | `ir/ir.schema.v3.json` (declaración de capabilities e imports), especificación `spec/task-003.yaml` (v1) y suite de pruebas `tests/public-003.yaml` con aserciones de host. |
| **`WI-018`** | spike | Verifier v0.3 + Compiler con Imports WASM + Runner con Host Linker | `completed` | Extensión en `irtool` para validar autorización de capabilities y emitir la sección `import` de WASM; soporte en `runner` para enlazar mocks de KV store y auditoría en Wasmtime. |
| **`WI-019`** | spike | E1-003 Construcción con Capabilities y Freeze | `completed` | Implementación de cálculo de orden con consulta de inventario/descuento en host KV y emisión de auditoría, 100 % pass rate y sellado de `poc-003-e1-freeze`. |
| **`WI-020`** | spike | E2-003 Reparación controlada guiada por Capability diagnostics | `completed` | Inyección de violación de autorización de capability (`E_UNDECLARED_CAPABILITY`), rechazo determinista y reparación en 1 intento. |
| **`WI-021`** | spike | E3-003 Extensión funcional con nueva regla de host | `completed` | Integración de consulta de límite de crédito de cliente en host sobre backend congelado. |
| **`WI-022`** | spike | E4-003 Mantenimiento modularizado y auditabilidad | `completed` | Actualización localizada de regla comercial y eventos de auditoría con diff conciso y reproducibilidad 3/3. |
| **`WI-023`** | chore | Análisis, matriz SC y conclusión POC 003 | `completed` | Evaluación de `SC-01` a `SC-16`, reporte final `poc-report-v004.md`, decisión Kaddo (`GO / PIVOT / STOP`). |
| **`WI-024`** | spike opcional | Benchmark pareado POC-003 vs Rust con Host Imports | `completed` | Comparación cuantitativa: Semantic IR v0.3 vs `Rust` (`extern "C"` imports a host Wasmtime); publicado en `BENCHMARK_REPORT_003.md`. |

## Conclusion Rule

- **GO (`validated`):** Se cumplen `SC-01` a `SC-16`. Justifica avanzar a la Fase 4 (**POC 004**: WebAssembly Component Model completo, interfaces WIT estandarizadas y empaquetado Wasm Component).
- **PIVOT (`inconclusive`):** Los imports de host funcionan pero la verificación estática de capabilities o el testing de efectos secundarios resulta frágil sin un runtime asíncrono.
- **STOP (`rejected`):** El agente no logra razonar sobre autorizaciones de host en la IR v0.3 o el backend no puede mantenerse congelado ante nuevas capacidades.

## Toolchain Freeze Custody

Los ejecutables del micro-toolchain en modo Release fueron congelados tras concluir `E1-003`:

- `target/release/irtool.exe`: `1eff80daf7136e89766ccc55a2050b5c389e6b01ed89a2fbc633894b6ac99668`
- `target/release/runner.exe`: `66700a4da713cb918ad722165a3b6bf26d3ebe57a7c483ee56f1ecb5460b9067`
- Custodiados en: `runs/toolchain-freeze-poc003.sha256` y tag git `poc-003-e1-freeze`.

## Evaluation and Answers to Core Questions

1. **¿Puede el agente interactuar con servicios externos mediante un Capability ABI declarativo e imports tipados?**
   Sí (`SC-01`, `SC-02`, `SC-12`, `SC-13`). El agente declaró en `ir/program-003.ir.json` las capabilities requeridas (`kv_store`, `audit_log`, `credit_service`), mapeó imports de host con firmas tipadas (`kv_get`, `emit_audit`, `check_credit`) y los invocó con el operador `call_import` de Semantic IR v0.3 sin alterar código en WAT/WASM ni requerir frameworks externos.

2. **¿Fueron las violaciones de autorización detectadas y diagnosticadas de forma determinista?**
   Sí (`SC-04`, `SC-05`, `SC-14`). Al retirar la capability `audit_log` en `E2-003`, el verificador emitió determinísticamente el error `E_UNDECLARED_CAPABILITY` con JSONPath exacto (`$.imports[1].capability`) en un payload de 343 bytes. El agente reparó la discrepancia en 1 solo intento recuperando la paridad binaria exacta (`637ee58c...`).

3. **¿Se mantuvo el backend del compilador y runner 100 % congelado ante nuevas capacidades de host y mantenimiento?**
   Sí (`SC-03`, `SC-07`, `SC-09`). A lo largo de `E2-003`, `E3-003` y `E4-003`, ni `irtool` ni `runner` requirieron un solo cambio en su código fuente. Los hashes SHA-256 de los binarios release en `runs/toolchain-freeze-poc003.sha256` se mantuvieron idénticos, demostrando la extensibilidad pura de la IR declarativa.

4. **¿Los efectos secundarios en el host fueron reproducibles y auditables?**
   Sí (`SC-02`, `SC-08`, `SC-10`). El runner en Wasmtime interceptó y registró todas las llamadas a funciones de host que producen efectos secundarios (`emit_audit`, `check_credit`), verificando contra las aserciones de `expected_host_calls` tanto el orden secuencial de llamada como los parámetros exactos (e.g. código de auditoría 999 para órdenes > $1000).

5. **¿Se conservó la reproducibilidad binaria exacta y el rendimiento del micro-toolchain?**
   Sí (`SC-11`, `SC-15`, `SC-16`). 3/3 builds independientes en `E1-003` y `E4-003` produjeron exactamente el mismo hash SHA-256 de WASM. La latencia combinada de verificación y compilación fue de tan solo 27 ms, y el runner ejecutó suites completas de 17 pruebas con mocks de host en 3 ms.

## Conclusion

Status: validated

Decision: GO

Evidence summary:
- E1-003: Construcción autónoma de módulo con imports declarativos (`kv_get`, `emit_audit`) y capabilities en 1 intento (632 bytes WASM, 6/6 públicas, 9/9 reservadas). Freeze sellado en tag `poc-003-e1-freeze`.
- E2-003: Violación de capability (`E_UNDECLARED_CAPABILITY`) rechazada con código de salida 1 y diagnóstico JSONPath de 343 bytes; reparada en 1 intento restaurando paridad bit a bit del binario.
- E3-003: Extensión funcional incorporando capability `credit_service` y función importada `check_credit` completada en 1 intento con 0 cambios al backend congelado (703 bytes WASM, 7/7 públicas, 10/10 reservadas, 59/59 casos en regresión total).
- E4-003: Mantenimiento modularizado (alícuota fiscal 12 % y evento de auditoría diferenciado 999) con diff conciso en subfunciones internas y reproducibilidad determinista 3/3 (`bf2021f6...`, 711 bytes WASM, 17/17 pruebas aprobadas).
- Success criteria passed: 16/16 (SC-01 a SC-16).
- Success criteria failed: 0/16.
- Inmutabilidad de Toolchain: 100 % confirmada en `runs/toolchain-freeze-poc003.sha256`.

Recommendation:
Decisión GO (validated). Habiendo demostrado que Semantic IR v0.3 permite a agentes de IA consumir servicios de host externos mediante un Capability ABI declarativo e imports WebAssembly Core tipados, con verificación estática de autorizaciones, auditoría determinista de efectos secundarios y backend 100 % congelado, se recomienda formalmente avanzar a la Fase 4: **POC 004 — WebAssembly Component Model, Interfaces WIT estandarizadas y Canonical ABI** (composición de componentes WASM interoperables, imports/exports de tipos complejos de alto nivel vía WIT, y empaquetado de componentes estándar para arquitecturas agent-native de microservicios).

