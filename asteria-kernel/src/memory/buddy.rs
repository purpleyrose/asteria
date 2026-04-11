const MAX_ORDER: usize = 11; // 2^11 = 2048 pages = 8MB max block size

pub struct BuddyAllocator {
    free_lists: [*mut FreeBlock; MAX_ORDER],
    base: u64, // Starting physical address of managed memory
    size: u64,
}

#[repr(C)]
struct FreeBlock {
    next: *mut FreeBlock,
}

fn size_to_order(size: u64) -> usize {
    let mut order = 0;
    let mut s = 4096u64;
    while s < size && order < MAX_ORDER - 1 {
        s *= 2;
        order += 1
    }
    order
}

impl BuddyAllocator {
    pub fn new() -> Self {
        Self {
            free_lists: [core::ptr::null_mut(); MAX_ORDER],
            base: 0,
            size: 0,
        }
    }
    pub unsafe fn init(&mut self, base: u64, size: u64) {
        self.base = base;
        self.size = size;
        // Add the entire region as blocks of the largest order
        // If size isn't a power of 2, we break it into multiple blocks
        let max_block_size = 4096u64 << (MAX_ORDER - 1);
        let mut addr = base;
        let end = base + size;
        while addr + max_block_size <= end {
            let block = addr as *mut FreeBlock;
            unsafe {
                (*block).next = self.free_lists[MAX_ORDER - 1];
            }
            self.free_lists[MAX_ORDER - 1] = block;
            addr += max_block_size;
        }
        // Handle remaining memory if size isn't a multiple of max block size
        while addr < end {
            let remaining = end - addr;
            let mut order = MAX_ORDER - 1;
            while order > 0 && (4096u64 << order) > remaining {
                order -= 1;
            }
            let block_size = 4096u64 << order;
            if addr & (block_size - 1) != 0 {
                while order > 0 && addr & ((4096u64 << order) - 1) != 0 {
                    order -= 1;
                }
            }
            let block = addr as *mut FreeBlock;
            unsafe {
                (*block).next = self.free_lists[order];
            }
            self.free_lists[order] = block;
            addr += 4096u64 << order;
        }
    }
    pub fn allocate(&mut self, size: u64) -> Option<u64> {
        let target_order = size_to_order(size);
        for mut order in target_order..MAX_ORDER {
            if !self.free_lists[order].is_null() {
                // Found a block of the required order
                let block = self.free_lists[order];
                unsafe {
                    self.free_lists[order] = (*block).next; // Remove block from free list
                }
                while order > target_order {
                    order -= 1;
                    let buddy = block as usize + (4096 << order);
                    let buddy_block = buddy as *mut FreeBlock;
                    unsafe {
                        (*buddy_block).next = self.free_lists[order];
                    }
                    self.free_lists[order] = buddy_block; // Add buddy to free list
                }
                return Some(block as u64);
            }
        }
        None // No suitable block found
    }
}
