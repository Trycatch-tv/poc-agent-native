---
id: WI-LOCK-01
title: Amortiguación hidrodinámica en cálculo de índice de flujo
component: hydro-sensor
status: active
version: 2
replaces: HYDRO-002
---

# WI-LOCK-01 — Amortiguación hidrodinámica en sensor de flujo

### Requisitos de hydro-sensor

- **HYDRO-002**: Con las válvulas de amortiguación activadas, cuando `water_level_cm > 0` y `gate_pressure_kpa >= 0`, `compute_flow_index` calcula el índice de flujo amortiguado como `water_level_cm + gate_pressure_kpa` (factor lineal 1x en lugar de multiplicar `water_level_cm * 2`). Si `water_level_cm + gate_pressure_kpa >= 1000`, retorna el tope máximo `1000`; en caso contrario retorna `water_level_cm + gate_pressure_kpa`.
