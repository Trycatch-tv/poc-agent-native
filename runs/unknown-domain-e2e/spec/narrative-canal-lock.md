# Sistema de Control de Esclusas e Hidrología de Canal

## 1. Visión general del producto
Sistema distribuido de control hidrodinámico y autorización de tránsito de embarcaciones en esclusas fluviales.
1. **Sensor Hidrológico (`hydro-sensor`):** Calcula el índice de flujo hidrodinámico a partir del nivel de agua y la presión en compuerta.
2. **Controlador de Esclusa (`lock-controller`):** Coordina la autorización de paso de buques, consulta el índice de flujo en `hydro-sensor`, obtiene recargos de lastre desde el host (`kv_get`) y registra auditorías de apertura (`emit_audit`).

## 2. Entidades de Dominio
### Decisión de Tránsito (`TransitDecision`)
- `status_code`: Código de resultado (0 = aprobado, 1 = calado inválido o nivel insuficiente, 2 = turbulencia excesiva).
- `flow_index`: Índice de flujo hidrodinámico reportado por el sensor.
- `ballast_fee`: Tarifa de lastre calculada en centavos.

## 3. Componente A: Sensor Hidrológico (`hydro-sensor`)
### Requisitos de hydro-sensor
- **`HYDRO-001` (Validación de lecturas físicas):** Si `water_level_cm <= 0` o `gate_pressure_kpa < 0`, la función `compute_flow_index` debe retornar `0`.
- **`HYDRO-002` (Cálculo de índice de flujo con tope de seguridad):** Cuando las lecturas son válidas, calcula el índice base como `(water_level_cm * 2) + gate_pressure_kpa`. Si el resultado es mayor o igual a `1000` (o supera `1000`), retorna exactamente `1000`; en caso contrario retorna `(water_level_cm * 2) + gate_pressure_kpa`.

## 4. Componente B: Controlador de Esclusa (`lock-controller`)
### Requisitos de lock-controller
- **`LOCK-001` (Validación de calado y nivel mínimo de cámara):** Si `vessel_draft_cm <= 0` o `water_level_cm < vessel_draft_cm`, rechaza inmediatamente el tránsito retornando un registro `TransitDecision` con `status_code = 1`, `flow_index = 0`, `ballast_fee = 0`, sin emitir auditoría.
- **`LOCK-002` (Evaluación hidrodinámica delegada a `hydro-sensor`):** Si el calado es válido (`vessel_draft_cm > 0` y `water_level_cm >= vessel_draft_cm`), invoca la operación importada `compute_flow_index(water_level_cm, gate_pressure_kpa)` del componente `hydro-sensor` (`wit_interface: "hydro-service"`, `capability: "component_call"`). Si el valor retornado es `<= 0` o `>= 1000`, rechaza el tránsito por turbulencia retornando `TransitDecision` con `status_code = 2`, `flow_index = compute_flow_index(water_level_cm, gate_pressure_kpa)`, `ballast_fee = 0`, sin emitir auditoría.
- **`LOCK-003` (Aprobación, cálculo de tarifa con `kv_get` y auditoría `emit_audit`):** Si `compute_flow_index(water_level_cm, gate_pressure_kpa)` está entre `1` y `999` inclusive, aprueba el tránsito: consulta el recargo por calado invocando `kv_get(vessel_draft_cm)` (`wit_interface: "host-services"`, `capability: "kv_store"`), emite el evento de auditoría `emit_audit(300)` (`wit_interface: "host-services"`, `capability: "audit_log"`) integrando la llamada en la expresión del registro (por ejemplo sumando `emit_audit(300) * 0` a `status_code` para que `status_code` sea `0`), y retorna `TransitDecision` con `status_code = 0`, `flow_index = compute_flow_index(water_level_cm, gate_pressure_kpa)` y `ballast_fee = (vessel_draft_cm * 3) + kv_get(vessel_draft_cm)`.

## 5. Contratos de Interfaz (WIT)
### Interfaz de `hydro-sensor`
```wit
interface hydro-service {
    compute-flow-index: func(water-level-cm: s32, gate-pressure-kpa: s32) -> s32;
}
```

### Interfaz de `lock-controller`
```wit
interface host-services {
    kv-get: func(key: s32) -> s32;
    emit-audit: func(event-type: s32) -> s32;
}

interface lock-service {
    evaluate-lock-transit: func(vessel-draft-cm: s32, water-level-cm: s32, gate-pressure-kpa: s32) -> transit-decision;
}
```
