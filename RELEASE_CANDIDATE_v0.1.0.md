# Release Candidate v0.1.0 — Agent-Native

**Iniciativa:** `KADDO-AGENT-NATIVE-STABILIZATION-CLOSURE`  
**Versión Objetivo:** Agent-Native v0.1.0  
**Fecha de Publicación:** 2026-10-16  
**Decisión Final del Equipo:** **GO para extracción a repositorio independiente**  
**Repositorio Base:** `Trycatch-tv/poc-agent-native`  

---

## 1. Declaración de Conformidad y Alcance

Agent-Native v0.1.0 queda formalmente estabilizado y validado como una tecnología autónoma de construcción y evolución de software a partir de lenguaje natural y Work Items estructurados, sin intervención manual sobre la representación intermedia (`Semantic IR`) ni dependencia obligatoria del ecosistema Kaddo.

### Capacidades Demostradas y Verificadas (v0.1.0)
1. **Compilación Determinista Multi-Componente (`irtool`):**
   - Validación sintáctica y semántica estricta sobre contratos WIT (`WIT-0.1.0`).
   - Resolución WIT obligatoria sin fallbacks silenciosos a archivos predefinidos (`E_WIT_NOT_FOUND`, `E_WIT_PARSE_ERROR`, `E_WIT_WORLD_NOT_FOUND`).
   - Emisión determinista de binarios WebAssembly Component Model con Canonical ABI.
   - Diagnósticos estructurados con código de error y JSONPath exacto para auto-recuperación de agentes.

2. **Orquestación y Reconciliación Incremental (`orchestrator`):**
   - Ingesta de narrativas funcionales en Markdown (`spec/narrative-005.md`, `spec/narrative-resource-reservation.md`).
   - Ingesta y reconciliación acumulativa de Work Items (`spec/work-items-e2/`, `e3/`, `e4/`, `work-items-res/`).
   - Detección de colisiones de versión, dependencias rotas y diff multidimensional de requisitos.
   - Publicación transaccional con carpetas versionadas `versions/build-<timestamp>/manifest.json` y actualización atómica de `active.json`.
   - Inferencia LLM real desacoplada (`LlmImplementationAgent`) con adaptador OpenAI-compatible, medición de tokens observados y bucle de auto-reparación diagnóstica.

3. **Runtime Generalizado y Composición en Memoria (`runner`):**
   - Ejecución completa sobre `Wasmtime 27.0.0` Component Model.
   - Invocación y composición nativa entre componentes (`shipping-calculator` + `order-processor`, `quota-engine` + `reservation-coordinator`).
   - Mockeo configurable de estado del host mediante YAML (`kv`, `credit_limits`, `credit_ok`, `tenant_status`, `emit_audit`).
   - Verificación estricta de llamadas e interfaces importadas.

4. **Operación Standalone Validada:**
   - Creación de proyecto independiente mediante `orchestrator init --project-root <dir>`.
   - Generación de especificación, compilación y suite de pruebas ejecutadas de forma completamente desacoplada de Kaddo.

---

## 2. Manifiesto Criptográfico de Artefactos de Referencia

Todos los artefactos binarios generados por el compilador determinista en la versión v0.1.0 exhiben reproducibilidad bit-a-bit (100% SHA-256 match en builds repetidas):

| Componente | Dominio / Escenario | Tamaño (Bytes) | SHA-256 Digest |
|---|---|---|---|
| `shipping-calculator.wasm` | Comercial / Flete (Base E1/E2) | 674 B | `b9ae16245dc4b800a31d5362384a45188dabb765e119885b933fdb969c90979e` |
| `order-processor.wasm` | Comercial / Liquidación (Base E1/E2) | 2,488 B | `a075570a554a67251a66d24c2fbcd9c46569e4a88cd69d15230b3abdae9a8751` |
| `order-processor.wasm` | Comercial / Evolución E3 (IVA 18%) | 2,488 B | `25ae47ee79b232225ad8de23430b2f4ee5c766a81e73a19b5cbe0b617f274454` |
| `shipping-calculator.wasm` | Comercial / Evolución E4 (Tarifas 550c/850c) | 674 B | `31bba5071664f51e60e303e321d32dfa94a1854baf9e268895d05afa6f9b36ba` |
| `usage-tracker.wasm` | Facturación SaaS / Uso | 592 B | `731f726e10affbd6bfc0c5d9f3f2b188d60cf2390a1bf884df0c982c5d9ee5a1` |
| `subscription-billing.wasm` | Facturación SaaS / Facturación | 1,280 B | `b248eac924829eac3e1a15152bc0f4b3a185c1b06271b36bb8f359b87e96ef5b` |
| `quota-engine.wasm` | Infraestructura / Cuotas v1 (Límite 1000) | 741 B | `cf3c3b0df3f9154f8a37e1a7ba89ea730efd9516668e0d4bb8bb1f26a798ee3e` |
| `reservation-coordinator.wasm` | Infraestructura / Reserva v1 | 2,216 B | `fa1d7ac207038e129ae35c755c3c0dfb6c6b4455ae0dfaa31f13b63b27b5006d` |
| `quota-engine.wasm` | Infraestructura / Cuotas v2 (Límite 2500) | 741 B | `70fa6e129aa18055bf09b0b4a68ebae71f54cf796c342f567b45cc16a192ee71` |
| `reservation-coordinator.wasm` | Infraestructura / Reserva v2 (+50 penalidad) | 2,234 B | `44ce415fa9fbe65961314d3527b137f86641e406208bb78bc2b7747e92383c27` |

---

## 3. Cobertura de Pruebas y Aprobación de Conformidad

La suite de validación automatizada comprende **20 pruebas de integración y unittests** distribuidas a lo largo del workspace:
- **`irtool`:** 8/8 passed (verificación sintáctica, tipado estricto, detección de violaciones WIT, compilación a Component Model).
- **`orchestrator`:** 6/6 passed (ingesta de narrativas desconocidas, rechazo de narrativas incompatibles sin invención de componentes, reconciliación acumulativa, bucle de auto-reparación diagnóstica).
- **`runner`:** 6/6 passed (operadores v0.1, records v0.2, imports/audits v0.3, Component Model v0.4, composición comercial v0.5, composición de reservas v0.5).
- **Prueba Standalone:** 9/9 casos de aceptación ejecutados con éxito en `runs/standalone-validation/`.

---

## 4. Guía de Extracción hacia Nuevo Repositorio

Para extraer Agent-Native a un repositorio independiente (`agent-native-core`):

1. **Estructura a migrar:**
   - `crates/irtool/` -> Motor de Semantic IR y compilador WebAssembly.
   - `crates/orchestrator/` -> Motor de consolidación de System Specification y agentes de implementación.
   - `crates/runner/` -> Runtime determinista sobre Wasmtime Component Model.
   - `spec/` -> Esquemas JSON de Semantic IR v0.5 y System Spec v1.0.0.
   - `wit/` -> Contratos de interfaces WIT fundamentales.
   - `BENCHMARK_REPORT_005.md` -> Evidencia empírica de rendimiento y aislamiento.
   - `RELEASE_CANDIDATE_v0.1.0.md` -> Declaración de release y trazabilidad de hashes.

2. **Dependencias requeridas:**
   - Rust 1.80+ (edición 2021).
   - Wasmtime 27.0.0 (Component Model activado).
   - `wit-component 0.221.0` y `wasm-encoder 0.221.0`.

3. **Comando de inicialización de nuevo proyecto en downstream:**
   ```bash
   orchestrator init --project-root ./my-agent-native-app
   orchestrator spec --narrative ./my-agent-native-app/spec/narrative.md --output ./my-agent-native-app/spec/system-spec.json
   orchestrator build --spec ./my-agent-native-app/spec/system-spec.json --out-dir ./my-agent-native-app/dist
   runner test --wasm ./my-agent-native-app/dist/app.wasm --cases ./my-agent-native-app/tests/acceptance.yaml --output ./my-agent-native-app/dist/results.json
   ```

---

## 5. Decisión de Arquitectura: GO

Con la finalización de los Work Items **WI-S09, WI-S10, WI-S11 y WI-S12**, se declara **cerrada la POC 005 y concluida la fase de estabilización**.

Agent-Native v0.1.0 cumple con todos los criterios de suficiencia técnica, determinismo y desacople funcional para su evolución como proyecto y repositorio independiente.
