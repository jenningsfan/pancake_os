use x86_64::instructions::port::Port;

use crate::port_write_wait;

const INIT: u8 = 0x11;
const CASCADE: u8 = 0x02;
const MODE_8086: u8 = 0x01;
const EOI: u8 = 0x20;

pub struct Pic {
    master_command: Port<u8>,
    master_data: Port<u8>,
    slave_command: Port<u8>,
    slave_data: Port<u8>
}

impl Pic {
    pub const fn new() -> Self {
        Self {
            master_command: Port::new(0x20),
            master_data: Port::new(0x21),
            slave_command: Port::new(0xA0),
            slave_data: Port::new(0xA1)
        }
    }

    pub fn init(&mut self) {        
        unsafe {
            port_write_wait(&mut self.master_command, INIT);
            port_write_wait(&mut self.slave_command, INIT);

            port_write_wait(&mut self.master_data, 0x20); // remap to int 20
            port_write_wait(&mut self.slave_data, 0x28); // remap to int 28

            port_write_wait(&mut self.master_data, 1 << CASCADE);
            port_write_wait(&mut self.slave_data, CASCADE);

            port_write_wait(&mut self.master_data, MODE_8086);
            port_write_wait(&mut self.slave_data, MODE_8086);

            port_write_wait(&mut self.master_data, 0xFF);
            port_write_wait(&mut self.slave_data, 0xFF);
        }
    }

    pub unsafe fn mask_irq(&mut self, irq: u8) {
        unsafe {
            if irq < 8 {
                let mask = self.master_data.read();
                self.master_data.write(mask | (1 << irq));
            }
            else {
                let irq = irq - 8;
                let mask = self.slave_data.read();
                self.slave_data.write(mask | (1 << irq));
            }
        }
    }

    pub unsafe fn unmask_irq(&mut self, irq: u8) {
        unsafe {
            if irq < 8 {
                let mask = self.master_data.read();
                self.master_data.write(mask & !(1 << irq));
            }
            else {
                let irq = irq - 8;
                let mask = self.slave_data.read();
                self.slave_data.write(mask & !(1 << irq));
            }
        }
    }

    /// Unsafe as if wrong IRQ EOI'd then bad things happen
    pub unsafe fn eoi(&mut self, irq: u8) {
        unsafe {
            if irq >= 8 {
                self.slave_command.write(EOI);
            }
            self.master_command.write(EOI);
        }
    }
}