# Benchmark Comparativo Pareado (`WI-008`): Agent-Native (`IR v0.1 → WASM`) vs Baseline (`Rust → wasm32-unknown-unknown`)

## 1. Diseño del experimento pareado
Bajo las mismas especificaciones (`spec/task.yaml` v1, v2, v3), las mismas suites de pruebas públicas y reservadas, el mismo validador WASM (`irtool validate` / `wasmparser`) y el mismo runtime (`runner test` sobre `Wasmtime 27.0.0`), se ejecutaron los cuatro ejercicios en la ruta tradicional (`crates/baseline-rust/src/lib.rs` compilado con `rustc 1.85.0 --target wasm32-unknown-unknown -C opt-level=z -C lto=true -C strip=symbols`) para compararlos frente a la ruta Agent-Native (`ir/program.ir.json` compilado con `irtool`).

## 2. Tabla comparativa consolidada (`E1`–`E4` vs `B1`–`B4`)

| Ejercicio / Par | Métrica | Ruta Agent-Native (`IR v0.1 → WASM`) | Baseline Rust (`Rust → wasm32-unknown-unknown`) | Diferencia / Ratio observado |
|---|---|---:|---:|---|
| **Construcción (`E1` vs `B1`)** | Verificación (`verify_ms` vs `cargo check`) | **`12 ms`** | `365 ms` | **`30.4x` más rápido** en Agent-Native |
| | Compilación (`compile_ms` vs `cargo build --release`) | **`15 ms`** | `959 ms` | **`63.9x` más rápido** en Agent-Native |
| | Ciclo total determinista (`verify + compile`) | **`27 ms`** | `1324 ms` | **`49.0x` más rápido** en Agent-Native |
| | Tamaño del binario `.wasm` | **`76 bytes`** | `138 bytes` | **`44.9 %` más compacto** en Agent-Native |
| | Pruebas públicas / reservadas | `6/6` / `9/9` (`100 %`) | `6/6` / `9/9` (`100 %`) | Empate (`100 %`) |
| **Reparación (`E2` vs `B2`)** | Tiempo de rechazo del defecto sembrado | **`11 ms`** | `181 ms` | **`16.5x` más rápido** en Agent-Native |
| | Tamaño del payload de diagnóstico JSON | **`264 bytes`** (`1` objeto JSON estructurado) | `6158 bytes` (stream JSONL de `cargo check`) | **`23.3x` menos ruido de contexto** en Agent-Native |
| | Ubicación del error | `$.functions[0].body.condition.left` (`expected`/`found`) | `src/lib.rs:11:8..11:16` (`E0425` + texto ANSI) | Ruta lógica AST directa vs span línea/columna |
| | Intentos de reparación (`repair_attempts`) | `1` | `1` | Empate (`1` intento) |
| | Ciclo `verify + compile` tras reparar | **`25 ms`** (`11 + 14 ms`) | `401 ms` (`138 + 263 ms`) | **`16.0x` más rápido** en Agent-Native |
| **Extensión (`E3` vs `B3`)** | Ciclo `verify + compile` | **`25 ms`** (`11 + 14 ms`) | `550 ms` (`237 + 313 ms`) | **`22.0x` más rápido** en Agent-Native |
| | Tamaño del binario `.wasm` (2 funciones) | **`134 bytes`** | `205 bytes` | **`34.6 %` más compacto** en Agent-Native |
| | Pruebas públicas / reservadas | `12/12` / `18/18` (`100 %`) | `12/12` / `18/18` (`100 %`) | Empate (`100 %`) |
| **Mantenimiento (`E4` vs `B4`)** | Ciclo `verify + compile` incremental / clean | **`25 ms`** (`11 + 14 ms`) | `508 ms` inc. / `584 ms` clean (`188 + 396 ms`) | **`20.3x`–`23.4x` más rápido** en Agent-Native |
| | Reproducibilidad binaria (`3/3` builds) | `3/3` (`947620fe...`) | `3/3` (`e840e0d1...`) | Empate (`3/3` idénticos) |
| | Líneas modificadas en el diff de mantenimiento | `3` líneas (`spec_version`, `4000`, `600`) | `3` líneas (comentario doc, `4000`, `600`) | Empate en localidad (`3` líneas) |
| | Tiempo de ejecución de suite en Wasmtime (`test_ms`) | `16–28 ms` | `7–15 ms` | Ambos `< 30 ms` (dominado por instanciación de proceso) |

## 3. Hallazgos cualitativos y arquitectónicos del benchmark

1. **Pureza del módulo `.wasm` emitido (`76 B / 134 B` vs `138 B / 205 B`):**
   - `irtool compile` emite exclusivamente las secciones `Type`, `Function`, `Export` y `Code` necesarias para las funciones declaradas.
   - Incluso con `#![no_std]`, `opt-level = "z"`, `lto = true` y `strip = "symbols"`, `rustc` (`wasm32-unknown-unknown`) inyecta una memoria lineal de 16 páginas (`(memory (;0;) 16)` = 1 MiB) y tres globales exportados (`memory`, `__data_end`, `__heap_base`), aumentando el tamaño binario en `+81 %` (`138 B` vs `76 B`) en `E1`/`B1` y `+53 %` (`205 B` vs `134 B`) en `E3`/`B3` y `E4`/`B4`.
2. **Densidad y accionabilidad de diagnósticos para el agente (`264 B` vs `6158 B`):**
   - Ante el mismo error semántico (usar `subtotal` en vez de `subtotal_cents`), `irtool verify` produce un único objeto JSON de `264 bytes` con la ruta exacta en el AST (`$.functions[0].body.condition.left`), el parámetro esperado (`"subtotal_cents"`) y el encontrado (`"subtotal"`).
   - `cargo check --message-format=json` emite `6158 bytes` (`23.3x` más tokens de contexto) incluyendo metadatos de paquetes, spans de caracteres, sugerencias textuales y bloques ANSI renderizados.
3. **Eliminación de fricción de ABI y boilerplate de compilador:**
   - En Rust, exponer funciones puras a WebAssembly Core sin `wasm-bindgen` obliga al agente a gestionar detalles de bajo nivel ajenos al dominio: `#![no_std]`, `#[panic_handler]`, `#[no_mangle]`, `pub extern "C"`, y a traducir manualmente `bool` a `i32` (`premium: i32`, `premium == 0`) en la firma de `calculate_discount`.
   - En la Semantic IR v0.1, el agente declara `"type": "bool"`, el verifier garantiza tipado booleano estricto (`E_TYPE_MISMATCH`, `E_IF_CONDITION`) y verifica obligatoriamente la trazabilidad de requisitos (`E_MISSING_REQUIREMENT`), mientras el compilador resuelve el ABI de WebAssembly de forma transparente.
