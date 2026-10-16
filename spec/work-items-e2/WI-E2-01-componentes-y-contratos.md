---
id: WI-E2-01
title: Arquitectura de Componentes y Contratos WIT
status: active
version: 1
---

# WI-E2-01 — Arquitectura de Componentes y Contratos WIT

## Requisitos

- **COMP-001**: El sistema se estructura en dos componentes WebAssembly independientes: `shipping-calculator` y `order-processor`.
- **WIT-001**: `shipping-calculator` exporta la interfaz `shipping-service` definida en `wit/shipping-calculator.wit` con la función `quote-shipping(subtotal-cents: s32, weight-grams: s32, is-express: s32) -> s32`.
- **WIT-002**: `order-processor` importa `shipping-service` y `host-services` y exporta `order-calculator` con la función `calculate-order(...) -> order-result`.
