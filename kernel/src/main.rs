#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use lazy_static::lazy_static;
use spin::Mutex;
use bootloader_api::{BootInfo, entry_point};
use core::fmt::Write;
use kernel::{display::{DISPLAY, Display, TTY, WRITER}, println, display};
use kernel::SERIAL;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    SERIAL.lock().init();
    writeln!(SERIAL.lock(), "Entered kernel with boot info: {boot_info:?}").unwrap();
    
    let fb: Option<&mut bootloader_api::info::FrameBuffer> = boot_info.framebuffer.as_mut();
    
    DISPLAY.call_once(|| {
        Display::new(fb).into()
    });

    display!().clear();

    WRITER.call_once(|| {
        TTY::new().expect("TTY should init").into()
    });

    println!("Hello World!");

    let mut i = 0;

    for _ in 0..100 {
        println!("{i}");
        i += 1;
    }

    loop {}
}

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {info}");
    loop {}
}
