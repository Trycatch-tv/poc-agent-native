# Resumen Comparativo Pareado — POC 002 (Agent-Native IR v0.2 vs Baseline Rust)

## Resumen Ejecutivo

| Dimensión | Agent-Native Semantic IR v0.2 | Baseline Rust (`wasm32-unknown-unknown`) | Ventaja Observada |
|---|---|---|---|
| **Velocidad de Toolchain (`verify + compile`)** | **`23 ms – 24 ms`** | `232 ms – 432 ms` (incremental) / `379 ms` (clean) | **`10.1x a 18.8x más rápido`** |
| **Volumen de Diagnóstico en Defecto (`E2` vs `B2`)** | **`593 bytes`** (JSONPath exacto) | `5,418 bytes` (JSONL `cargo check`) | **`9.14x más compacto (89.1 % menos ruido)`** |
| **Tamaño de Binario `.wasm`** | `308 B – 459 B` | `286 B – 400 B` | Ambos en rango ultra-compacto sub-500 bytes |
| **Corrección Funcional (Pass Rate)** | `100 %` (12/12 pub, 18/18 hid) | `100 %` (12/12 pub, 18/18 hid) | **Empate perfecto (`100 %`)** |
| **Reproducibilidad Binaria Exacta** | `3/3` compilaciones idénticas | `3/3` compilaciones idénticas | **Empate (`100 %`)** |
| **Gestión de Memoria y Punteros** | Bump Allocation determinista (`$heap_ptr = 1024`) en compilador, cero punteros manuales | Requiere `unsafe`, `static mut`, `&raw const` y structs `#[repr(C)]` | **Mayor seguridad y menor carga cognitiva en Agent-Native** |

---

## Tabla Detallada por Ejercicio

### 1. Construcción Inicial (E1-002 vs B1-002)
- **Agent-Native:** `11 ms` verify + `12 ms` compile = `23 ms`. Tamaño: `308 bytes`. 15/15 tests aprobados.
- **Baseline Rust:** `148 ms` check + `84 ms` build = `232 ms` (`379 ms` clean). Tamaño: `286 bytes`. 15/15 tests aprobados.
- **Factor de Aceleración:** **`10.09x`**.

### 2. Detección y Reparación Controlada (E2-002 vs B2-002)
- **Defecto inyectado:** campo `subtotal` en lugar de `subtotal_cents` en constructor de `OrderCalculation`.
- **Diagnóstico Agent-Native:** `593 bytes` formateados (`$.functions[2].body.else.fields.subtotal`), 0 warnings.
- **Diagnóstico Rust:** `5,418 bytes` en stream JSON con trazas de tipos no resueltos y advertencias de statics mutables.
- **Reducción de contexto:** **`9.14x menos bytes`**.
- **Reconstrucción:** `23 ms` en Agent-Native vs `210 ms` en Rust (**`9.13x más rápido`**).

### 3. Extensión Funcional con Errores de Dominio (E3-002 vs B3-002)
- **Cambio:** Introducción de `OrderCalculation.status_code` (0 = OK, 1 = ítem inválido, 2 = envío inválido, 3 = límite excedido).
- **Agent-Native:** Backend congelado intacto, `24 ms` cycle, `459 bytes`.
- **Baseline Rust:** `424 ms` cycle, `400 bytes`.
- **Factor de Aceleración:** **`17.67x`**.

### 4. Mantenimiento Modular Localizado (E4-002 vs B4-002)
- **Cambio:** Actualización de tasa impositiva del 10 % al 15 % en subfunción interna `calculate_tax`.
- **Agent-Native:** 2 líneas modificadas, `23 ms` rebuild, `3/3` builds con SHA-256 idéntico.
- **Baseline Rust:** 1 línea modificada, `432 ms` rebuild, `3/3` builds con SHA-256 idéntico.
- **Factor de Aceleración:** **`18.78x`**.
