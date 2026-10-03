use crate::println;

pub mod allocator;
pub mod buddy;
pub mod paging;
pub mod slab;

#[repr(C)]
pub struct EfiMemoryDescriptor {
    pub mem_type: u32,
    pub padding: u32,
    pub phys_start: u64,
    pub virt_start: u64,
    pub num_pages: u64,
    pub attr: u64,
}

pub fn print_memory_map(memory_map: u64, memory_map_size: u64, descriptor_size: u64) {
    let count = memory_map_size / descriptor_size;
    for i in 0..count {
        let desc = unsafe { &*((memory_map + i * descriptor_size) as *const EfiMemoryDescriptor) };
        println!(
            "Type: {}, PhysStart: {:#x}, VirtStart: {:#x}, NumPages: {}, Attr: {:#x}",
            desc.mem_type, desc.phys_start, desc.virt_start, desc.num_pages, desc.attr
        );
    }
}

/// Parse the UEFI memory map, initialize a BuddyAllocator over the largest
/// EfiConventionalMemory (type 7) region, and return (allocator, max_address).
///
/// `max_address` is the highest physical address to identity-map in paging —
/// it should cover *all* memory regions (not just the free one), since the
/// kernel itself and the memory map live outside type 7.
pub fn init(
    memory_map: u64,
    memory_map_size: u64,
    descriptor_size: u64,
) -> (buddy::BuddyAllocator, u64) {
    let mut largest: Option<&EfiMemoryDescriptor> = None;
    let mut max_address = 0;
    let count = memory_map_size / descriptor_size;
    for i in 0..count {
        let desc = unsafe { &*((memory_map + i * descriptor_size) as *const EfiMemoryDescriptor) };
        let end_address = desc.phys_start + desc.num_pages * 4096;
        if desc.mem_type >= 1 && desc.mem_type <= 10 && end_address > max_address {
            max_address = end_address;
        }
        if desc.mem_type == 7 {
            match largest {
                None => largest = Some(desc),
                Some(best) if desc.num_pages > best.num_pages => largest = Some(desc),
                _ => {}
            }
        }
    }
    let mut buddy = buddy::BuddyAllocator::new();
    if let Some(region) = largest {
        unsafe {
            buddy.init(region.phys_start, region.num_pages * 4096);
        }
    }
    println!("Max address: {:#x}", max_address);
    (buddy, max_address)
}
