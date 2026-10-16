#![no_std]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[link(wasm_import_module = "host")]
extern "C" {
    fn kv_get(key: i32) -> i32;
    fn check_credit(customer_id: i32, amount: i32) -> i32;
    fn emit_audit(event_id: i32, payload: i32) -> i32;
}

#[repr(C)]
pub struct OrderResult {
    pub status_code: i32,
    pub subtotal_cents: i32,
    pub discount_cents: i32,
    pub tax_cents: i32,
    pub shipping_cents: i32,
    pub total_cents: i32,
}

static mut RESULT: OrderResult = OrderResult {
    status_code: 0,
    subtotal_cents: 0,
    discount_cents: 0,
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

fn calculate_tax(taxable_amount_cents: i32) -> i32 {
    if taxable_amount_cents < 0 {
        0
    } else {
        taxable_amount_cents * 12 / 100
    }
}

#[no_mangle]
pub extern "C" fn calculate_order(
    customer_id: i32,
    price1: i32,
    qty1: i32,
    price2: i32,
    qty2: i32,
    shipping_cents: i32,
) -> *const OrderResult {
    let line1 = calculate_line_item(price1, qty1);
    let line2 = calculate_line_item(price2, qty2);

    unsafe {
        if line1 < 0 || line2 < 0 {
            RESULT = OrderResult {
                status_code: 1,
                subtotal_cents: 0,
                discount_cents: 0,
                tax_cents: 0,
                shipping_cents: 0,
                total_cents: 0,
            };
        } else if shipping_cents < 0 {
            RESULT = OrderResult {
                status_code: 2,
                subtotal_cents: 0,
                discount_cents: 0,
                tax_cents: 0,
                shipping_cents: 0,
                total_cents: 0,
            };
        } else {
            let subtotal_cents = line1 + line2;
            let discount_rate = kv_get(customer_id);
            let discount_cents = subtotal_cents * discount_rate / 100;
            let base_imponible = subtotal_cents - discount_cents;
            let tax_cents = calculate_tax(base_imponible);
            let total_cents = base_imponible + tax_cents + shipping_cents;

            let credit_ok = check_credit(customer_id, total_cents);
            if credit_ok == 0 {
                RESULT = OrderResult {
                    status_code: 3,
                    subtotal_cents: 0,
                    discount_cents: 0,
                    tax_cents: 0,
                    shipping_cents: 0,
                    total_cents: 0,
                };
            } else {
                let event_id = if total_cents >= 10000 { 999 } else { 100 };
                emit_audit(event_id, total_cents);

                RESULT = OrderResult {
                    status_code: 0,
                    subtotal_cents,
                    discount_cents,
                    tax_cents,
                    shipping_cents,
                    total_cents,
                };
            }
        }
        &raw const RESULT as *const OrderResult
    }
}

