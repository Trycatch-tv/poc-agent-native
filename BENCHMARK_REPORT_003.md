# Benchmark Report 003 — Agent-Native IR v0.3 vs Baseline Rust (`wasm32-unknown-unknown`) con Host Imports

> **Experimento:** `KADDO-AGENT-NATIVE-POC-003` (Work Item `WI-024`)
>
> **Entorno:** `rustc 1.93.0-nightly`, `Wasmtime 27.0.0`, Windows x86_64
>
> **Sujeto de prueba:** Motor de cálculo comercial con Capability ABI (KV Store, Audit Log y Credit Service) y efectos secundarios auditables.

---

## 1. Motivación y Metodología

La POC 003 demostró que una **Semantic IR v0.3** permite a un agente interactuar con servicios externos mediante un Capability ABI declarativo e imports tipados de WebAssembly Core con backend completamente congelado.

Este benchmark pareado (**`WI-024`**) evalúa empíricamente la misma especificación funcional (`spec/task-003.yaml` v3) implementada en dos rutas tecnológicas independientes:
1. **Ruta Agent-Native:** `ir/program-003.ir.json` procesada por el micro-toolchain determinista (`irtool verify` + `irtool compile` directo a WASM con `ImportSection`).
2. **Ruta Baseline Rust:** `crates/baseline-rust-003/src/lib.rs` (`#![no_std]`, `crate-type = ["cdylib"]`, `opt-level = "z"`, `lto = true`, `strip = "symbols"`, imports vía `#[link(wasm_import_module = "host")] extern "C"`).

Ambas rutas son evaluadas por el mismo ejecutor en runtime (`target/release/runner.exe` sobre Wasmtime 27.0.0) con las mismas suites de pruebas públicas (`tests/public-003.yaml`) y reservadas (`tests/hidden-003.yaml`), validando tanto los valores de retorno del registro `OrderResult` en memoria lineal como la secuencia exacta de llamadas al host mock (`expected_host_calls`).

---

## 2. Resultados Cuantitativos Pareados

| Dimensión de Medición | Semantic IR v0.3 (Agent-Native) | Baseline Rust (`wasm32-unknown-unknown`) | Ventaja Agent-Native |
|---|---|---|---|
| **Verificación (`verify` vs `cargo check`)** | **`11 ms`** | `183 ms` | **`16.6x más rápido`** |
| **Compilación (`compile` vs `cargo build`)** | **`16 ms`** | `262 ms` (`1.78 s` clean) | **`16.3x más rápido`** |
| **Ciclo Total Toolchain (`verify + compile`)** | **`27 ms`** | `445 ms` | **`16.5x más rápido`** |
| **Tamaño Binario WebAssembly (`.wasm`)** | `711 bytes` | `552 bytes` | Ambos ultra-compactos (< 800 B) |
| **Volumen Diagnóstico en Falla** | **`343 bytes`** (JSONPath exacto) | `8,642 bytes` (`cargo check --message-format=json`) | **`25.1x más compacto (96.0 % menos ruido)`** |
| **Pass Rate Pruebas Públicas (`7/7`)** | `7/7 (100 %)` | `7/7 (100 %)` | **Empate (100 %)** |
| **Pass Rate Pruebas Reservadas (`10/10`)** | `10/10 (100 %)` | `10/10 (100 %)` | **Empate (100 %)** |
| **Auditoría de Efectos Secundarios (`expected_host_calls`)** | `100 % aprobados` | `100 % aprobados` | **Empate (100 %)** |
| **Reproducibilidad Binaria SHA-256** | `3/3 builds idénticos` | `3/3 builds idénticos` | **Empate (100 % determinismo)** |

---

## 3. Análisis de Trade-Offs de Ingeniería

### A. Latencia del Bucle de Feedback para el Agente
En cada ciclo de interacción, el agente en la ruta Agent-Native recibe feedback estático y emite el artefacto WebAssembly en **27 milisegundos**. En la ruta de Rust tradicional con imports `extern "C"`, el ciclo requiere **445 milisegundos** en compilación incremental y **~1.8 segundos** en compilaciones frías. Esta diferencia de más de un orden de magnitud permite bucles de reparación y auto-ajuste autónomos casi instantáneos sin penalizar la factura de inferencia ni bloquear el pipeline.

### B. Densidad y Precisión Diagnóstica
Ante una discrepancia en la firma o autorización de una llamada de importación, `rustc` emite **8,642 bytes** de salida JSON conteniendo trazas de sugerencias de tipos y advertencias de inicialización. En contraste, `irtool verify` emite **343 bytes** estructurados con JSONPath inequívoco (`$.imports[1].capability`), reduciendo drásticamente el consumo de tokens en la ventana de contexto del LLM y eliminando alucinaciones inducidas por el exceso de ruido sintáctico.

### C. Autorización Estática vs Fallo de Link en Runtime
- En Rust, el compilador permite declarar cualquier bloque `extern "C"` arbitrario sin verificar si el host concede o restringe dicho permiso. La ausencia de la función de importación sólo se manifiesta tardíamente en tiempo de ejecución al intentar instanciar el módulo en el runtime (fallo de linking).
- En Semantic IR v0.3, el Capability ABI impone una autorización estática formal en el verificador: si el módulo invoca una función importada sin haber declarado la capability correspondiente en la raíz (`capabilities`), el código es rechazado estáticamente en 11 ms con `E_UNDECLARED_CAPABILITY`, antes de generar un solo byte de WebAssembly.

### D. Seguridad de Memoria y Modelo de Ejecución
- En Rust `#![no_std]`, el agente se ve obligado a emplear bloques `unsafe`, punteros crudos (`&raw const`), buffers mutables estáticos (`static mut RESULT`) y lidiar con directivas de representación C (`#[repr(C)]`).
- En Semantic IR v0.3, toda la asignación en memoria lineal (alineación a 4 bytes mediante el puntero global `$heap_ptr = 1024`) y el pasaje de parámetros se resuelve determinísticamente en el compilador, liberando al agente de la carga de gestionar memoria de bajo nivel.

---

## 4. Conclusión

El benchmark de la Fase 3 confirma de manera concluyente que:
1. **La ventaja de latencia de toolchain se mantiene en `16.5x`**, sosteniéndose por debajo de los 30 ms incluso con múltiples imports y composiciones funcionales.
2. **La compacidad diagnóstica alcanza un factor de `25.1x`** (96 % de reducción de bytes de error frente al compilador tradicional de Rust).
3. **El Capability ABI aporta una garantía de seguridad estática superior**, rechazando invocaciones no autorizadas en fase de verificación y facilitando pruebas deterministas de efectos secundarios con host mocks en Wasmtime.
