#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger


use stm32f4xx_hal as hal;

use crate::hal::{pac, prelude::*};
use cortex_m_rt::{entry};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();


    // pc13 is the user button
    // pa5 is the led
    let mut led = gpioa.pa5.into_push_pull_output();
    let button = gpioc.pc13;

    // Initialize LED to on or off
    led.set_low();

    loop {
        for _ in 0..1_000 {
            led.set_high();
        }
        for _ in 0..50_000 {
            led.set_low();
        }
    }
}
