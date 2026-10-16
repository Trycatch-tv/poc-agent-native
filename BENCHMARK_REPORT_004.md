# Benchmark Report 004 — Agent-Native Semantic IR v0.4 vs Baseline Rust con Interfaces WIT y Component Model

> **Experimento:** `KADDO-AGENT-NATIVE-POC-004` (Work Item `WI-032`)
>
> **Entorno:** `rustc 1.93.0-nightly`, `Wasmtime 27.0.0`, Windows x86_64
>
> **Sujeto de prueba:** Motor de cálculo comercial con WebAssembly Component Model, interfaces WIT estandarizadas (`order-calculator`, `host-services`), Canonical ABI y cotización de envío (`quote-shipping`).

---

## 1. Motivación y Metodología

La POC 004 demostró que **Semantic IR v0.4** permite a un agente de IA diseñar, evolucionar y componer lógica comercial en contratos estandarizados de **WIT** (WebAssembly Interface Types) con empaquetado canónico de componentes y backend 100 % congelado post-`E1-004`.

Este benchmark pareado (**`WI-032`**) evalúa de forma empírica y reproducible la misma especificación funcional (`spec/task-004.yaml` v3) implementada en dos rutas tecnológicas independientes:
1. **Ruta Agent-Native:** `ir/program-004.ir.json` procesada por el micro-toolchain determinista (`irtool verify` + `irtool compile` con sección personalizada `component-type:order-processor` y metadatos WIT).
2. **Ruta Baseline Rust:** `crates/baseline-rust-004/src/lib.rs` (`#![no_std]`, `crate-type = ["cdylib"]`, `opt-level = "z"`, `lto = true`, `strip = "symbols"`, imports de host y sección `component-type:order-processor`).

Ambas rutas fueron ejecutadas y verificadas por el mismo ejecutor en runtime (`target/release/runner.exe` sobre Wasmtime 27.0.0) contra las mismas suites de pruebas públicas (`tests/public-004.yaml`, 9 casos) y reservadas (`tests/hidden-004.yaml`, 12 casos), sumando 21 casos de prueba exhaustivos con validación de valores estructurados, estados de host y efectos secundarios secuenciales.

---

## 2. Resultados Cuantitativos Pareados

| Dimensión de Medición | Semantic IR v0.4 (Agent-Native) | Baseline Rust (`wasm32-unknown-unknown`) | Ventaja Agent-Native |
|---|---|---|---|
| **Verificación (`verify` vs `cargo check`)** | **`30.1 ms`** | `161.5 ms` | **`5.4x más rápido`** |
| **Compilación (`compile` vs `cargo build`)** | **`24.6 ms`** | `76.9 ms` incremental (`351.8 ms` clean) | **`3.1x a 14.3x más rápido`** |
| **Ciclo Total Toolchain (`verify + compile`)** | **`54.7 ms`** | `238.4 ms` incremental (`513.3 ms` clean) | **`4.4x a 9.4x más rápido`** |
| **Tamaño Binario Componente (`.wasm`)** | `956 bytes` | `761 bytes` | Ambos ultra-compactos (< 1 KB) |
| **Volumen Diagnóstico en Desalineación WIT** | **`366 bytes`** (JSONPath exacto) | `~8,800 bytes` (`cargo check --message-format=json`) | **`24.0x más compacto (95.8 % menos tokens)`** |
| **Pass Rate Pruebas Públicas (`9/9`)** | `9/9 (100 %)` en 4 ms | `9/9 (100 %)` en 3 ms | **Empate (100 % corrección)** |
| **Pass Rate Pruebas Reservadas (`12/12`)** | `12/12 (100 %)` en 4 ms | `12/12 (100 %)` en 2 ms | **Empate (100 % corrección)** |
| **Auditoría de Efectos Secundarios (`expected_host_calls`)** | `100 % aprobados` | `100 % aprobados` | **Empate (100 % trazabilidad)** |
| **Reproducibilidad Binaria SHA-256** | `3/3 builds deterministas` | `3/3 builds deterministas` | **Empate (100 % determinismo)** |

---

## 3. Análisis de Trade-Offs de Ingeniería

### A. Latencia y Agilidad en el Bucle de Feedback del Agente
En cada ciclo de interacción autónoma, el agente recibe feedback estático completo y emite el artefacto WebAssembly Component en **54.7 ms**. En la ruta de Rust con LLVM, el ciclo requiere **238.4 ms** en modo incremental y más de **medio segundo (513 ms)** en compilaciones en frío. Para agentes iterativos que prueban hipótesis y reparan defectos, la latencia sub-60ms de la Semantic IR permite múltiples bucles de ajuste en el tiempo que Rust tarda en inicializar una sola compilación.

### B. Compacidad Diagnóstica y Consumo de Contexto
Ante una desalineación de interfaz o discrepancia de tipos en el contrato WIT, el verificador de Semantic IR v0.4 emite un diagnóstico estructurado de **366 bytes** con el código exacto (`E_UNKNOWN_EXPORT_INTERFACE`), la ruta JSONPath (`$.functions[9].wit_interface`), el valor esperado (`order-calculator`) y el encontrado. Rust emite más de **8.8 KB** de estructuras JSON complejas con referencias a spans sintácticos, macros de expansión y sugerencias de ayuda que congestionan la ventana de contexto del LLM y aumentan la probabilidad de alucinaciones.

### C. Abstracción Segura sin `unsafe` ni Boilerplate de Memoria
- En Rust `#![no_std]`, el agente debe gestionar punteros crudos (`&raw const`), variables globales mutables estáticas (`static mut RESULT`), bloques `unsafe` y directivas de memoria C (`#[repr(C)]`), introduciendo riesgos de concurrencia y corrupción si no se dominan estos patrones de bajo nivel.
- En Semantic IR v0.4, la gestión de memoria lineal, el empaquetado de registros y la interfaz WIT son primitivas declarativas nativas de primer nivel. El compilador resuelve automáticamente la ubicación en heap y las llamadas a funciones sin que el agente deba manipular punteros ni invocar código inseguro.

### D. Interoperabilidad Ecosistémica
Tanto el componente Agent-Native como el binario de Rust incorporan la sección personalizada estándar `component-type:order-processor` con metadatos WIT, demostrando que la Semantic IR v0.4 es 100 % compatible con el estándar WebAssembly Component Model y apta para orquestarse en runtimes heterogéneos modernos.

---

## 4. Conclusión

El benchmark de la Fase 4 confirma empíricamente que:
1. **La Semantic IR v0.4 mantiene una ventaja de latencia de 4.4x a 9.4x** sobre Rust, garantizando bucles completos de feedback sub-60ms.
2. **La compacidad diagnóstica ofrece una reducción del 95.8 % en tokens de error (24.0x más conciso)**, maximizando la eficiencia de inferencia del agente.
3. **La emisión de componentes WebAssembly con contratos WIT es 100 % viable, determinista y segura**, validando la transición a la Fase 5 (orquestación multi-componente).
