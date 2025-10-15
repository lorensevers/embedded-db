#![no_main]
#![no_std]

pub mod codec;
pub mod db;
pub mod kv;

use defmt_rtt as _;

// I'm building this for the nRF52840 board - similar to the nRF52840 DK
// https://docs.nordicsemi.com/bundle/ncs-latest/page/zephyr/boards/nordic/nrf52840dk/doc/index.html

use nrf52840_hal as _;
use panic_probe as _;

// Panic handler - just trigger a UDF so it won't print a panic message
// We are using no_std, so we can't use the default panic handler
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use defmt::assert;

    #[test]
    fn it_works() {
        assert!(true)
    }
}

// This should run forever to keep the board on
// The cortex_m::asm::wfi() should keep the CPU in low power mode
// until an interrupt like a button press
pub fn idle_forever() -> ! {
    loop {
        cortex_m::asm::wfi()
    }
}
