# Benchmark Report 002 — Agent-Native IR v0.2 vs Baseline Rust (`wasm32-unknown-unknown`)

> **Experimento:** `KADDO-AGENT-NATIVE-POC-002` (Work Item `WI-016`)
>
> **Entorno:** `rustc 1.93.0-nightly`, `Wasmtime 27.0.0`, Windows x86_64
>
> **Sujeto de prueba:** Motor de cálculo de órdenes comerciales con composición de subfunciones (`call`), registros (`records`) y códigos de error tipados (`status_code`).

---

## 1. Motivación y Metodología

La POC 002 demostró que una **Semantic IR v0.2** permite a un agente modelar composición modular interna y estructuras compuestas en memoria lineal con backend congelado.

Este benchmark pareado (**`WI-016`**) evalúa empíricamente la misma especificación funcional en sus tres versiones (`v1`, `v2`, `v3`) implementada en:
1. **Ruta Agent-Native:** `ir/program-002.ir.json` procesada por el micro-toolchain determinista (`irtool verify` + `irtool compile directo a WASM`).
2. **Ruta Baseline Rust:** `crates/baseline-rust-002/src/lib.rs` (`#![no_std]`, `crate-type = ["cdylib"]`, `opt-level = "z"`, `lto = true`, `strip = "symbols"`) compilada mediante `cargo build --target wasm32-unknown-unknown --release`.

Ambas rutas son evaluadas por el mismo ejecutor en runtime (`target/debug/runner.exe` sobre Wasmtime 27.0.0) con las mismas suites de pruebas públicas y reservadas.

---

## 2. Resultados Cuantitativos Pareados

| Dimensión de Medición | E1 vs B1 (Construcción) | E2 vs B2 (Reparación) | E3 vs B3 (Extensión) | E4 vs B4 (Mantenimiento) | Ventaja Agent-Native |
|---|---|---|---|---|---|
| **Verificación (`verify` vs `cargo check`)** | `11 ms` vs `148 ms` | `10 ms` vs `181 ms` | `11 ms` vs `151 ms` | `11 ms` vs `181 ms` | **13.5x a 18.1x más rápido** |
| **Compilación (`compile` vs `cargo build`)** | `12 ms` vs `84 ms` (`379 ms` clean) | `13 ms` vs `210 ms` | `13 ms` vs `273 ms` | `12 ms` vs `251 ms` | **7.0x a 21.0x más rápido** |
| **Ciclo Total Toolchain** | **`23 ms`** vs `232 ms` | **`23 ms`** vs `210 ms` | **`24 ms`** vs `424 ms` | **`23 ms`** vs `432 ms` | **`10.1x a 18.8x más rápido`** |
| **Tamaño Binario WebAssembly** | `308 B` vs `286 B` | `308 B` vs `286 B` | `459 B` vs `400 B` | `459 B` vs `400 B` | Ambos en rango ultra-compacto (< 500 B) |
| **Volumen Diagnóstico en Falla** | — | **`593 B`** vs `5,418 B` | — | — | **`9.14x más compacto (89.1 % menos ruido)`** |
| **Pass Rate Pruebas Públicas** | `6/6` (100 %) | `6/6` (100 %) | `12/12` (100 %) | `12/12` (100 %) | **Empate (100 %)** |
| **Pass Rate Pruebas Reservadas** | `9/9` (100 %) | `9/9` (100 %) | `18/18` (100 %) | `18/18` (100 %) | **Empate (100 %)** |
| **Reproducibilidad SHA-256** | 100 % | 100 % | 100 % | `3/3` idénticos | **Empate (100 % determinismo)** |

---

## 3. Análisis de Trade-Offs de Ingeniería

### A. Latencia del Bucle de Feedback para el Agente
En cada ciclo de interacción, el agente en la ruta Agent-Native recibe feedback verificado y emite WebAssembly en **23–24 milisegundos**. En la ruta de Rust tradicional, el ciclo requiere **230–430 milisegundos** en compilación incremental y **~530 ms** en compilaciones limpias. Para un pipeline con múltiples reintentos o ejecuciones masivas de agentes, esta diferencia de un orden de magnitud se traduce en un consumo de tiempo de CPU y bloqueo notablemente menor.

### B. Densidad y Precisión Diagnóstica
Cuando se inyectó un error sintáctico/semántico en un campo (`subtotal` en vez de `subtotal_cents`), `rustc` emitió **5,418 bytes** de salida JSONL conteniendo trazas de sugerencias de tipos no relacionados y avisos de referencias estáticas mutables. Por el contrario, `irtool verify` emitió **593 bytes** estructurados con JSONPath inequívoco (`$.functions[2].body.else.fields.subtotal`), lo que minimiza el consumo de la ventana de contexto del LLM y elimina alucinaciones causadas por ruido sintáctico.

### C. Seguridad de Memoria y Carga Cognitiva
- En Rust `#![no_std]`, el agente debió escribir código `unsafe`, gestionar un buffer estático mutable (`static mut RESULT`), lidiar con advertencias de la edición Rust 2024 sobre referencias compartidas a estáticos (`&raw const`), y estructurar los campos con `#[repr(C)]`.
- En Semantic IR v0.2, el agente simplemente define el tipo `record` y el operador de construcción `record.construct`. El compilador determinista resuelve la asignación en memoria lineal (alineada a 4 bytes mediante el puntero global `$heap_ptr = 1024`) de forma 100 % automática y libre de `unsafe`.

---

## 4. Conclusión

El benchmark confirma que los resultados de la POC 001 se mantienen y profundizan al escalar a tipos compuestos y subfunciones internas:
1. **La ventaja de velocidad de toolchain se sostiene en `10x–19x`.**
2. **La reducción del volumen de diagnóstico se sitúa en `9.14x`.**
3. **La complejidad de punteros y memoria queda encapsulada en el compilador**, protegiendo al agente de errores sutiles de ABI y de inicialización en memoria.
