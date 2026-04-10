#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use bootloader_api::{BootInfo, entry_point};
use kernel::println;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {    
    kernel::init(boot_info.into());
    //writeln!(SERIAL.lock(), "Entered kernel with boot info: {boot_info:?}").unwrap();

    println!("Hello World!");

    // let mut i = 0;

    // for _ in 0..100 {
    //     println!("{i}");
    //     i += 1;
    // }

    //x86_64::instructions::interrupts::int3(); // int 3

    // unsafe {
    //     *(0xdeadbe00 as *mut u8) = 42;
    // };

    println!("i'm still alive yupeee");

    loop {
        x86_64::instructions::hlt();
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {info}");
    loop {
        x86_64::instructions::hlt();
    }
}
