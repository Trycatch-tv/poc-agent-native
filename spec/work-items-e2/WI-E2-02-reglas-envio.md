---
id: WI-E2-02
title: Reglas de Cálculo de Tarifas de Envío
status: active
version: 1
---

# WI-E2-02 — Reglas de Cálculo de Tarifas de Envío

## Requisitos

- **SHIP-001**: Validación de peso positivo. Si `weight_grams <= 0`, retorna código de error `-1`.
- **SHIP-002**: Envío gratis por volumen de compra. Si `subtotal_cents >= 50000` y `is_express == 0`, la tarifa de envío es 0 centavos.
- **SHIP-003**: Tarifas base estándar por tramos de peso:
  - Hasta 2,000 g: 500 centavos ($5.00).
  - De 2,001 g a 5,000 g: 800 centavos ($8.00).
  - Mayor a 5,000 g: 1,200 centavos ($12.00).
- **SHIP-004**: Recargo por servicio exprés. Si `is_express == 1`, se añade un recargo fijo de 600 centavos ($6.00) sobre la tarifa base de peso.
