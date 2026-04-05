#![no_std]
#![no_main]
mod serial;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    print!("Hello, Asteria Kernel!\n");
    loop {}
}

