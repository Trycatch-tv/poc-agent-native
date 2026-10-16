# Sistema de Reserva y Asignación de Recursos Multi-Tenant

**Subtítulo:** Gestión descentralizada de cuotas y coordinación de reservas con trazabilidad determinista.
**Dominio:** Computación en la nube e infraestructura elástica.

## Componentes del Sistema

### Componente A: quota-engine

- **Nombre:** Motor de Cuotas de Recursos
- **Descripción:** Evalúa límites de capacidad y disponibilidad según consumo actual y unidades solicitadas.
- **Archivo WIT:** `wit/quota-engine.wit`
- **World:** `quota-engine`
- **Interfaces exportadas:**
  - `kaddo:resources/quota-service@0.1.0`
- **Interfaces importadas:** ninguna
- **Capacidades requeridas:** ninguna

### Componente B: reservation-coordinator

- **Nombre:** Coordinador de Reservas
- **Descripción:** Coordina las reservas de recursos, verifica el estado del tenant y emite auditorías.
- **Archivo WIT:** `wit/reservation-coordinator.wit`
- **World:** `reservation-coordinator`
- **Interfaces exportadas:**
  - `kaddo:resources/reservation-service@0.1.0`
- **Interfaces importadas:**
  - `kaddo:resources/quota-service@0.1.0`
  - `kaddo:resources/host-services@0.1.0`
- **Capacidades requeridas:**
  - `host_services`
- **Dependencias entre componentes:**
  - `reservation-coordinator` invoca `quota-engine` para evaluar disponibilidad.

## Requisitos del Sistema

### Requisitos de quota-engine

- **QUOTA-001**: Validación de argumentos no negativos. Si `requested_units <= 0` o `current_usage < 0`, retorna 0 (rechazado).
- **QUOTA-002**: Límite de cuota global. Si `current_usage + requested_units > 1000`, retorna 0 (rechazado por sobrepasar límite global de 1000 unidades). En caso contrario, retorna 1 (aprobado).

### Requisitos de reservation-coordinator

- **RES-001**: Validación de tenant. Si `tenant_id <= 0`, retorna `status_code = 1` (tenant inválido), `approved_units = 0`, `penalty_points = 0`.
- **RES-002**: Validación de estado activo en host. Consulta `tenant_status(tenant_id)`. Si el estado retornado es 0 (bloqueado/inactivo), retorna `status_code = 2`, `approved_units = 0`, `penalty_points = 0`.
- **RES-003**: Evaluación de cuota delegada. Invoca `check_quota(tenant_id, requested_units, current_usage)`. Si retorna 0, retorna `status_code = 3` (cuota insuficiente), `approved_units = 0`, `penalty_points = 0`.
- **RES-004**: Aprobación de reserva y auditoría. Si la cuota fue aprobada, emite `emit_audit(200, requested_units)` y retorna `status_code = 0`, `approved_units = requested_units`, `penalty_points = 0`.
