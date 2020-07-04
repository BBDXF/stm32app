// #![deny(unsafe_code)]
#![no_main]
#![no_std]

// Halt on panic
#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
extern crate panic_halt; // panic handler

use cortex_m;
use cortex_m_rt::entry;
use hal::gpio::{gpioa, gpiof, Alternate, Output, PushPull};
use hal::{interrupt, prelude::*, serial, stm32, timer};
use stm32f4xx_hal as hal;

static mut SER1: Option<
    serial::Serial<
        stm32::USART1,
        (
            gpioa::PA9<Alternate<hal::gpio::AF7>>,
            gpioa::PA10<Alternate<hal::gpio::AF7>>,
        ),
    >,
> = None;
static mut LED1: Option<gpiof::PF9<Output<PushPull>>> = None;
static mut LED2: Option<gpiof::PF10<Output<PushPull>>> = None;
static mut TM2: Option<timer::Timer<stm32::TIM2>> = None;
#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the system clock
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(168.mhz()).freeze();

        // Set up the LED.
        let gf = dp.GPIOF.split();
        unsafe {
            LED1 = Some(gf.pf9.into_push_pull_output());
            LED2 = Some(gf.pf10.into_push_pull_output());
        }

        // gpioa ahb1enr
        let ga = dp.GPIOA.split();

        // initial serial1
        // set apb1enr cr1 ...
        let mut ser1 = serial::Serial::usart1(
            dp.USART1,
            // pa9,pa10 AF config: afrl afrh moder
            (ga.pa9.into_alternate_af7(), ga.pa10.into_alternate_af7()),
            serial::config::Config::default().baudrate(115200.bps()),
            clocks,
        )
        .unwrap();
        // usart1 rx interrupt en
        ser1.listen(serial::Event::Rxne);

        unsafe {
            SER1 = Some(ser1);
        }

        // Set up a timer expiring after 1s
        let mut timer = timer::Timer::tim2(dp.TIM2, 1.hz(), clocks);

        // Generate an interrupt when the timer expires
        timer.listen(timer::Event::TimeOut);

        unsafe {
            TM2 = Some(timer);
        }

        // 开启NVIC总中断
        unsafe {
            stm32::NVIC::unmask(hal::interrupt::USART1);
            stm32::NVIC::unmask(hal::interrupt::TIM2);
        }
    }
    loop {}
}

#[interrupt]
fn USART1() {
    let ser1 = unsafe { SER1.as_mut().unwrap() };
    let led2 = unsafe { LED2.as_mut().unwrap() };
    let mut d1 = 0;
    if ser1.is_rxne() {
        d1 = ser1.read().unwrap();
        ser1.write(d1).unwrap();
    }
    if d1 > 128 {
        led2.set_high().unwrap();
    } else {
        led2.set_low().unwrap();
    }
}

#[interrupt]
fn TIM2() {
    static mut cur: u8 = 0;
    unsafe {
        let led1 = LED1.as_mut().unwrap();
        if *cur != 0 {
            led1.set_high().unwrap();
            *cur = 0;
        } else {
            led1.set_low().unwrap();
            *cur = 1;
        }
        let tm2 = TM2.as_mut().unwrap();
        tm2.clear_interrupt(timer::Event::TimeOut);
        tm2.start(1.hz());
    }
}
