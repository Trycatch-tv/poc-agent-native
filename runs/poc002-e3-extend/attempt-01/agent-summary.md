# Resumen de Ejecución — Ejercicio E3-002 (Extensión Funcional)

- **Ejercicio:** `E3-002` (Extensión funcional con validaciones de dominio y `status_code` tipado en Semantic IR v0.2).
- **Especificación:** `spec/task-002.yaml` (versión 2).
- **Intento:** 1 de 3 (`SC-06` límite <= 3 cumplido).
- **Cambios realizados en IR:**
  - `spec_version`: `1 -> 2`.
  - Agregado campo `status_code` a `OrderCalculation`.
  - Implementación de discriminación de errores de dominio:
    - `status_code = 1`: item con precio o cantidad negativa (`ERR_INVALID_ITEM`).
    - `status_code = 2`: costo de envío negativo (`ERR_INVALID_SHIPPING`).
    - `status_code = 3`: subtotal con impuestos superior a 1,000,000 centavos (`ERR_LIMIT_EXCEEDED`).
    - `status_code = 0`: cálculo exitoso (`STATUS_OK`).
- **Binario resultante:**
  - Tamaño: `459 bytes`.
  - SHA-256: `8079c054444d0541f555780e104a5259d2519a5556103104e1f14d0b35232f0e`.

## Resultados Observables

1. **Verificación semántica (`irtool verify`):**
   - 0 diagnósticos en 1 intento.
2. **Validación del binario WebAssembly (`irtool validate`):**
   - Valida 100 % con `wasmparser`.
3. **Pruebas públicas (`tests/public-002.yaml`):**
   - 12 de 12 pruebas pasadas (`100 %`, 18 ms).
4. **Pruebas reservadas (`tests/hidden-002.yaml`):**
   - 18 de 18 pruebas pasadas (`100 %`, 19 ms).
5. **Backend congelado (`SC-07`):**
   - Los binarios `target/debug/irtool.exe` y `target/debug/runner.exe` permanecieron 100 % intactos sin recompilar (mismos hashes de `runs/toolchain-freeze-poc002.sha256`).
