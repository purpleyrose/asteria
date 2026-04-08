#![no_std]
#![no_main]
mod gdt;
mod idt;
mod serial;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    gdt::load();
    idt::load();
    println!("Hello, Asteria Kernel!");
    loop {}
}
