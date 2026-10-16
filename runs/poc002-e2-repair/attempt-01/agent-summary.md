# Resumen de Ejecución — Ejercicio E2-002 (Reparación Controlada)

- **Ejercicio:** `E2-002` (Reparación controlada guiada por diagnósticos JSONPath).
- **Defecto semántico inyectado:** `E_UNKNOWN_FIELD` en el constructor de `OrderCalculation` (`subtotal` en lugar de `subtotal_cents`).
- **Ubicación del defecto:** `$.functions[2].body.else.fields.subtotal`.
- **Diagnóstico del verificador:**
  - Código: `E_UNKNOWN_FIELD`.
  - Ruta JSONPath: `$.functions[2].body.else.fields.subtotal`.
  - Campos esperados: `subtotal_cents | tax_cents | shipping_cents | total_cents`.
  - Campo encontrado: `subtotal`.
- **Intentos de reparación del agente:** 1 intento (`SC-05` límite <= 2 cumplido).
- **Binario resultante:**
  - Tamaño: `308 bytes`.
  - SHA-256: `f42bcb6e03c616b2891a9c4d1e060c4e141d502b01f44a326bc749d65a0d771a` (exactamente idéntico bit a bit al artefacto previo de `E1-002`).

## Resultados Observables

1. **Rechazo determinista (`SC-04`):**
   - El compilador/verificador rechazó la IR con `exit code = 1` y diagnóstico estructurado JSONPath.
2. **Reparación en 1 intento (`SC-05`):**
   - El diff fue de 1 línea: `"subtotal": {` -> `"subtotal_cents": {`.
3. **Pruebas públicas y reservadas:**
   - Públicas: 6/6 pasadas (46 ms).
   - Reservadas: 9/9 pasadas (35 ms).
4. **Backend congelado intacto:**
   - Los hashes de `target/debug/irtool.exe` y `target/debug/runner.exe` coinciden al 100 % con `runs/toolchain-freeze-poc002.sha256`.
