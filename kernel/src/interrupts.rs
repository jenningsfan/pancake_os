use x86_64::{
    instructions::interrupts::without_interrupts, registers::control::Cr2, structures::idt::{
        InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
    },
};
use lazy_static::lazy_static;
use spin::Mutex;
use crate::{gdt, pic8259::Pic, println, ps2::keyboard::KEYBOARD};

pub static PIC: Mutex<Pic> = Mutex::new(Pic::new());

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 0x20,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 { self as u8 }
    fn as_usize(self) -> usize { self as usize }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Core exceptions
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DF_IST_INDEX);
        }

        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault.set_handler_fn(gpf_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        idt.x87_floating_point.set_handler_fn(x87_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);

        // IRQs
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_handler);
        idt[0x27].set_handler_fn(irq7);

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

//
// === EXCEPTION HANDLERS ===
//

// No error code
extern "x86-interrupt" fn divide_error_handler(sf: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE ERROR\n{:#?}", sf);
}

extern "x86-interrupt" fn debug_handler(sf: InterruptStackFrame) {
    panic!("EXCEPTION: DEBUG\n{:#?}", sf);
}

extern "x86-interrupt" fn nmi_handler(sf: InterruptStackFrame) {
    panic!("EXCEPTION: NMI\n{:#?}", sf);
}

extern "x86-interrupt" fn breakpoint_handler(sf: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", sf);
}

extern "x86-interrupt" fn overflow_handler(sf: InterruptStackFrame) {
    panic!("EXCEPTION: OVERFLOW\n{:#?}", sf);
}

extern "x86-interrupt" fn bound_range_handler(sf: InterruptStackFrame) {
    panic!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", sf);
}

extern "x86-interrupt" fn invalid_opcode_handler(sf: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", sf);
}

extern "x86-interrupt" fn device_not_available_handler(sf: InterruptStackFrame) {
    panic!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", sf);
}

extern "x86-interrupt" fn x87_handler(sf: InterruptStackFrame) {
    panic!("EXCEPTION: X87 FLOATING POINT\n{:#?}", sf);
}

extern "x86-interrupt" fn machine_check_handler(sf: InterruptStackFrame) -> ! {
    panic!("EXCEPTION: MACHINE CHECK\n{:#?}", sf);
}

extern "x86-interrupt" fn simd_handler(sf: InterruptStackFrame) {
    panic!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", sf);
}

extern "x86-interrupt" fn virtualization_handler(sf: InterruptStackFrame) {
    panic!("EXCEPTION: VIRTUALIZATION\n{:#?}", sf);
}

//
// With error code
//

extern "x86-interrupt" fn double_fault_handler(
    sf: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", sf);
}

extern "x86-interrupt" fn invalid_tss_handler(
    sf: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: INVALID TSS\n{:#?}\nError: {:#x}", sf, error_code);
}

extern "x86-interrupt" fn segment_not_present_handler(
    sf: InterruptStackFrame,
    error_code: u64,
) {
    unsafe {
        let instr: u64 = *(sf.instruction_pointer).as_ptr();
        println!("I: {instr:04X}");
        let ret: u64 = *(sf.stack_pointer).as_ptr();
        println!("S: {ret:04X}");
    }
    panic!("EXCEPTION: SEGMENT NOT PRESENT\n{:#?}\nError: {:#x}", sf, error_code);
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    sf: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: STACK SEGMENT FAULT\n{:#?}\nError: {:#x}", sf, error_code);
}

extern "x86-interrupt" fn gpf_handler(
    sf: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\nError: {:#x}", sf, error_code);
}

extern "x86-interrupt" fn page_fault_handler(
    sf: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!(
        "EXCEPTION: PAGE FAULT\nAccessed: {:?}\nError: {:?}\n{:#?}",
        Cr2::read(),
        error_code,
        sf
    );
}

extern "x86-interrupt" fn alignment_check_handler(
    sf: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: ALIGNMENT CHECK\n{:#?}\nError: {:#x}", sf, error_code);
}

//
// IRQs
//

extern "x86-interrupt" fn timer_handler(_sf: InterruptStackFrame) {
    //print!(".");
    unsafe { PIC.lock().eoi(InterruptIndex::Timer.as_u8()); }
}

extern "x86-interrupt" fn keyboard_handler(_sf: InterruptStackFrame) {
    without_interrupts(|| {
        KEYBOARD.lock().irq_handler();
        unsafe { PIC.lock().eoi(InterruptIndex::Keyboard.as_u8()); }
    });
}

extern "x86-interrupt" fn irq7(_sf: InterruptStackFrame) {} // currently nothing is attached to IRQ 7 so it must be spurious so no EOI is needed