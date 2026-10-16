# Agent Summary — E4: Mantenimiento de regla existente (`attempt-01`)

## 1. Objetivo del ejercicio
Evaluar la subhipótesis **H1-D** (**SC-08** y **SC-09**) y cerrar los criterios transversales (**SC-10** a **SC-16**): modificar una política existente en `calculate_shipping` (`spec/task.yaml` v3: envío gratis desde `4000` centavos en vez de `5000`, y tarifa plana de `600` centavos en vez de `500`) mediante una edición mínima y localizada en `ir/program.ir.json`, sin alterar `calculate_discount` ni tocar el backend congelado en `poc-e1-freeze`.

## 2. Delta localizado en `ir/program.ir.json` (`diff.patch`)
El diff entre `ir.before.json` (v2) e `ir.after.json` (v3) modifica exactamente 3 líneas de `ir/program.ir.json`:
1. `"spec_version": 2` → `"spec_version": 3`
2. En `calculate_shipping` (`$.functions[0].body.else.condition.right.value`): `5000` → `4000`
3. En `calculate_shipping` (`$.functions[0].body.else.else.value`): `500` → `600`

La función `calculate_discount` (`$.functions[1]`) permaneció `100 %` intacta.

## 3. Resultados de verificación, compilación, reproducibilidad y pruebas
- **`irtool verify`:** `0` diagnósticos (`diagnostics.json = []`, `exit code = 0`) en el primer intento (`repair_attempts = 0`).
- **Reproducibilidad (`SC-11`):** `3/3` compilaciones consecutivas de `ir/program.ir.json` produjeron exactamente el mismo binario `program.wasm` de `134` bytes con `sha256 = 947620fe23636397ba820b7cadf673e95fb3e09f68591bd0b54bfc8183d0a450`.
- **Validez WASM (`SC-10`):** `irtool validate build/program.wasm` aprobó con `exit code = 0`.
- **Pruebas públicas (`public-tests.json`):** `12/12` aprobadas (`6/6` pruebas actualizadas de `calculate_shipping` v3 + `6/6` pruebas de regresión de `calculate_discount`, **SC-08**).
- **Pruebas reservadas (`hidden-tests.json`):** `18/18` aprobadas (`9/9` pruebas reservadas de `calculate_shipping` v3 + `9/9` pruebas reservadas de `calculate_discount`, **SC-08**).
- **Estabilidad del toolchain (`toolchain.json`):** Los hashes SHA-256 de `target/debug/irtool.exe` y `target/debug/runner.exe` coinciden bit a bit con `runs/toolchain-freeze.sha256` (`backend_modified_since_freeze = false`, **SC-09**).
