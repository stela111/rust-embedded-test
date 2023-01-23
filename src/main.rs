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

//use cortex_m::asm;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::entry;
use stm32f4xx_hal::gpio::PinState;
use stm32f4xx_hal::pac;
use stm32f4xx_hal::i2c;
use stm32f4xx_hal::prelude::*;

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
    let mut led = gpioc.pc13.into_push_pull_output();

    let gpiob = dp.GPIOB.split();
    let scl = gpiob.pb6.into_pull_up_input();
    let sda = gpiob.pb7.into_pull_up_input();

    let gpioa = dp.GPIOA.split();
    let but = gpioa.pa0.into_pull_up_input();

    let i2c = dp.I2C1.i2c((scl, sda), i2c::Mode::from(500_000.Hz()), &clocks);
    let bus = i2c_bus::I2cBus::new(i2c);
    let mut oled = ssd_1306::Ssd1306::new(bus);
    oled.init();
    oled.clear();
    oled.write("Hello!");
    oled.set_pos(1, 0);

    let mut ch: u8 = 0;
    let mut count = 0;
    loop {
        count = match but.is_low() {
            true if count < 100 => count + 1,
            true => {
                oled.write_ch(ch as char);
                ch = if ch == 255 { 0 } else { ch+1 };
                0
            },
            false => 0
        };
        led.set_state(PinState::from(count < 50));

        while !syst.has_wrapped() {}
    }
}
