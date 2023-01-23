use crate::ssd_1306;
use core::iter;
use stm32f4xx_hal::i2c;

pub struct I2cBus<I:i2c::Instance,P> {
    addr: u8,
    i2c: i2c::I2c<I,P>
}

impl<I:i2c::Instance,P> I2cBus<I,P> {
    pub fn new(i2c: i2c::I2c<I,P>) -> I2cBus<I,P> {
        I2cBus {
            addr: 0x3c,
            i2c: i2c
        }
    }
}

impl<I:i2c::Instance,P> ssd_1306::Bus for I2cBus<I,P> {
    fn send_command(&mut self, cmd: &[u8]) {
        let data = iter::once(0).chain(cmd.iter().cloned());
        self.i2c.write_iter(self.addr, data).unwrap();
    }
    // fn send_data(&mut self, data: &[u8]) {
    //     let head = &[0x40];
    //     let data = head.iter().chain(data.iter()).map(|&v| v);
    //     self.i2c.write_iter(self.addr, data).unwrap();
    // }
    fn send_data<D>(&mut self, data: D) 
    where
        D: IntoIterator<Item = u8>,
    {
        let data = iter::once(0x40).chain(data);
        self.i2c.write_iter(self.addr, data).unwrap();
    }
}
