use crate::ssd_1306;
use core::iter;
use core::fmt::Debug;
use embedded_hal::blocking::i2c::WriteIter;

pub struct I2cBus<I, E>
where
    I: WriteIter<Error = E>,
    E: Debug
{
    addr: u8,
    i2c: I
}

impl<I, E> I2cBus<I, E>
where
    I: WriteIter<Error = E>,
    E: Debug
{
    pub fn new(i2c: I) -> I2cBus<I, E> {
        I2cBus {
            addr: 0x3c,
            i2c
        }
    }
}

impl<I, E> ssd_1306::Bus for I2cBus<I, E>
where
    I: WriteIter<Error = E>,
    E: Debug
{
    fn send_command(&mut self, cmd: &[u8]) {
        let data = iter::once(0).chain(cmd.iter().cloned());
        self.i2c.write(self.addr, data).unwrap();
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
        self.i2c.write(self.addr, data).unwrap();
    }
}
