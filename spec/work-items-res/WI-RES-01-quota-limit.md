---
id: WI-RES-01
title: Expansión de cuota máxima por tenant
status: active
version: 2
component: quota-engine
replaces: QUOTA-002
---

# WI-RES-01 — Expansión de cuota máxima por tenant

## Requisitos Modificados

- **QUOTA-002** (v2): Si `current_usage + requested_units > 2500`, retorna 0 (rechazado por sobrepasar límite global ampliado de 2500 unidades). En caso contrario, retorna 1 (aprobado).
