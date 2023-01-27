#![no_std]
#![no_main]

mod font5x8;
mod ssd_1306;
mod i2c_bus;

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::entry;
use stm32f4xx_hal::gpio::PinState;
use stm32f4xx_hal::pac;
use stm32f4xx_hal::i2c;
use stm32f4xx_hal::prelude::*;

fn u16_to_chars(v: u16) -> [char; 5]
{
    let mut v = v;
    let mut iter = (0..5).map(|_| {
        let d = (v%10) as u8;
        v = v/10;
        (d + 48) as char
   });
   let mut ret = [iter.next().unwrap(),
   iter.next().unwrap(),
   iter.next().unwrap(),
   iter.next().unwrap(),
   iter.next().unwrap()];
   ret.reverse();
   ret
}

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(25.MHz())
        .sysclk(50.MHz())
        .hclk(50.MHz())
        .pclk1(25.MHz())
        .freeze();

    let mut syst = cp.SYST;
    syst.set_clock_source(SystClkSource::Core); // == AHB == HCLK
    syst.set_reload(clocks.hclk().to_kHz() - 1);
    syst.clear_current();
    syst.enable_counter();

    let gpioc = dp.GPIOC.split();
    let gpiob = dp.GPIOB.split();
    let gpioa = dp.GPIOA.split();

    let mut led = gpioc.pc13.into_push_pull_output();
    let scl = gpiob.pb6.into_pull_up_input();
    let sda = gpiob.pb7.into_pull_up_input();
    let but = gpioa.pa0.into_pull_up_input();

    let s1 = gpioa.pa8.into_alternate_open_drain();
    let s2 = gpioa.pa9.into_alternate_open_drain();
    let key = gpioa.pa10.into_floating_input();

    let encoder = dp.TIM1.qei((s1, s2));

    let i2c = dp.I2C1.i2c((scl, sda), i2c::Mode::from(500_000.Hz()), &clocks);
    let bus = i2c_bus::I2cBus::new(i2c);
    let mut oled = ssd_1306::Ssd1306::new(bus);
    oled.init();
    oled.clear();
    oled.write_str("Hello!");

    let mut ch: u8 = 0;
    let mut count = 0;
    let mut old_enc = encoder.count();
    let mut old_key = key.is_high();
    let mut old_but = but.is_low();
    let mut pos = (1, 0);
    loop {
        let enc = encoder.count();
        let key = key.is_high();
        if key && key != old_key {
        }
        old_key = key;
        
        if enc != old_enc {
            oled.set_pos(0, 6*8);
            oled.write_chars(u16_to_chars(enc));
            old_enc = enc;
            oled.set_pos(pos.0, pos.1);

            ch = (enc & 0xff) as u8;

            oled.write_char(ch as char);
        }

        let but = but.is_low();
        if but && but != old_but {
            oled.set_pos(pos.0, pos.1);
            oled.write_char(ch as char);
            pos = oled.get_pos();
        }
        old_but = but;

        count = if count < 100 {
            count + 1
        } else {
             0 
        };

        led.set_state(PinState::from(but && count < 50));

        while !syst.has_wrapped() {}
    }
}
