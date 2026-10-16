# Especificación Narrativa del Sistema Comercial (POC 005)

## 1. Visión general del producto

El sistema procesa pedidos comerciales en línea mediante una arquitectura desacoplada basada en componentes autónomos. La solución se compone de dos subsistemas funcionales con responsabilidades claramente delimitadas:

1. **Calculador de Envíos (`shipping-calculator`):** Determina el costo del flete en centavos a partir del subtotal del pedido, el peso en gramos del paquete y la modalidad de entrega (estándar o exprés).
2. **Procesador de Órdenes (`order-calculator`):** Valida la integridad de las líneas de pedido, calcula los subtotales, aplica deducciones por fidelidad o promociones mediante consultas a catálogos externos, aplica las alícuotas impositivas vigentes, delega la cotización del envío al componente especializado de fletes, valida la solvencia crediticia del cliente y registra eventos auditables para control financiero.

Ambos componentes interactúan de manera tipada a través de contratos de interfaz estandarizados (WebAssembly Interface Types - WIT), preservando aislamiento de memoria y permitiendo su evolución independiente.

---

## 2. Entidades de Dominio

### Línea de Pedido (`LineItem`)
- `price_cents`: Precio unitario en centavos (debe ser estrictamente mayor a 0).
- `quantity`: Cantidad solicitada (debe ser estrictamente mayor a 0).

### Resultado del Procesamiento (`OrderResult`)
- `status_code`: Código numérico que indica el estado del pedido:
  - `0`: Pedido exitoso y liquidado (`ORDER_OK`).
  - `1`: Error por línea de pedido inválida (`ERR_INVALID_LINE`).
  - `2`: Error por flete inválido o peso inconsistente (`ERR_INVALID_SHIPPING`).
  - `3`: Error por límite de crédito excedido o cliente sin solvencia (`ERR_CREDIT_LIMIT_EXCEEDED`).
- `subtotal_cents`: Monto neto consolidado de las líneas de producto.
- `discount_cents`: Deducción aplicada según beneficios del cliente o promociones.
- `tax_cents`: Impuesto liquidado sobre el subtotal neto.
- `shipping_cents`: Tarifa de flete calculada por el componente de envíos.
- `total_cents`: Monto final exigible (`subtotal - discount + tax + shipping`).

---

## 3. Componente A: Calculador de Envíos (`shipping-calculator`)

### Responsabilidad
Calcular de forma determinista la tarifa de envío en centavos considerando el volumen, peso y tipo de servicio.

### Reglas Comerciales de Flete

- **`SHIP-001` (Validación de entrada):** Si el peso en gramos es menor o igual a cero (`weight_grams <= 0`), la operación debe retornar un código de error `-1` (o señalar flete inválido).
- **`SHIP-002` (Flete gratuito por volumen de compra):** Si el subtotal de compra es mayor o igual a 50,000 centavos (\$500.00) y el servicio no es exprés (`is_express == 0`), el envío estándar es completamente gratuito (`shipping = 0`).
- **`SHIP-003` (Tarifa estándar por peso):**
  - Paquetes livianos (`weight_grams <= 2000`): Flete base de 500 centavos (\$5.00).
  - Paquetes medianos (`2000 < weight_grams <= 5000`): Flete base de 800 centavos (\$8.00).
  - Paquetes pesados (`weight_grams > 5000`): Flete base de 1,200 centavos (\$12.00).
- **`SHIP-004` (Recargo de entrega exprés):** Si se solicita modalidad exprés (`is_express == 1`), se añade un recargo fijo de 600 centavos (\$6.00) sobre la tarifa base calculada según el peso (sin exoneración por volumen).

---

## 4. Componente B: Procesador de Órdenes (`order-calculator`)

### Responsabilidad
Gestionar el flujo integral de liquidación de órdenes comerciales, coordinando validaciones, descuentos, impuestos, fletes y auditoría.

### Reglas Comerciales de Orden

- **`LINE-001` (Integridad de líneas):** Ambas líneas de compra deben poseer precio y cantidad positivos (`price1 > 0`, `qty1 > 0`, `price2 > 0`, `qty2 > 0`). Si alguna línea viola esta condición, la orden aborta con `status_code = 1` y todos los importes en 0.
- **`LINE-002` (Cálculo de subtotal):** El subtotal corresponde a la suma de los productos de cada línea: `(price1 * qty1) + (price2 * qty2)`.
- **`DISC-001` (Consulta externa de beneficios):** Para clientes registrados (`customer_id > 0`), el sistema consulta el almacén de beneficios del host (`kv_get(customer_id)`) para obtener la tasa de descuento porcentual asignada.
- **`DISC-002` (Tope de descuento):** El descuento no puede superar el 20 % del subtotal (`discount_cents = min(subtotal * discount_rate / 100, subtotal * 20 / 100)`). Si el cliente no registra descuento (`kv_get <= 0`), `discount_cents = 0`.
- **`TAX-001` (Liquidación fiscal):** El impuesto al valor agregado se calcula como el 15 % del subtotal imponible (`(subtotal_cents - discount_cents) * 15 / 100`).
- **`SHIP-DELEGATE` (Delegación de flete):** El procesador de órdenes no implementa la lógica de fletes; invoca directamente al componente `shipping-calculator` mediante el contrato de interfaz WIT `quote-shipping`. Si el componente de fletes retorna error (`< 0`), la orden finaliza con `status_code = 2`.
- **`CREDIT-001` (Verificación de solvencia):** Antes de confirmar la orden, se invoca el servicio de crédito del host `check_credit(customer_id, total_cents)`. Si el crédito es denegado (`false`), la orden aborta con `status_code = 3`.
- **`AUDIT-001` (Registro auditable):** Para cada orden aprobada (`status_code = 0`), el sistema debe emitir obligatoriamente un evento de auditoría al host `emit_audit(100, total_cents)`. Si la orden fue rechazada por crédito insuficiente (`status_code = 3`), emite `emit_audit(999, total_cents)`.

---

## 5. Contratos de Interfaz (WIT)

### Interfaz Exportada por `shipping-calculator` (`kaddo:commercial/shipping-service@0.1.0`)
```wit
interface shipping-service {
    quote-shipping: func(
        subtotal-cents: s32,
        weight-grams: s32,
        is-express: s32
    ) -> s32;
}
```

### Interfaz Exportada por `order-processor` (`kaddo:commercial/order-calculator@0.1.0`)
```wit
interface order-calculator {
    use types.{order-result};

    calculate-order: func(
        customer-id: s32,
        price1: s32,
        qty1: s32,
        price2: s32,
        qty2: s32,
        weight-grams: s32,
        is-express: s32
    ) -> order-result;
}
```

---

## 6. Criterios de Aceptación del Sistema

1. **Procesamiento estándar exitoso:** Una orden válida para un cliente solvente con flete estándar liquida subtotal, descuento, impuesto y flete con `status_code = 0` y emite evento `100`.
2. **Rechazo por datos inconsistentes:** Una línea con precio o cantidad <= 0 aborta inmediatamente con `status_code = 1` sin consultar crédito ni emitir auditoría.
3. **Rechazo por flete no asegurable:** Si el peso en gramos es inválido (`<= 0`), la orden finaliza con `status_code = 2`.
4. **Rechazo por crédito insuficiente:** Si `check_credit` retorna falso, la orden devuelve `status_code = 3` y emite evento `999`.
5. **Composición verificable:** Los componentes `shipping-calculator` y `order-processor` se compilan independientemente a componentes WebAssembly y se componen mediante `wasmtime::component::Linker`.
