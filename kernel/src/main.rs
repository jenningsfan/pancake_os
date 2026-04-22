#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

extern crate alloc;

use bootloader_api::{BootInfo, BootloaderConfig, config::{Mapping}, entry_point};
use kernel::{memory::{self}, println};
use x86_64::{VirtAddr, structures::paging::{Page, PageTable, Translate}};
use alloc::{vec, boxed::Box};

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

const BUILD_TIME: &str = env!("BUILD_TIME");

fn recurse_print(table: &PageTable, level: u8, phys_offsest: u64) {
    for (i, entry) in table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L{level} Entry {i}: {entry:?}");
    
            if level != 1 {
    
                let phys = entry.frame().unwrap().start_address();
                let virt = phys.as_u64() + phys_offsest;
                let ptr = VirtAddr::new(virt).as_mut_ptr();
                let lower: &PageTable = unsafe { &*ptr };
                recurse_print(lower, level - 1, phys_offsest);
            }
        }
    }
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {    
    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());    
    let fb_addr = VirtAddr::from_ptr(boot_info.framebuffer.as_ref().unwrap().buffer().as_ptr());
    
    kernel::init(boot_info.into());
    //writeln!(SERIAL.lock(), "Entered kernel with boot info: {boot_info:?}").unwrap();
    
    println!("Welcome to PancakeOS.\nBuild time: {}", BUILD_TIME);
    
    //let x = Box::new(42);

    
    let mut v = vec![0xDEADBEEF; 768];
    
    let mut i: u64 = 0;
    println!("no crashing");

    loop {
        x86_64::instructions::hlt();
        for _ in 0..128 {
            v.push(i);
        }
        println!("v len: {}", v.len());
        println!("v addr: {:0X}", v.as_ptr().addr());
        i = i.wrapping_add(1);
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
