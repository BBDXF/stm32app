#![no_std]
#![no_main]

extern crate stm32f4;
extern crate panic_halt;
extern crate cortex_m_rt;

use cortex_m_rt::entry;
use stm32f4::stm32f407;

// use `main` as the entry point of this application
#[entry]
fn main() -> ! {
    // get handles to the hardware
    let peripherals = stm32f407::Peripherals::take().unwrap();

    let pf = &peripherals.GPIOF;
    let rcc = &peripherals.RCC;

    // enable system clock RCC for gpiof
    rcc.ahb1enr.write(|w|{
        w.gpiofen().set_bit()
    });

    // config GPIOF pin9 and pin10 for led
    {
        // 1. mode 01 通用输出
        pf.moder.write(|w|{
            w.moder9().output()
                .moder10().output()
        });
        // 2. otype 0 推挽输出
        pf.otyper.write(|w|{
            w.ot9().push_pull()
                .ot10().push_pull()
        });
        // 3. ospeed 10, 50MHz high speed
        pf.ospeedr.write(|w|{
            w.ospeedr9().high_speed()
                .ospeedr10().high_speed()
        });
        // 4. pup 01， 上拉
        pf.pupdr.write(|w|{
            w.pupdr9().pull_up()
                .pupdr10().pull_up()
        });
        // 5. idr/odr/bsrr
        // by condition to read or set
        pf.odr.reset();
    }

    loop{
        // pf.odr.write(|w| {
        //     w.odr9().set_bit()
        //         .odr10().clear_bit()
        // });
        pf.bsrr.write(|w|{
            w.bs9().set_bit()
                .br10().set_bit()
        });
        cortex_m::asm::delay(168*10000*3);
        // pf.odr.write(|w| {
        //     w.odr9().clear_bit()
        //         .odr10().set_bit()
        // });
        pf.bsrr.write(|w|{
            w.br9().set_bit()
                .bs10().set_bit()
        });
        cortex_m::asm::delay(168*10000*3);
    }
}