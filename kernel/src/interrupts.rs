use x86_64::{VirtAddr, registers::control::Cr2, structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode}};
use lazy_static::lazy_static;
use spin::Mutex;
use crate::{gdt, pic8259::Pic, print, println, ps2::keyboard::KEYBOARD};

pub static PIC: Mutex<Pic> = Mutex::new(Pic::new());

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 0x20,
    Keyboard
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        self as usize
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        
        unsafe {
            idt.general_protection_fault.set_handler_addr(VirtAddr::from_ptr(gpf_handler as *const fn(stack_frame: InterruptStackFrame, _error_code:u64)));
            idt.double_fault.set_handler_addr(VirtAddr::from_ptr(double_fault_handler as *const fn(stack_frame: InterruptStackFrame, _error_code:u64)))
            //idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DF_IST_INDEX);
        } // safe as stack index is valid and used only for DF

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_handler);
        
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn gpf_handler(stack_frame: InterruptStackFrame) -> !
{
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    panic!("EXCEPTION: PAGE FAULT\nAccessed address: {:?}\nError code: {:?}\nStack frame:\n{:#?}", Cr2::read(), error_code, stack_frame);
}

extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    //print!(".");
    unsafe {
        PIC.lock().eoi(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    KEYBOARD.lock().irq_handler();
    unsafe {
        PIC.lock().eoi(InterruptIndex::Keyboard.as_u8());
    }
}

// extern "x86-interrupt" fn tss_invalid_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
//     panic!("EXCEPTION: INVALID TSS");
// }