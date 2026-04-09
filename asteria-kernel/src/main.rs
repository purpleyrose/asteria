#![no_std]
#![no_main]
mod gdt;
mod idt;
mod memory;
mod serial;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(memory_map: u64, memory_map_size: u64, descriptor_size: u64) -> ! {
    gdt::load();
    idt::load();
    println!("Hello, Asteria Kernel!");
    println!(
        "Memory map: addr={:#x}, size={}, desc_size={}",
        memory_map, memory_map_size, descriptor_size
    );
    // memory::print_memory_map(memory_map, memory_map_size, descriptor_size);
    let mut allocator = memory::init(memory_map, memory_map_size, descriptor_size);
    if let Some(page) = allocator.allocate_page() {
        println!("Allocated page at: {:#x}", page);
    } else {
        println!("Failed to allocate page");
    }

    if let Some(page) = allocator.allocate_page() {
        println!("Allocated page at: {:#x}", page);
    } else {
        println!("Failed to allocate page");
    }

    loop {}
}
