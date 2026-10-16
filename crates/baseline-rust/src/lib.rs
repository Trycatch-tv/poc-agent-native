#![no_std]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Requirements: SHIP-001, SHIP-002, SHIP-003 (Spec v3)
#[no_mangle]
pub extern "C" fn calculate_shipping(subtotal_cents: i32) -> i32 {
    if subtotal_cents < 0 {
        -1
    } else if subtotal_cents >= 4000 {
        0
    } else {
        600
    }
}

/// Requirements: DISC-001, DISC-002, DISC-003 (Spec v3)
#[no_mangle]
pub extern "C" fn calculate_discount(subtotal_cents: i32, premium: i32) -> i32 {
    if subtotal_cents < 0 {
        -1
    } else if premium == 0 {
        0
    } else {
        subtotal_cents / 10
    }
}