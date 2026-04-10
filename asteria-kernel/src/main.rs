#![no_std]
#![no_main]
extern crate alloc;
mod gdt;
mod idt;
mod memory;
mod serial;

use alloc::vec::Vec;

#[global_allocator]
static GLOBAL_ALLOCATOR: memory::allocator::KernelAllocator =
    memory::allocator::KernelAllocator::new();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(memory_map: u64, memory_map_size: u64, descriptor_size: u64) -> ! {
    gdt::load();
    idt::load();

    println!("Hello, Asteria!");

    let mut frame_allocator = memory::init(memory_map, memory_map_size, descriptor_size);
    memory::paging::init(&mut frame_allocator);
    let slab_allocator = memory::slab::SlabAllocator::init(&mut frame_allocator);
    GLOBAL_ALLOCATOR.init(frame_allocator, slab_allocator);

    let mut v: Vec<u32> = Vec::new();
    v.push(42);
    v.push(100);
    v.push(200);
    println!("Vector len: {}, first: {}", v.len(), v[0]);

    loop {}
}
