#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod display;
pub mod psf;
pub mod interrupts;
pub mod gdt;
pub mod pic8259;
pub mod ps2;
pub mod memory;

use bootloader_api::BootInfo;
use lazy_static::lazy_static;
use spin::{Mutex, MutexGuard};
use x86_64::{VirtAddr, instructions::port::{Port, PortWrite}};

use crate::memory::{MMapFrameAllocator, allocator};
use crate::ps2::{controller::PS2_CONTROLLER, keyboard::KEYBOARD};

lazy_static! {
    pub static ref SERIAL: Mutex<uart_16550::SerialPort> = unsafe { uart_16550::SerialPort::new(0x3F8).into() };
}

pub struct Locked<A> {
    inner: Mutex<A>
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Self {
            inner: Mutex::new(inner)
        }
    }

    pub fn lock(&self) -> MutexGuard<A> {
        self.inner.lock()
    }
}

pub fn init(boot_info: &'static mut BootInfo) {
    gdt::init();
    interrupts::init_idt();
    interrupts::PIC.lock().init();
    SERIAL.lock().init();

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_offset) };
    let mut frame_allocator = unsafe {memory::MMapFrameAllocator::init(&boot_info.memory_regions)};

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initalistion failed");

    let fb: Option<&mut bootloader_api::info::FrameBuffer> = boot_info.framebuffer.as_mut();
    
    display::DISPLAY.call_once(|| {
        display::Display::new(fb).into()
    });
    
    display::WRITER.call_once(|| {
        display::TTY::new().expect("TTY should init").into()
    });
    
    display::DISPLAY.get().unwrap().lock().clear();
    
    unsafe {
        interrupts::PIC.lock().unmask_irq(0); // unmask timer
    }
    PS2_CONTROLLER.lock().init();
    KEYBOARD.lock().init();
    
    x86_64::instructions::interrupts::enable();
}

pub unsafe fn port_write_wait<T>(port: &mut Port<T>, value: T) where T: PortWrite {
    unsafe {
        Port::new(0x80).write(0x00 as u8); // wait
        port.write(value);
    }
}