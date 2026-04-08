#![no_std]
#![feature(abi_x86_interrupt)]

pub mod display;
pub mod psf;
pub mod interrupts;
pub mod gdt;

use bootloader_api::BootInfo;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref SERIAL: Mutex<uart_16550::SerialPort> = unsafe { uart_16550::SerialPort::new(0x3F8).into() };
}

pub fn init(boot_info: &'static mut BootInfo) {
    gdt::init();
    interrupts::init_idt();
    SERIAL.lock().init();
    
    let fb: Option<&mut bootloader_api::info::FrameBuffer> = boot_info.framebuffer.as_mut();
    
    display::DISPLAY.call_once(|| {
        display::Display::new(fb).into()
    });

    display::WRITER.call_once(|| {
        display::TTY::new().expect("TTY should init").into()
    });
}