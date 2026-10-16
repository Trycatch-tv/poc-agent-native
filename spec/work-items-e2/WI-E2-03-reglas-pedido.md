---
id: WI-E2-03
title: Procesamiento de Líneas, Impuestos y Crédito
status: active
version: 1
---

# WI-E2-03 — Procesamiento de Líneas, Impuestos y Crédito

## Requisitos

- **ORD-001**: Flujo orquestado de cálculo de pedido calculando subtotal, descuento, impuestos (15%), envío y total.
- **LINE-001**: Validación de líneas de pedido. Si `unit_price <= 0` o `quantity <= 0`, retorna `status_code = 1` y todos los totales en 0.
- **TAX-001**: Cálculo de impuesto al valor agregado del 15% sobre la base imponible (`subtotal - discount`).
- **CREDIT-001**: Verificación de límite de crédito con `host.check_credit`. Si es rechazado, retorna `status_code = 3` y registra auditoría de rechazo `emit_audit(999, total)`.
- **AUD-001**: Registro de auditoría de aprobación `emit_audit(100, total)` en caso de éxito (`status_code = 0`).
