//! Uses the timer interrupt to blink a led with different frequencies.
//!
//! This assumes that a LED is connected to pc13 as is the case on the blue pill board.
//!
//! Note: Without additional hardware, PC13 should not be used to drive an LED, see page 5.1.2 of
//! the reference manual for an explanation. This is not an issue on the blue pill.

#![no_std]
#![no_main]

mod ibusbm;
mod deque;

// you can put a breakpoint on `rust_begin_unwind` to catch panics
use panic_probe as _;

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1])]
mod app {
    use fugit::ExtU64;
    use stm32f4xx_hal::{
        gpio::{gpioa::PA5, Output, PinState, PushPull},
        pac,
        prelude::*,
        timer::{MonoTimer64Us, CounterMs, Event},
    };
    use rtt_target::{rtt_init_print, rprintln};
    use crate::ibusbm::*;
    use crate::deque::Deque;

    const CAPACITY: usize = 0x40;

    #[shared]
    struct Shared<'a> {
        deque: Deque<u8, CAPACITY>
    }

    #[local]
    struct Local {
        led: PA5<Output<PushPull>>,
    }

    #[monotonic(binds = TIM3, default = true)]
    type MicrosecMono = MonoTimer64Us<pac::TIM3>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let rcc = cx.device.RCC.constrain();

        // Freeze the configuration of all the clocks in the system and store the frozen frequencies
        // in `clocks`
        let clocks = rcc.cfgr.freeze();


        // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the
        // function in order to configure the port. For pins 0-7, crl should be passed instead
        let mut gpioa = cx.device.GPIOA.split();
        let led = gpioa
            .pa5
            .into_push_pull_output();

        let mono = cx.device.TIM3.monotonic64_us(&clocks);
        tick::spawn().ok();

        rtt_init_print!();
        rprintln!("Hello, world!");

        // Init the static resources to use them later through RTIC
        (Shared { deque: Deque::new() }, Local { led:led }, init::Monotonics(mono))
    }

    #[task(local = [led], shared=[deque])]
    fn tick(mut cx: tick::Context) {
        tick::spawn_after(1_u64.secs()).ok();
        // Depending on the application, you could want to delegate some of the work done here to
        // the idle task if you want to minimize the latency of interrupts with same priority (if
        // you have any). That could be done
        rprintln!("tick");
        cx.local.led.toggle();
        cx.shared.deque.lock(|deque| {
            for ch in [0x20, 0x40, 0xDB, 0x05, 0xDC, 0x05, 0x54, 0x05,
                0xDC, 0x05, 0xE8, 0x03, 0xD0, 0x07, 0xD2, 0x05,
                0xE8, 0x03, 0xDC, 0x05, 0xDC, 0x05, 0xDC, 0x05,
                0xDC, 0x05, 0xDC, 0x05, 0xDC, 0x05, 0xDA, 0xF3] {
                deque.push(ch);
            }
        });
        parse::spawn().unwrap();
    }

    #[task(shared=[deque])]
    fn parse(mut cx: parse::Context ) {
        let mut msg: Option<IBusMsg> = None;
        let mut step = 0u8;
        cx.shared.deque.lock(|deque| {
            (msg, step) = popIBusMsg(deque);
        });
        match msg {
            Some(m) => rprintln!("Received a message"),
            None => ()
        };
    }
}