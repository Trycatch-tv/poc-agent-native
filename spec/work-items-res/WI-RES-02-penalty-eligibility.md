---
id: WI-RES-02
title: Incorporación de cálculo de penalización en coordinación
status: active
version: 2
component: reservation-coordinator
replaces: RES-004
---

# WI-RES-02 — Incorporación de cálculo de penalización en coordinación

## Requisitos Modificados

- **RES-004** (v2): Si la cuota fue aprobada, calcula penalización: si `requested_units > 1000`, asigna `penalty_points = 50`; en caso contrario `penalty_points = 0`. Emite `emit_audit(200, requested_units)` y retorna `status_code = 0`, `approved_units = requested_units`, y los `penalty_points` calculados.
