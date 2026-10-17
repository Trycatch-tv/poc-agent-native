# Release Candidate v0.1.0 — Agent-Native

**Iniciativa:** `KADDO-AGENT-NATIVE-FINAL-CLOSURE` (`WI-S17`)  
**Versión Objetivo:** Agent-Native v0.1.0  
**Fecha de Publicación:** 2026-10-16  
**Decisión Final del Equipo:** **GO para extracción a repositorio independiente**  
**Repositorio Base:** `Trycatch-tv/poc-agent-native`  

---

## 1. Declaración de Conformidad y Alcance Validado (v0.1.0)

Agent-Native v0.1.0 queda formalmente validado como un motor standalone de generación, verificación determinista, compilación a WebAssembly Component Model, ejecución dinámica y evolución acumulativa de componentes dentro del alcance de **Semantic IR v0.5** (`i32`, `bool`, `Record`, aritmética/comparación, condicionales, capacidades de host `kv_get`/`emit_audit`/`check_credit`/`tenant_status` y composición WIT `component_call`).

### Garantías Técnicas Cerradas en WI-S17

1. **Ajuste A — Propagación Estricta de Fallas de Dependencias en Runtime (`crates/runner/src/main.rs`):**
   - Se eliminaron todos los `return 0` y `.unwrap_or(0)` en la resolución e invocación de componentes enlazados.
   - Toda falla técnica de dependencia propaga un error estructurado `[DependencyError] component=<iface> operation=<func> kind=<missing_dependency|invalid_binary|instantiation_failed|execution_trap>: <detalle>`, produciendo `actual: None` y `passed: false`, sin confundirse jamás con un rechazo de negocio válido (`0`).
   - Verificado automáticamente en `tests::test_dependency_failures_never_fallback_to_zero`.

2. **Ajuste B — Promoción Atómica de Versiones y Fuente Única de Verdad (`crates/orchestrator/src/lib.rs`):**
   - Se eliminó el patrón `remove_file` + `rename` sobre `active.json`.
   - `atomic_replace_file()` utiliza `MoveFileExW` con `MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH` en Windows y `std::fs::rename` atómico en Unix/POSIX.
   - Se eliminaron las copias de conveniencia en la raíz de `out_dir`: `active.json` apunta exclusivamente a `versions/<version_id>/manifest.json` y a los artefactos inmutables `versions/<version_id>/<component>.wasm` y `<component>.ir.json`.
   - `recover_and_verify_dist_state()` limpia archivos temporales huérfanos (`.staging`, `.active.json.tmp`) y verifica la integridad SHA-256 de la versión activa.
   - Verificado ante fallos inyectados en copia, manifiesto y promoción en `tests::test_atomic_promotion_and_partial_failure_preservation`.

3. **Ajuste C — Generación y Ejecución E2E de un Dominio Desconocido con LLM Real (`runs/unknown-domain-e2e/`):**
   - El runtime (`crates/runner/src/main.rs`) fue refactorizado para ejecutar cualquier componente WebAssembly dinámicamente mediante introspección de `wasmtime::component::Component` (eliminando todos los bloques `bindgen!` específicos de dominio).
   - Se creó el escenario **Control de Esclusas e Hidrología de Canal** (`hydro-sensor` + `lock-controller` en paquete WIT `kaddo:canal@0.1.0`), ausente de `SynthesizerAgent` y de `runner`.
   - `orchestrator build --spec ... --out-dir ... --agent llm` invocó inferencia real con `claude-sonnet-4-6` (`provider = "claude-cli"`), generando Semantic IR v0.5 válida en el primer intento (`attempts = 1`), verificada y compilada por `irtool` y ejecutada por `runner test` con **5/5 pruebas aprobadas en v1** y **5/5 pruebas aprobadas en v2** tras aplicar el Work Item evolutivo `WI-LOCK-01`.

4. **Ajuste D — Higiene de Credenciales, Telemetría Exacta y Evidencia Persistida (`crates/orchestrator/src/agent.rs`):**
   - Se eliminó la creación de archivos temporales `curlllm_*.cfg` en disco; las cabeceras HTTP se transmiten exclusivamente por `stdin` (`curl --config -`) en memoria.
   - `AgentExecutionMetrics` representa `prompt_tokens`, `completion_tokens` y `total_tokens` como `Option<usize>` junto con `tokens_reported: bool`, sin estimar jamás tokens mediante `prompt.len() / 4` (verificado en `agent::tests::test_unreported_tokens_are_none_never_estimated`).

---

## 2. Evidencia Persistida del Dominio Desconocido con LLM Real (`runs/unknown-domain-e2e/`)

| Versión | Componente | Proveedor / Modelo | Latencia (`ms`) | `prompt_tokens` | `completion_tokens` | `total_tokens` | Bytes WASM | SHA-256 `.wasm` | SHA-256 `.ir.json` |
|---|---|---|---|---|---|---|---|---|---|
| `build-1792206976764513300` (v1 Base) | `hydro-sensor` | `claude-cli` / `claude-sonnet-4-6` | `41,095` | `17,347` | `3,388` | `20,735` | 611 B | `af44b40b0b4fa3754fbeabaf44e6633307f910fa2ed43bae0381004e6ae975af` | `a3ab9191377e7968270540752933ff2464ae27b43ef6c64181908033e7f2dafa` |
| `build-1792206976764513300` (v1 Base) | `lock-controller` | `claude-cli` / `claude-sonnet-4-6` | `128,149` | `17,903` | `10,244` | `28,147` | 1,613 B | `2e301bf2c836f9c023cc4c89199e4be1be1a20a0302cc0a3d992c063e1af9f3a` | `663eb98b16350b2867fe0125f402b7665c419d390996b37353a233d3df835a0e` |
| `build-1792207389433634100` (v2 `WI-LOCK-01`) | `hydro-sensor` | `claude-cli` / `claude-sonnet-4-6` | `39,538` | `17,401` | `2,959` | `20,360` | 605 B | `4ae71032e4cb83385721ce25848f906230e5fbcb74a2af55787f3cbcb3873641` | `58315f1fc2783220d83722b22b61dcb7315a2a37b35167e40ac7a8728be7904c` |
| `build-1792207389433634100` (v2 `WI-LOCK-01`) | `lock-controller` | `claude-cli` / `claude-sonnet-4-6` | `93,999` | `17,903` | `7,859` | `25,762` | 1,618 B | `d4fb2fc0e20837f9b637cbbe2a2a0e3a3f88405bd768b2add44b77ddb29b6f37` | `b130a61a9c07f97006b979e8ecb26bfe49558c09fbc23d42e3311614b838f528` |

Artefactos persistidos en el repositorio:
- Especificación y evolución: `runs/unknown-domain-e2e/spec/narrative-canal-lock.md`, `runs/unknown-domain-e2e/spec/work-items/WI-LOCK-01-hydro-damping.md`, `system-spec-v1.json`, `system-spec-v2.json`, `diff-v1-v2.json`.
- Versiones inmutables y puntero atómico: `runs/unknown-domain-e2e/dist/active.json`, `runs/unknown-domain-e2e/dist/versions/build-1792206976764513300/manifest.json`, `runs/unknown-domain-e2e/dist/versions/build-1792207389433634100/manifest.json`.
- Reportes de aceptación ejecutados: `runs/unknown-domain-e2e/results/acceptance-v1-report.json` (5/5 passed) y `runs/unknown-domain-e2e/results/acceptance-v2-report.json` (5/5 passed).

---

## 3. Cobertura de Pruebas Automatizadas del Workspace

La suite de validación automatizada comprende **26 pruebas unitarias y de integración** en el workspace (`cargo test --workspace`):
- **`irtool` (8/8 passed):** verificación sintáctica y semántica, contratos WIT, compilación determinista a Component Model.
- **`orchestrator` (9/9 passed):** interpretación de narrativas no vistas, rechazo de narrativas incompatibles, reconciliación acumulativa, bucle de recuperación diagnóstica, modo LLM estricto sin API key, prohibición de estimación de tokens (`test_unreported_tokens_are_none_never_estimated`), promoción atómica y preservación de `active.json` ante fallas parciales de copia, manifiesto y promoción (`test_atomic_promotion_and_partial_failure_preservation`).
- **`runner` (9/9 passed):** ejecución dinámica sobre Wasmtime Component Model, propagación estricta de errores de dependencias sin fallback a cero (`test_dependency_failures_never_fallback_to_zero`), evolución acumulativa v1 -> v2 -> v3 de reservas (`test_cumulative_reservation_evolution_v1_v2_v3`), y validación integral v1 y v2 del dominio desconocido generado por LLM (`test_unseen_llm_domain_canal_lock_v1_and_v2`).

---

## 4. Comandos de Reproducción

```bash
# 1. Ejecutar toda la suite de pruebas del workspace (26 tests)
cargo test --workspace

# 2. Reproducir generación de especificación y construcción con LLM real sobre el dominio desconocido
cargo run --bin orchestrator -- spec --narrative runs/unknown-domain-e2e/spec/narrative-canal-lock.md --output runs/unknown-domain-e2e/spec/system-spec-v1.json
cargo run --bin orchestrator -- build --spec runs/unknown-domain-e2e/spec/system-spec-v1.json --out-dir runs/unknown-domain-e2e/dist --agent llm

# 3. Ejecutar pruebas de aceptación sobre los componentes publicados
cargo run --bin runner -- test \
  --wasm runs/unknown-domain-e2e/dist/versions/build-1792206976764513300/lock-controller.wasm \
  --link runs/unknown-domain-e2e/dist/versions/build-1792206976764513300/hydro-sensor.wasm \
  --cases runs/unknown-domain-e2e/tests/acceptance-v1.yaml \
  --output runs/unknown-domain-e2e/results/acceptance-v1-report.json
```
