use crate::println;
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

pub struct FrameAllocator {
    bitmap: *mut u8,    // Pointer to the bitmap
    total_pages: usize, // Total number of pages
}

pub fn init(memory_map: u64, memory_map_size: u64, descriptor_size: u64) -> FrameAllocator {
    // Find the largest type 7 region
    let mut largest_region: Option<&EfiMemoryDescriptor> = None;
    let count = memory_map_size / descriptor_size;
    for i in 0..count {
        let desc = unsafe { &*((memory_map + i * descriptor_size) as *const EfiMemoryDescriptor) };
        if desc.mem_type == 7 {
            if largest_region.is_none() || desc.num_pages > largest_region.unwrap().num_pages {
                largest_region = Some(desc);
            }
        }
    }
    let mut max_address: u64 = 0;
    for i in 0..count {
        let desc = unsafe { &*((memory_map + i * descriptor_size) as *const EfiMemoryDescriptor) };
        let end = desc.phys_start + desc.num_pages * 4096;
        if end > max_address {
            max_address = end;
        }
    }
    let total_pages = (max_address / 4096) as usize;
    let bitmap_size = (total_pages + 7) / 8; // Size in bytes

    let bitmap = unsafe {
        // Allocate bitmap in the largest type 7 region
        let region = largest_region.expect("No type 7 region found");
        let bitmap_addr = region.phys_start;
        bitmap_addr as *mut u8
    };

    // Zero the bitmap
    unsafe {
        core::ptr::write_bytes(bitmap, 0, bitmap_size);
    }
    // Walk the memory map again to mark free pages
    for i in 0..count {
        let desc = unsafe { &*((memory_map + i * descriptor_size) as *const EfiMemoryDescriptor) };
        if desc.mem_type == 7 {
            let start_page = (desc.phys_start / 4096) as usize;
            for p in 0..desc.num_pages as usize {
                let page = start_page + p;
                let byte_index = page / 8;
                let bit_index = page % 8;
                unsafe {
                    *bitmap.add(byte_index) |= 1 << bit_index; // Mark as used
                }
            }
        }
    }

    // Mark the bitmap itself as used
    let bitmap_start_page = (bitmap as u64 / 4096) as usize;
    let bitmap_pages = (bitmap_size + 4095) / 4096;
    for p in 0..bitmap_pages {
        let page = bitmap_start_page + p;
        let byte_index = page / 8;
        let bit_index = page % 8;
        unsafe {
            *bitmap.add(byte_index) &= !(1 << bit_index); // Mark as used
        }
    }
    // Reserve low 1MB
    let low_pages = 0x100000 / 4096; // 256 pages
    for p in 0..low_pages {
        let byte_index = p / 8;
        let bit_index = p % 8;
        unsafe {
            *bitmap.add(byte_index) &= !(1 << bit_index); // Mark as used
        }
    }

    FrameAllocator {
        bitmap,
        total_pages,
    }
}

impl FrameAllocator {
    pub fn allocate_page(&mut self) -> Option<u64> {
        for page in 0..self.total_pages {
            let byte_index = page / 8;
            let bit_index = page % 8;
            unsafe {
                if (*self.bitmap.add(byte_index) & (1 << bit_index)) != 0 {
                    // Mark as used
                    *self.bitmap.add(byte_index) &= !(1 << bit_index);
                    return Some((page as u64) * 4096);
                }
            }
        }
        None // No free pages
    }

    pub fn free_page(&mut self, addr: u64) {
        let page = (addr / 4096) as usize;
        let byte_index = page / 8;
        let bit_index = page % 8;
        unsafe {
            *self.bitmap.add(byte_index) |= 1 << bit_index; // Mark as free
        }
    }
}
