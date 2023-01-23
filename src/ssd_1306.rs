use core::iter;
use crate::font5x8::FONT;
pub trait Bus {
    fn send_command(&mut self, cmd: &[u8]);
    fn send_data<B: IntoIterator<Item = u8>>(&mut self, iter: B);
}

pub struct Ssd1306<BusT: Bus> {
    bus: BusT,
    row: u8,
    col: u8
}

impl<BusT: Bus> Ssd1306<BusT> {
    pub fn new(bus: BusT) -> Ssd1306<BusT> {
        Ssd1306 {
            bus,
            row: 0,
            col: 0
        }
    }
    pub fn init(&mut self) {
        self.bus.send_command(&[0xAE]); // display off
        self.bus.send_command(&[0xA8, 0x1f]); // mux ratio
        self.bus.send_command(&[0x8D, 0x14]); // charge pump?
        self.bus.send_command(&[0xB3, 0x80]); // clock div?
        self.bus.send_command(&[0xD3, 0x00]); // vertical shift
        self.bus.send_command(&[0x40]); // display start line
        self.bus.send_command(&[0xA0]); // col 0 = SEG0
        self.bus.send_command(&[0xC0]); // scan dir normal
        self.bus.send_command(&[0xDA, 0x02]); // com pins
        self.bus.send_command(&[0xD9, 0xF1]); // pre charge
        self.bus.send_command(&[0xDB, 0x40]); // vcomh deselect level
        self.bus.send_command(&[0xA4]); // not all on
        self.bus.send_command(&[0xA6]); // not inverted display
        self.bus.send_command(&[0x81, 0x1F]); // contrast
        self.bus.send_command(&[0xAF]); // display on
    }    
    pub fn clear(&mut self) {
        for p in 0..4 {
            self.set_pos(p, 0);
            self.bus.send_data(iter::repeat(0).take(128));
        }
        self.set_pos(0,0);
    }
    fn set_page(&mut self)
    {
        self.bus.send_command(&[0x20, 0x02]); // page addr mode
        self.bus.send_command(&[self.col & 0x0f]); // low nibble
        self.bus.send_command(&[0x10 | ((self.col>>4) & 0x0f)]); // high nibble
        self.bus.send_command(&[0xB0 | (0x0f & self.row)]); // page start addr
    }
    pub fn set_pos(&mut self, row: u8, col: u8) {
        self.row = row;
        self.col = col;
        self.set_page();
    }
    pub fn write_ch(&mut self, ch: char) {
        self.set_page();
        let ch = ch as usize;
        self.bus.send_data(FONT[(ch*5)..(ch*5+5)].iter().cloned());
        self.col += 6;
        if self.col > 128-6 {
            self.col = 0;
            self.row += 1;
            if self.row >= 4 {
                self.row = 0;
            }
        }
    }

    pub fn write(&mut self, text: &str)
    {
        for ch in text.chars()
        {
            self.write_ch(ch);
        }
    }
}
