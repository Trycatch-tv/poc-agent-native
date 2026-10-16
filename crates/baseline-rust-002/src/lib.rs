#![no_std]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
pub struct OrderCalculation {
    pub status_code: i32,
    pub subtotal_cents: i32,
    pub tax_cents: i32,
    pub shipping_cents: i32,
    pub total_cents: i32,
}

static mut RESULT: OrderCalculation = OrderCalculation {
    status_code: 0,
    subtotal_cents: 0,
    tax_cents: 0,
    shipping_cents: 0,
    total_cents: 0,
};

fn calculate_line_item(unit_price_cents: i32, quantity: i32) -> i32 {
    if unit_price_cents < 0 || quantity < 0 {
        -1
    } else {
        unit_price_cents * quantity
    }
}

fn calculate_tax(subtotal_cents: i32) -> i32 {
    if subtotal_cents < 0 {
        -1
    } else {
        subtotal_cents * 15 / 100
    }
}

#[no_mangle]
pub extern "C" fn calculate_order(
    price1: i32,
    qty1: i32,
    price2: i32,
    qty2: i32,
    shipping_cents: i32,
) -> *const OrderCalculation {
    let line1 = calculate_line_item(price1, qty1);
    let line2 = calculate_line_item(price2, qty2);

    unsafe {
        if line1 < 0 || line2 < 0 {
            RESULT = OrderCalculation {
                status_code: 1,
                subtotal_cents: 0,
                tax_cents: 0,
                shipping_cents: 0,
                total_cents: 0,
            };
        } else if shipping_cents < 0 {
            RESULT = OrderCalculation {
                status_code: 2,
                subtotal_cents: 0,
                tax_cents: 0,
                shipping_cents: 0,
                total_cents: 0,
            };
        } else {
            let subtotal = line1 + line2;
            let tax = calculate_tax(subtotal);
            let subtotal_with_tax = subtotal + tax;
            if subtotal_with_tax > 1_000_000 {
                RESULT = OrderCalculation {
                    status_code: 3,
                    subtotal_cents: 0,
                    tax_cents: 0,
                    shipping_cents: 0,
                    total_cents: 0,
                };
            } else {
                let total = subtotal_with_tax + shipping_cents;
                RESULT = OrderCalculation {
                    status_code: 0,
                    subtotal_cents: subtotal,
                    tax_cents: tax,
                    shipping_cents,
                    total_cents: total,
                };
            }
        }
        &raw const RESULT as *const OrderCalculation
    }
}
