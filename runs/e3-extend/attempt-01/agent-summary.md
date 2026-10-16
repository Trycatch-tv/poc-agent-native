# Agent Summary — E3: Extensión funcional (`attempt-01`)

## 1. Objetivo del ejercicio
Evaluar la subhipótesis **H1-C** (**SC-06** y **SC-07**): verificar que el programa en Semantic IR v0.1 puede extenderse con una nueva función exportada (`calculate_discount(subtotal_cents: i32, premium: bool) -> i32`) a partir de `spec/task.yaml` v2 sin alterar la función existente `calculate_shipping` y sin modificar el backend (`irtool` y `runner`) congelado en `poc-e1-freeze`.

## 2. Cambios realizados en `ir/program.ir.json`
- Se incrementó `spec_version` de `1` a `2`.
- Se preservó sin cambios la definición completa de `calculate_shipping` (`SHIP-001`, `SHIP-002`, `SHIP-003`).
- Se añadió la función exportada `calculate_discount`:
  - Parámetros: `subtotal_cents: i32`, `premium: bool` (`i32` en la frontera WASM: `0 = false`, `1 = true`).
  - Retorno: `i32`.
  - Requisitos: `["DISC-001", "DISC-002", "DISC-003"]`.
  - Cuerpo (`if` anidado):
    1. Si `lt_s(param("subtotal_cents"), const(0))` → `const(-1)` (`DISC-001`).
    2. Si no, si `eq(param("premium"), const(false))` → `const(0)` (`DISC-002`).
    3. Si no → `div_s(param("subtotal_cents"), const(10))` (`DISC-003`).

## 3. Resultados de verificación, compilación y pruebas
- **`irtool verify`:** `0` diagnósticos (`diagnostics.json = []`, `exit code = 0`) en el primer intento (`repair_attempts = 0`).
- **`irtool compile` + `irtool validate`:** Módulo WebAssembly válido de `134` bytes (`sha256 = e9450f963e606588e4c01c7b2860e1735598dd0f040ed221a62f11de42211e02`) exportando `calculate_shipping` y `calculate_discount`.
- **Pruebas públicas (`public-tests.json`):** `12/12` aprobadas (`6/6` regresión de `calculate_shipping` + `6/6` nuevas de `calculate_discount`, **SC-06**).
- **Pruebas reservadas (`hidden-tests.json`):** `18/18` aprobadas (`9/9` regresión de `calculate_shipping` + `9/9` casos reservados de borde de `calculate_discount`, **SC-06**).
- **Estabilidad del toolchain (`toolchain.json`):** Los hashes SHA-256 de `target/debug/irtool.exe` y `target/debug/runner.exe` coinciden bit a bit con `runs/toolchain-freeze.sha256` (`backend_modified_since_freeze = false`, **SC-07**).
