#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    unsafe {
        core::arch::asm!(
            "mov dx, 0x3FB",
            "mov al, 0x03",   // 8N1, clear DLAB
            "out dx, al",
            "mov dx, 0x3F8",
            "mov al, 0x4B",
            "out dx, al",
            out("dx") _,
            out("al") _,
        );
    }
    loop {}
}
