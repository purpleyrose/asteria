#![no_std]
#![no_main]
mod gdt;
mod idt;
mod memory;
mod paging;
mod serial;
mod slab;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(memory_map: u64, memory_map_size: u64, descriptor_size: u64) -> ! {
    gdt::load();
    idt::load();

    println!("Hello, Asteria!");

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

    paging::init(&mut allocator);

    println!("Paging initalized");

    let slab_page = allocator
        .allocate_page()
        .expect("Failed to allocate page for slab");
    let mut slab = slab::Slab::new(slab_page, 64);
    if let Some(obj) = slab.allocate() {
        println!("Allocated object at: {:#x}", obj as u64);
    } else {
        println!("Failed to allocate object from slab");
    }

    let mut slab_alloc = slab::SlabAllocator::init(&mut allocator);
    if let Some(p) = slab_alloc.allocate(100) {
        println!("Allocated 100 bytes at: {:#x}", p as u64);
    } else {
        println!("Failed to allocate 100 bytes from slab allocator");
    }

    if let Some(p) = slab_alloc.allocate(2000) {
        println!("Allocated 2000 bytes at: {:#x}", p as u64);
    } else {
        println!("Failed to allocate 2000 bytes from slab allocator");
    }

    if let Some(p) = slab_alloc.allocate(3000) {
        println!("Allocated 3000 bytes at {:#x}", p as u64);
    } else {
        println!("Failed to allocate 3000 bytes from slab allocator");
    }

    loop {}
}
