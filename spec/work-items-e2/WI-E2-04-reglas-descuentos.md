---
id: WI-E2-04
title: Políticas de Descuentos Comerciales
status: active
version: 1
---

# WI-E2-04 — Políticas de Descuentos Comerciales

## Requisitos

- **DISC-001**: Recuperación de tasa de descuento del cliente mediante `host.kv_get(customer_id)`. Se aplica el porcentaje sobre el subtotal del pedido.
- **DISC-002**: Límite máximo de tasa de descuento del 20%. Si el valor almacenado en el host es mayor a 20%, se aplica únicamente el 20%.
