//! Uses the timer interrupt to blink a led with different frequencies.
//!
//! This assumes that a LED is connected to pc13 as is the case on the blue pill board.
//!
//! Note: Without additional hardware, PC13 should not be used to drive an LED, see page 5.1.2 of
//! the reference manual for an explanation. This is not an issue on the blue pill.

#![no_std]
#![no_main]


mod deque;

// you can put a breakpoint on `rust_begin_unwind` to catch panics
use panic_probe as _;

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [SDIO])]
mod app {
    use stm32f4xx_hal::{
        gpio::{gpioa::PA5, Output, PinState, PushPull},
        pac::{USART1, TIM3},
        prelude::*,
        timer::{MonoTimer64Us, CounterMs, Event},
        serial::{config::Config, Event::Rxne, Serial},
    };
    use rtt_target::{rtt_init_print, rprintln, rprint};
    use rustibus::RustIBus::{IBusMsg, popIBusMsg};
    use crate::deque::{Deque, Writer};
    use heapless::spsc::{Queue, Consumer, Producer};

    const CAPACITY: usize = 0x40;


    static mut Q: Queue<u8, CAPACITY> = Queue::new();
    static mut buf: Deque<CAPACITY> = Deque::new();


    #[shared]
    struct Shared { }

    #[local]
    struct Local {
        led: PA5<Output<PushPull>>,
        usart1: Serial<USART1>
    }

    #[monotonic(binds = TIM3, default = true)]
    type MicrosecMono = MonoTimer64Us<TIM3>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let rcc = cx.device.RCC.constrain();

        // Freeze the configuration of all the clocks in the system and store the frozen frequencies
        // in `clocks`
        let clocks = rcc.cfgr.use_hse(8.MHz()).sysclk(180.MHz()).freeze();


        // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the
        // function in order to configure the port. For pins 0-7, crl should be passed instead
        let mut gpioa = cx.device.GPIOA.split();
        let led = gpioa
            .pa5
            .into_push_pull_output();

        // serial
        let pins = (gpioa.pa9, gpioa.pa10);
        let mut serial = Serial::new(
            cx.device.USART1,
            pins,
            Config::default().baudrate(115_200.bps()).wordlength_8(),
            &clocks,
        )
            .unwrap()
            .with_u8_data();
        serial.listen(Rxne);

        // Timer
        let mono = cx.device.TIM3.monotonic64_us(&clocks);
        // tick::spawn().ok();

        rtt_init_print!();
        rprintln!("Hello, world!");


        // Init the static resources to use them later through RTIC
        (Shared { },
         Local { led:led, usart1: serial},
         init::Monotonics(mono))
    }

    #[task(binds = USART1, priority = 1, local = [led, usart1])]
    fn usart1(mut cx: usart1::Context) {
        cx.local.led.set_high();
        match cx.local.usart1.read() {
            Ok(ch) => {
                cx.local.led.toggle();
                let mut producer = unsafe { Q.split().0 };
                producer.enqueue(ch);
                parse::spawn();
            },
            _ => ()
        }
        cx.local.led.set_low();
    }


    // #[task(local = [led])]
    // fn tick(mut cx: tick::Context) {
    //     tick::spawn_after(1_u64.secs()).ok();
    //     // Depending on the application, you could want to delegate some of the work done here to
    //     // the idle task if you want to minimize the latency of interrupts with same priority (if
    //     // you have any). That could be done
    //     rprintln!("tick");
    //     cx.local.led.toggle();
    //     let mut producer = unsafe { Q.split().0 };
    //     for ch in [0x20, 0x40, 0xDB, 0x05, 0xDC, 0x05, 0x54, 0x05,
    //         0xDC, 0x05, 0xE8, 0x03, 0xD0, 0x07, 0xD2, 0x05,
    //         0xE8, 0x03, 0xDC, 0x05, 0xDC, 0x05, 0xDC, 0x05,
    //         0xDC, 0x05, 0xDC, 0x05, 0xDC, 0x05, 0xDA, 0xF3] {
    //         producer.enqueue(ch);
    //     }
    //     parse::spawn();
    // }

    #[task]
    fn parse(mut cx: parse::Context ) {
        let mut consumer = unsafe { Q.split().1 };

        while consumer.ready() {
            let ch = unsafe{consumer.dequeue_unchecked()};
            unsafe{ buf.push(ch) };
        }
        while unsafe {!buf.is_empty()} {
            let (mut msg, step) = unsafe{ popIBusMsg(&buf) };
            match msg {
                Some(m) => {
                    match m{
                        IBusMsg::SetMsg(data) => rprintln!("data: {:?}", data),
                        _ => ()
                    }
                },
                None => ()
            };
            if step == 0 { break; }
            for _ in 0..step {
                _ = unsafe { buf.pop() };
            }
        }
    }
}
