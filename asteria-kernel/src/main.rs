#![no_std]
#![no_main]
extern crate alloc;
mod gdt;
mod idt;
mod memory;
mod serial;

use alloc::alloc::{alloc, dealloc};
use alloc::vec::Vec;
use core::alloc::Layout;

#[global_allocator]
static GLOBAL_ALLOCATOR: memory::allocator::KernelAllocator =
    memory::allocator::KernelAllocator::new();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

const SMOKE_N: usize = 300;

/// Count how many distinct 4096-byte page bases appear among `ptrs`.
///
/// Each slab is exactly one buddy page, so every distinct page base is a
/// separate slab the 32-byte class had to grow into. A result > 1 is direct
/// proof the grow path fired; with SMOKE_N = 300 and 127 objects per 32-byte
/// slab, we expect exactly 3.
fn distinct_page_count(ptrs: &[*mut u8]) -> usize {
    // Relies on the test filling slabs one after another with no frees in
    // between, so the page base only changes at a slab boundary.
    let mut count = 0;
    let mut last_page_base = 0;
    for &ptr in ptrs {
        let page_base = ptr as u64 & !0xFFF;
        if page_base != last_page_base {
            count += 1;
            last_page_base = page_base;
        }
    }

    count
}

fn slab_smoke_test() {
    let layout = Layout::from_size_align(32, 8).unwrap();
    let mut ptrs: [*mut u8; SMOKE_N] = [core::ptr::null_mut(); SMOKE_N];

    // 1. Hammer the 32-byte class hard enough to force several slabs.
    for i in 0..SMOKE_N {
        ptrs[i] = unsafe { alloc(layout) };
    }
    println!(
        "smoke: allocated {} x 32B; first={:p} last={:p}",
        SMOKE_N,
        ptrs[0],
        ptrs[SMOKE_N - 1]
    );
    println!(
        "smoke: spanned {} distinct slab page(s) (expect 3)",
        distinct_page_count(&ptrs)
    );

    // 2. LIFO recycling. The 3 most-recent allocations live in the current
    //    head slab, so frees there are immediately reusable. Free a, then b,
    //    then c; the next 3 allocations should return c, b, a (reverse order).
    let a = ptrs[SMOKE_N - 1];
    let b = ptrs[SMOKE_N - 2];
    let c = ptrs[SMOKE_N - 3];
    unsafe {
        dealloc(a, layout);
        dealloc(b, layout);
        dealloc(c, layout);
    }
    let r1 = unsafe { alloc(layout) };
    let r2 = unsafe { alloc(layout) };
    let r3 = unsafe { alloc(layout) };
    println!(
        "smoke: LIFO expect [{:p} {:p} {:p}] got [{:p} {:p} {:p}] -> {}",
        c,
        b,
        a,
        r1,
        r2,
        r3,
        if r1 == c && r2 == b && r3 == a {
            "PASS"
        } else {
            "FAIL"
        }
    );

    // 3. Free everything — exercises the O(1) `ptr & !0xFFF` path across all
    //    three slabs, not just the head.
    for i in 0..SMOKE_N - 3 {
        unsafe { dealloc(ptrs[i], layout) };
    }
    unsafe {
        dealloc(r1, layout);
        dealloc(r2, layout);
        dealloc(r3, layout);
    }
    println!("smoke: freed all; done");
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(memory_map: u64, memory_map_size: u64, descriptor_size: u64) -> ! {
    gdt::load();
    idt::load();

    println!("Hello, Asteria!");

    let (mut buddy_allocator, max_address) =
        memory::init(memory_map, memory_map_size, descriptor_size);
    memory::paging::init(&mut buddy_allocator, max_address);
    let slab_allocator = memory::slab::SlabAllocator::init();
    GLOBAL_ALLOCATOR.init(buddy_allocator, slab_allocator);

    let mut v: Vec<u32> = Vec::new();
    v.push(42);
    v.push(100);
    v.push(200);
    println!("Vector len: {}, first: {}", v.len(), v[0]);

    slab_smoke_test();

    loop {}
}
