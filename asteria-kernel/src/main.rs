#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!(
            "mov dx, 0x3F8",
            "mov al, {c}",
            "out dx, al",
            c = const b'K',
            out("dx") _,
            out("al") _,
        );

        // PL011 UART on QEMU virt machine
        #[cfg(target_arch = "aarch64")]
        {
            let uart = 0x09000000 as *mut u8;
            uart.write_volatile(b'K');
        }
    }
    loop {}
}
