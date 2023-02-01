#![no_std]
#![no_main]

mod font5x8;
mod ssd_1306;
mod i2c_bus;
use embedded_utils::encoder;

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::entry;
use stm32f4xx_hal::gpio::PinState;
use stm32f4xx_hal::pac;
use stm32f4xx_hal::prelude::*;

fn u16_to_chars<const N: usize>(v: u16) -> [char; N]
{
    let mut v = v;
    let mut iter = (0..5).map(|_| {
        let d = (v%10) as u8;
        v = v/10;
        (d + 48) as char
   });
   let mut ret = ['0'; N];
   for v in ret.iter_mut().rev() {
        *v = iter.next().unwrap();
   }
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

    let key = gpioa.pa10.into_floating_input();

    // let s1 = gpioa.pa8.into_alternate_open_drain();
    // let s2 = gpioa.pa9.into_alternate_open_drain();
    // let encoder = dp.TIM1.qei((s1, s2));

    let mut enc = encoder::Encoder::new(
        gpioa.pa8.into_pull_up_input(),
        gpioa.pa9.into_pull_up_input()
    );

    let i2c = dp.I2C1.i2c((scl, sda), 500_000.Hz(), &clocks);
    let bus = i2c_bus::I2cBus::new(i2c);
    let mut oled = ssd_1306::Ssd1306::new(bus);
    oled.init();
    oled.clear();
    oled.write_str("Hello!");
    oled.set_pos(1,0);

    let mut ch: u8 = 0;
    let mut count = 0;
    let mut old_key = key.is_high();
    let mut old_but = but.is_low();
 
    fn update_ch<Bus>(ch: u8, oled: &mut ssd_1306::Ssd1306<Bus>)
    where
        Bus: ssd_1306::Bus
    {
        let pos = oled.get_pos();
        oled.set_pos(0, 6*8);
        oled.write_chars(u16_to_chars::<3>(ch as u16));
        oled.set_pos(pos.0, pos.1);
        oled.write_char(ch as char);
        oled.set_pos(pos.0, pos.1);
    }

    loop {
        match enc.update() {
            Ok(encoder::Direction::Positive) => {
                ch = if ch < u8::MAX {ch+1} else {u8::MIN};
                update_ch(ch, &mut oled);
            }
            Ok(encoder::Direction::Negative) => {
                ch = if ch > u8::MIN {ch-1} else {u8::MAX};
                update_ch(ch, &mut oled);
            }
            _ => ()
        };
        let key = key.is_high();
        if key && key != old_key {
        }
        old_key = key;
        
        let but = but.is_low();
        if but && but != old_but {
            oled.write_char(ch as char);
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
