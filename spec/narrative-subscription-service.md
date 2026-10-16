# Especificación Narrativa de la Plataforma de Suscripciones SaaS

## 1. Visión general del producto

La plataforma gestiona el cobro mensual y consumo de infraestructura de clientes en un modelo SaaS. El sistema se descompone en dos componentes WebAssembly independientes:

1. **Rastreador de Uso (`usage-tracker`):** Determina las unidades de cómputo consumidas según los eventos procesados y el volumen de datos transferidos.
2. **Facturación de Suscripción (`subscription-billing`):** Liquida el cargo mensual compuesto por la tarifa base del plan recurrente y el consumo variable facturado a través de `usage-tracker`.

Ambos componentes interactúan mediante contratos tipados WebAssembly Interface Types (WIT).

---

## 2. Entidades de Dominio

### Resultado de Liquidación (`BillingResult`)
- `status_code`: Estado del cálculo (`0` = exitoso).
- `base_plan_cents`: Tarifa mensual del plan base en centavos.
- `usage_charge_cents`: Importe liquidado por consumo volumétrico.
- `total_charge_cents`: Total final consolidado (`base_plan_cents + usage_charge_cents`).

---

## 3. Componente A: Rastreador de Uso (`usage-tracker`)

### Reglas
- **`USAGE-001` (Validación de entrada):** Si `events_count < 0`, aborta retornando código de error `-1`.
- **`USAGE-002` (Cálculo de unidades):** Calcula `unidades = events_count + (bytes_transferred_mb / 10)`.

---

## 4. Componente B: Facturación de Suscripción (`subscription-billing`)

### Reglas
- **`BILL-001` (Plan base mensual):** La cuota base fija es de 2,900 centavos ($29.00 USD) al mes.
- **`BILL-002` (Cargo por uso):** Invoca `usage-tracker.calculate_units` y factura a razón de 5 centavos por unidad calculada.
