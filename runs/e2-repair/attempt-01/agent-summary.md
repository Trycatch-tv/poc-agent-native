# Agent Summary — E2: Reparación controlada (`attempt-01`)

## 1. Objetivo del ejercicio
Evaluar la subhipótesis **H1-B** (**SC-04** y **SC-05**): verificar que un defecto sembrado en la IR válida de `E1` sea rechazado determinísticamente por `irtool verify` con código y ruta lógica exacta (`diagnostics.injected.json`), y que el agente repare `ir/program.ir.json` en máximo 2 intentos guiado por dicho diagnóstico sin modificar el toolchain congelado en `poc-e1-freeze`.

## 2. Inyección del defecto controlado (**SC-04**)
- **Archivo base:** `runs/e2-repair/attempt-01/ir.before.json` (`sha256: 953f73aca2c344b8dc167f73f4b14c53fb660ca37a79adcd29e2ac765a3df3e4`)
- **Defecto inyectado (`ir.injected.json`):** En `$.functions[0].body.condition.left`, se reemplazó `{"op": "param", "name": "subtotal_cents"}` por `{"op": "param", "name": "subtotal"}`.
- **Resultado de `irtool verify`:** Rechazado con `exit code = 1` y diagnóstico estructurado en `diagnostics.injected.json`:
  - `code`: `"E_UNKNOWN_PARAM"`
  - `function`: `"calculate_shipping"`
  - `path`: `"$.functions[0].body.condition.left"`
  - `expected`: `"subtotal_cents"`
  - `found`: `"subtotal"`

## 3. Reparación guiada por diagnóstico (**SC-05**)
- **Intentos requeridos (`repair_attempts`):** `1` (umbral `<= 2`).
- **Acción:** A partir de `path`, `expected` y `found` en `diagnostics.injected.json`, el agente restauró `"name": "subtotal_cents"` en `$.functions[0].body.condition.left` de `ir/program.ir.json`.
- **Verificación posterior:**
  - `irtool verify`: `0` diagnósticos (`diagnostics.json = []`, `exit code = 0`).
  - `irtool compile` + `irtool validate`: módulo `.wasm` válido de `76` bytes (`sha256 = 026cbbc30f6252a535cab1b1608df265f7641ff0ef5aae1b22fd36e8e3dee505`, idéntico bit a bit al de `E1`).
  - `runner test` (públicas): `6/6` aprobadas (`0` fallos).
  - `runner test` (reservadas): `9/9` aprobadas (`0` fallos).
  - **Integridad del freeze:** `irtool.exe` y `runner.exe` conservan exactamente los hashes registrados en `runs/toolchain-freeze.sha256` (`backend_modified_since_freeze = false`).
