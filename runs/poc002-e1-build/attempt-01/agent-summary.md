# Resumen de Ejecución — Ejercicio E1-002 (Construcción Inicial)

- **Ejercicio:** `E1-002` (Construcción inicial de orden comercial con subfunciones y records en Semantic IR v0.2).
- **Intento:** 1 de 3 (`SC-01` cumplido).
- **Especificación:** `spec/task-002.yaml` (versión 1).
- **Artefacto principal generado:** `ir/program-002.ir.json`.
- **Binario emitido:** `build/program-002.wasm` (`308 bytes`).
- **Hash SHA-256 del binario:** `f42bcb6e03c616b2891a9c4d1e060c4e141d502b01f44a326bc749d65a0d771a`.

## Resultados Observables

1. **Verificación semántica (`irtool verify`):**
   - 0 diagnósticos (`exit 0`).
2. **Validación del binario WebAssembly (`irtool validate`):**
   - Valida 100 % con `wasmparser`. Módulo WebAssembly Core con memoria lineal determinista y llamadas internas.
3. **Pruebas públicas (`tests/public-002.yaml`):**
   - 6 de 6 pruebas pasadas (`100 %`, 40 ms).
4. **Pruebas reservadas (`tests/hidden-002.yaml`):**
   - 9 de 9 pruebas pasadas (`100 %`, 49 ms).
5. **Success Criteria validados en E1-002:**
   - `SC-01`: Construcción en 1 intento (límite <= 3).
   - `SC-02`: 100 % en públicas (6/6) y reservadas (9/9).
   - `SC-12`: 0 líneas de WAT/WASM editadas por el agente.
   - `SC-13`: Semantic IR v0.2 es el único artefacto modificado.
   - `SC-14`: Diagnósticos = `[]` (< 500 B).
   - `SC-15`: Latencia total verify + compile < 50 ms.
   - `SC-16`: WASM valida limpiamente.
