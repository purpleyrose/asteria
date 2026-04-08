static GDT: [u64; 3] = [
    0x0000000000000000, // null descriptor
    0x00af9a000000ffff, // code segment descriptor
    0x00cf92000000ffff, // data segment descriptor
];

#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

pub fn load() {
    let gdt_ptr = GdtPointer {
        limit: (core::mem::size_of_val(&GDT) - 1) as u16,
        base: &GDT as *const _ as u64,
    };

    unsafe {
        core::arch::asm!(
            "lgdt [{gdt}]",
            "push 0x08",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            gdt = in(reg) &gdt_ptr,
            tmp = out(reg) _,
        );
    }
}
