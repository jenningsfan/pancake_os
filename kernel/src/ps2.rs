use core::panic;

use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};
use bitflags::bitflags;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::println;

lazy_static!{
    pub static ref PS2: Mutex<PS2Controller> = Mutex::new(PS2Controller::new());
}

bitflags! {
    #[derive(Debug)]
    struct StatusReg: u8 {
        const OutBufStatus = 1;
        const InBufStatus = 1 << 1;
        const SysFlag = 1 << 2;
        const DataForController = 1 << 3; // clear = data to in buffer is for PS/2 device, set = data for PS/2 controller command
        // bit 4 unknown
        // bit 5 unknown
        const TimeoutError = 1 << 6;
        const ParityError = 1 << 7;
    }
}

bitflags! {
    struct ControllerConfig: u8 {
        const Port1IRQ = 1;
        const Port2IRQ = 1 << 1;
        const SysFlag = 1 << 2;
        // bit 3 unused (should be 0)
        const Port1ClockDisabled = 1 << 4;
        const Port2ClockDisabled = 1 << 5;
        const Port1Translation = 1 << 6;
        // bit 7 unsude (should be 0)
    }
}

#[repr(u8)]
enum Command {
    ReadControllerConfig = 0x20,
    WriteControllerConfig = 0x60,
    DisablePort2 = 0xA7,
    EnablePort2 = 0xA8,
    TestPort2 = 0xA9,
    TestController = 0xAA,
    TestPort1 = 0xAB,
    DisablePort1 = 0xAD,
    EnablePort1 = 0xAE,
}

pub struct PS2Controller {
    data_port: Port<u8>,
    status_reg: PortReadOnly<u8>,
    command_reg: PortWriteOnly<u8>,
    port1: bool,
    port2: bool,
}

impl PS2Controller {
    pub fn new() -> Self {
        Self {
            data_port: Port::new(0x60),
            status_reg: PortReadOnly::new(0x64),
            command_reg: PortWriteOnly::new(0x64),
            port1: false,
            port2: false,
        }
    }

    pub fn init(&mut self) {
        const SELF_TEST_SUCCEDED: u8 = 0x55;

        x86_64::instructions::interrupts::without_interrupts(|| {
            unsafe {
                // disable devices so they don't interfere
                self.write_command(Command::DisablePort1 as u8);
                self.write_command(Command::DisablePort2 as u8);

                // flush output buffer
                self.data_port.read();
                
                // set controller config
                self.write_command(Command::ReadControllerConfig as u8);
                let mut controller_config = ControllerConfig::from_bits_truncate(self.read_and_wait());
                controller_config.remove(ControllerConfig::Port1IRQ);
                controller_config.remove(ControllerConfig::Port1ClockDisabled);
                controller_config.remove(ControllerConfig::Port1Translation);
                self.write_command_val(Command::WriteControllerConfig as u8, controller_config.bits());

                // controller self test
                self.write_command(Command::TestController as u8);
                let r = self.read_and_wait();
                if r != SELF_TEST_SUCCEDED {
                    panic!("PS/2 controller self test failed with {r:02X}");
                }

                // determine if there are 2 channels
                self.write_command(Command::EnablePort2 as u8);
                self.write_command(Command::ReadControllerConfig as u8);
                let mut controller_config = ControllerConfig::from_bits_truncate(self.read_and_wait());
                if !controller_config.contains(ControllerConfig::Port2ClockDisabled) {
                    self.port2 = true;
                    controller_config.remove(ControllerConfig::Port2IRQ);
                }
                else {
                    println!("PS/2: Port 2 does not exist")
                }

                self.write_command(Command::DisablePort2 as u8);
                self.write_command_val(Command::WriteControllerConfig as u8, controller_config.bits());

                self.write_command(Command::TestPort1 as u8);
                self.port1 = match self.read_and_wait() {
                    0x00 => { println!("PS/2: Port 1 test success"); true },
                    0x01 => { println!("PS/2: Port 1 failed due to clock line stuck low"); false },
                    0x02 => { println!("PS/2: Port 1 failed due to clock line stuck high"); false },
                    0x03 => { println!("PS/2: Port 1 failed due to data line stuck low"); false },
                    0x04 => { println!("PS/2: Port 1 failed due to data line stuck high"); false },
                    r@ _ => { println!("PS/2: Port 1 failed for an unknown reason: {r:02X}"); false }
                };

                self.write_command(Command::TestPort2 as u8);
                self.port2 = match self.read_and_wait() {
                    0x00 => { println!("PS/2: Port 2 test success"); true },
                    0x01 => { println!("PS/2: Port 2 failed due to clock line stuck low"); false },
                    0x02 => { println!("PS/2: Port 2 failed due to clock line stuck high"); false },
                    0x03 => { println!("PS/2: Port 2 failed due to data line stuck low"); false },
                    0x04 => { println!("PS/2: Port 2 failed due to data line stuck high"); false },
                    r@ _ => { println!("PS/2: Port 2 failed for an unknown reason: {r:02X}"); false }
                };
            }
        })
    }

    unsafe fn wait_for_input_buffer_write(&mut self) {
        unsafe {
            while StatusReg::from_bits_truncate(self.status_reg.read())
            .contains(StatusReg::InBufStatus) { }
        }
    }

    unsafe fn wait_for_input_buffer_read(&mut self) {
        unsafe {
            while !StatusReg::from_bits_truncate(self.status_reg.read())
                .contains(StatusReg::OutBufStatus) { }
        }
    }

    unsafe fn read_and_wait(&mut self) -> u8 {
        unsafe {
            self.wait_for_input_buffer_read();
            self.data_port.read()
        }
    }

    unsafe fn write_command(&mut self, command: u8) {
        // poll until input buffer is clear
        unsafe {
            self.wait_for_input_buffer_write();
            self.command_reg.write(command);
        }
    }

    unsafe fn write_command_val(&mut self, command: u8, value: u8) {
        unsafe {
            self.wait_for_input_buffer_write();
            self.command_reg.write(command);
    
            self.wait_for_input_buffer_write();
            self.data_port.write(value);
        }
    }
}