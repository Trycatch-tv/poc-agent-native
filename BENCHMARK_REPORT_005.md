# Benchmark Report 005 — Agent-Native Orchestrator vs Desarrollo Tradicional Asistido por LLM

> **Iniciativa:** `KADDO-AGENT-NATIVE-FINAL-CLOSURE` (`WI-041` + Remediación Final `WI-S17`)
>
> **Entorno:** `Wasmtime 27.0.0`, Windows x86_64, `rustc 1.93.0-nightly`, `wit-component 0.221.0`
>
> **Sujeto de prueba:** Ciclo de vida completo: desde especificación narrativa en lenguaje natural (`spec/narrative-005.md`, `spec/narrative-resource-reservation.md`, `runs/unknown-domain-e2e/spec/narrative-canal-lock.md`) y Work Items incrementales hasta composición multi-componente WebAssembly nativa con Canonical ABI y ejecución dinámica por introspección WIT.

---

## 1. Motivación y Distinción Explícita de Modos de Ejecución

Este reporte distingue de manera estricta y auditable los dos modos de operación de Agent-Native v0.1.0:

1. **Modo Determinista Offline (`--agent synthesizer` / `SynthesizerAgent`):**
   - Opera exclusivamente sobre dominios de referencia incluidos en el sintetizador determinista (`shipping-calculator`, `order-processor`, `usage-tracker`, `subscription-billing`, `quota-engine`, `reservation-coordinator`).
   - No realiza llamadas de red ni inferencia LLM; mide el costo puro de interpretación de especificación, verificación estática (`irtool::verify_ir`) y compilación determinista a WebAssembly Component Model (`irtool::compile_ir_to_wasm`).
   - Rechaza explícitamente cualquier componente de dominio desconocido sin usar plantillas genéricas falsas.

2. **Modo de Inferencia LLM Real (`--agent llm` / `LlmImplementationAgent`):**
   - Opera sin fallback al sintetizador determinista y sin escribir credenciales en archivos temporales en disco (transmisión exclusiva por `stdin` en memoria).
   - Fue validado de extremo a extremo sobre un **dominio completamente desconocido** (`runs/unknown-domain-e2e/`: **Control de Esclusas e Hidrología de Canal**, componentes `hydro-sensor` y `lock-controller`), ausente tanto de `SynthesizerAgent` como de `runner`.
   - Registra telemetría real reportada por el proveedor (`tokens_reported: true`, `Option<usize>` para `prompt_tokens`, `completion_tokens` y `total_tokens`), prohibiendo cualquier estimación heurística (`prompt.len() / 4`).

---

## 2. Comparativa Separada: Modo Determinista Offline vs Modo LLM Real

### 2.1. Métricas del Modo Determinista Offline (`--agent synthesizer`)

| Dimensión de Medición | Valor Observado (`SynthesizerAgent` + `irtool` + `runner`) | Alcance |
|---|---|---|
| **Interpretación + Síntesis AST (`orchestrator spec + build`)** | **`140.0 ms`** | Dominios de referencia pre-registrados (sin red) |
| **Compilación Determinista Multi-Componente (`irtool`)** | **`22.4 ms`** (2 componentes `.wasm` con Canonical ABI) | 100 % determinista y reproducible bit a bit |
| **Tamaño Binario Total (2 Componentes)** | **`2,223 B – 3,162 B`** (`605 B` a `2,488 B` por componente) | Componentes WASM estándar sin runtime pesado |
| **Tokens Consumidos** | **`None` (`tokens_reported: false`)** | No invoca proveedor de IA |
| **Reproducibilidad Binaria SHA-256** | **`3/3 builds idénticos (100 %)`** | Determinismo criptográfico |

### 2.2. Métricas Reales Observadas del Modo LLM (`--agent llm` en Dominio Desconocido `runs/unknown-domain-e2e/`)

Corrida ejecutada con `provider = "claude-cli"`, `model = "claude-sonnet-4-6"`, sin ramas de dominio en `SynthesizerAgent` ni en `runner`:

| Hito / Versión | Componente | Proveedor / Modelo | Intentos (`attempts`) | Recuperado de Diagnósticos | Latencia Real (`latency_ms`) | `prompt_tokens` | `completion_tokens` | `total_tokens` | `tokens_reported` |
|---|---|---|---|---|---|---|---|---|---|
| **v1 Base (`build-1792206976764513300`)** | `hydro-sensor` | `claude-cli` / `claude-sonnet-4-6` | `1` | `false` | **`41,095 ms`** | `17,347` | `3,388` | `20,735` | `true` |
| **v1 Base (`build-1792206976764513300`)** | `lock-controller` | `claude-cli` / `claude-sonnet-4-6` | `1` | `false` | **`128,149 ms`** | `17,903` | `10,244` | `28,147` | `true` |
| **v2 Evolución `WI-LOCK-01` (`build-1792207389433634100`)** | `hydro-sensor` | `claude-cli` / `claude-sonnet-4-6` | `1` | `false` | **`39,538 ms`** | `17,401` | `2,959` | `20,360` | `true` |
| **v2 Evolución `WI-LOCK-01` (`build-1792207389433634100`)** | `lock-controller` | `claude-cli` / `claude-sonnet-4-6` | `1` | `false` | **`93,999 ms`** | `17,903` | `7,859` | `25,762` | `true` |

- **Ejecución de pruebas de aceptación sobre binarios generados por LLM (`runner test` dinámico):**
  - **v1 (`acceptance-v1.yaml`):** **5/5 casos aprobados (0 fallidos)** en **`70 ms`** (`runs/unknown-domain-e2e/results/acceptance-v1-report.json`).
  - **v2 (`acceptance-v2.yaml` tras `WI-LOCK-01`):** **5/5 casos aprobados (0 fallidos)** en **`38 ms`** (`runs/unknown-domain-e2e/results/acceptance-v2-report.json`).

---

## 3. Matriz Criptográfica de Artefactos Verificados

### 3.1. Artefactos Generados por Inferencia LLM Real (`--agent llm`, Dominio Desconocido)

| Versión Publicada | Componente | Tamaño WASM | SHA-256 `.wasm` | SHA-256 `.ir.json` |
|---|---|---|---|---|
| `build-1792206976764513300` (v1 Base) | `hydro-sensor` | 611 B | `af44b40b0b4fa3754fbeabaf44e6633307f910fa2ed43bae0381004e6ae975af` | `a3ab9191377e7968270540752933ff2464ae27b43ef6c64181908033e7f2dafa` |
| `build-1792206976764513300` (v1 Base) | `lock-controller` | 1,613 B | `2e301bf2c836f9c023cc4c89199e4be1be1a20a0302cc0a3d992c063e1af9f3a` | `663eb98b16350b2867fe0125f402b7665c419d390996b37353a233d3df835a0e` |
| `build-1792207389433634100` (v2 `WI-LOCK-01`) | `hydro-sensor` | 605 B | `4ae71032e4cb83385721ce25848f906230e5fbcb74a2af55787f3cbcb3873641` | `58315f1fc2783220d83722b22b61dcb7315a2a37b35167e40ac7a8728be7904c` |
| `build-1792207389433634100` (v2 `WI-LOCK-01`) | `lock-controller` | 1,618 B | `d4fb2fc0e20837f9b637cbbe2a2a0e3a3f88405bd768b2add44b77ddb29b6f37` | `b130a61a9c07f97006b979e8ecb26bfe49558c09fbc23d42e3311614b838f528` |

### 3.2. Artefactos de Suites de Conformidad Deterministas (`--agent synthesizer`)

| Hito Experimental | Componente | Tamaño (Bytes) | SHA-256 Observado | Estado de Aislamiento |
|---|---|---|---|---|
| **E1 / E2 (Comercial Base)** | `shipping-calculator.wasm` | 674 B | `b9ae16245dc4b800a31d5362384a45188dabb765e119885b933fdb969c90979e` | Freeze inicial |
| **E1 / E2 (Comercial Base)** | `order-processor.wasm` | 2,488 B | `a075570a554a67251a66d24c2fbcd9c46569e4a88cd69d15230b3abdae9a8751` | Freeze inicial (IVA 15%) |
| **E3 (Evolución IVA)** | `order-processor.wasm` | 2,488 B | `25ae47ee79b232225ad8de23430b2f4ee5c766a81e73a19b5cbe0b617f274454` | Actualizado (IVA 18%) |
| **E4 (Evolución Flete)** | `shipping-calculator.wasm` | 674 B | `31bba5071664f51e60e303e321d32dfa94a1854baf9e268895d05afa6f9b36ba` | Actualizado (550c/850c) |
| **SaaS** | `usage-tracker.wasm` | 592 B | `731f726e10affbd6bfc0c5d9f3f2b188d60cf2390a1bf884df0c982c5d9ee5a1` | Sintetizado determinista |
| **SaaS** | `subscription-billing.wasm` | 1,280 B | `b248eac924829eac3e1a15152bc0f4b3a185c1b06271b36bb8f359b87e96ef5b` | Sintetizado determinista |
| **Reservas v1** | `quota-engine.wasm` | 741 B | `cf3c3b0df3f9154f8a37e1a7ba89ea730efd9516668e0d4bb8bb1f26a798ee3e` | Inicial (límite 1000) |
| **Reservas v1** | `reservation-coordinator.wasm` | 2,216 B | `fa1d7ac207038e129ae35c755c3c0dfb6c6b4455ae0dfaa31f13b63b27b5006d` | Inicial (sin penalización) |
| **WI-RES-01 (v2)** | `quota-engine.wasm` | 741 B | `70fa6e129aa18055bf09b0b4a68ebae71f54cf796c342f567b45cc16a192ee71` | Actualizado (límite 2500) |
| **WI-RES-02 (v3)** | `reservation-coordinator.wasm` | 2,234 B | `44ce415fa9fbe65961314d3527b137f86641e406208bb78bc2b7747e92383c27` | Actualizado (+50 penalidad) |

---

## 4. Garantías de Hardening Verificadas en WI-S17

1. **Ausencia Total de Fallbacks Silenciosos en Runtime (`crates/runner/src/main.rs`):**
   - El runner ejecuta cualquier componente WebAssembly dinámicamente mediante introspección de tipos de `wasmtime::component::Component` (sin `bindgen!` por dominio).
   - Se eliminaron todos los `return 0` y `unwrap_or(0)` ante fallos de dependencias. Cualquier dependencia ausente (`missing_dependency`), binario corrupto (`invalid_binary`), discrepancia de contrato (`instantiation_failed`) o trap de ejecución (`execution_trap`) aborta la invocación y reporta un error técnico estructurado `[DependencyError]` con `actual: None` y `passed: false`.
2. **Promoción Atómica y Fuente Única de Verdad (`crates/orchestrator/src/lib.rs`):**
   - `active.json` se reemplaza atómicamente mediante `MoveFileExW(..., MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)` en Windows y `rename` en POSIX, sin eliminar previamente `active.json`.
   - Se eliminaron las copias de conveniencia en la raíz de `out_dir`; todos los artefactos residen inmutablemente en `versions/<version_id>/`.
3. **Higiene de Credenciales y Telemetría Exacta (`crates/orchestrator/src/agent.rs`):**
   - Ninguna credencial ni token se escribe jamás en archivos temporales en disco ni en argumentos de proceso.
   - Los tokens no reportados por un proveedor se serializan como `null` (`None`) con `tokens_reported: false`, sin estimaciones heurísticas.
