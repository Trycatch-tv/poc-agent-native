# Resumen de Ejecución — Ejercicio E4-002 (Mantenimiento Modular Localizado)

- **Ejercicio:** `E4-002` (Mantenimiento modular localizado en subfunción interna `calculate_tax` de Semantic IR v0.2).
- **Especificación:** `spec/task-002.yaml` (versión 3 — `TAX-002-V3` actualizando alícuota impositiva de 10 % a 15 %).
- **Intento:** 1 de 3 (`SC-06` límite <= 3 cumplido).
- **Aislamiento y granularidad del cambio en IR:**
  - `spec_version`: `2 -> 3`.
  - Subfunción modificada: únicamente `calculate_tax` cambiando literal `"value": 10` a `"value": 15`.
  - Funciones no modificadas: `calculate_line_item`, `calculate_order`.
  - Tipos no modificados: `LineItem`, `OrderCalculation`.
  - Diff total en el programa: exactamente 2 líneas modificadas.
- **Binario resultante:**
  - Tamaño: `459 bytes`.
  - SHA-256: `0a2265bca6b1431f2ea15bb3da9831f5e1f8082230c36fd1659c613f9af88c85`.
  - Reproducibilidad (`SC-11`): 3 compilaciones consecutivas arrojaron idéntico hash SHA-256.

## Resultados Observables

1. **Verificación semántica (`irtool verify`):**
   - 0 diagnósticos en 1 intento.
2. **Validación del binario WebAssembly (`irtool validate`):**
   - Valida 100 % con `wasmparser`.
3. **Pruebas públicas (`tests/public-002.yaml`):**
   - 12 de 12 pruebas pasadas (`100 %`, 18 ms).
4. **Pruebas reservadas (`tests/hidden-002.yaml`):**
   - 18 de 18 pruebas pasadas (`100 %`, 20 ms).
5. **Backend congelado (`SC-07`):**
   - Los binarios `target/debug/irtool.exe` y `target/debug/runner.exe` permanecieron 100 % intactos sin recompilar (mismos hashes de `runs/toolchain-freeze-poc002.sha256`).
