#[derive(Copy, Clone)]
#[repr(C, packed)]

struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist_offset: u8,
    attributes: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    ist_offset: 0,
    attributes: 0,
    offset_mid: 0,
    offset_high: 0,
    reserved: 0,
}; 256];

fn set_handler(vector: usize, handler: u64) {
    unsafe {
        IDT[vector] = IdtEntry {
            offset_low: handler as u16,
            selector: 0x08, // code segment selector
            ist_offset: 0,
            attributes: 0x8E, // present, DPL=0, interrupt gate
            offset_mid: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            reserved: 0,
        }
    }
}

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

pub fn load() {
    set_handler(0, isr_0 as *const () as u64);
    set_handler(6, isr_6 as *const () as u64);
    set_handler(8, isr_8 as *const () as u64);
    set_handler(13, isr_13 as *const () as u64);
    set_handler(14, isr_14 as *const () as u64);
    let idt_ptr = IdtPointer {
        limit: (256 * core::mem::size_of::<IdtEntry>() - 1) as u16,
        base: &raw const IDT as u64,
    };
    unsafe {
        core::arch::asm!(
            "lidt [{idt}]",
            idt = in(reg) &idt_ptr,
        );
    }
}

unsafe extern "C" fn exception_handler(frame: *const InteruptFrame) {
    let frame = unsafe { &*frame };
    crate::println!(
        "EXCEPTION: {} (vector {}), error_code={:#x}, rip={:#x}",
        exception_name(frame.vector),
        frame.vector,
        frame.error_code,
        frame.rip
    );
    loop {}
}

fn exception_name(vector: u64) -> &'static str {
    match vector {
        0 => "Divide-by-zero",
        6 => "Invalid opcode",
        8 => "Double fault",
        13 => "General protection fault",
        14 => "Page fault",
        _ => "Unknown exception",
    }
}

#[repr(C)]
struct InteruptFrame {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rbp: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    vector: u64,
    error_code: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

macro_rules! isr_no_error {
    ($name:ident, $vector:expr) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            core::arch::naked_asm!(
                "push 0", // Push dumm error code
                "push {vector}",
                "jmp {common}",
                vector = const $vector,
                common = sym common_handler_stub,
        );
        }
    };
}

macro_rules! isr_with_error {
    ($name:ident, $vector:expr) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            core::arch::naked_asm!(
                "push {vector}",
                "jmp {common}",
                vector = const $vector,
                common = sym common_handler_stub,
        );
        }
    };
}

isr_no_error!(isr_0, 0); // Divide-by-zero exception
isr_no_error!(isr_6, 6); // Invalid opcode exception
isr_with_error!(isr_8, 8); // Double fault exception
isr_with_error!(isr_13, 13); // General protection fault
isr_with_error!(isr_14, 14); // Page fault

#[unsafe(naked)]
unsafe extern "C" fn common_handler_stub() {
    core::arch::naked_asm!(
        // Push all 15 GP registers
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov rdi, rsp",
        "call {handler}",
        // Pop all registers in reverse order
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        "add rsp, 16", // Clean up error code and vector
       "iretq",
        handler = sym exception_handler,
    );
}
