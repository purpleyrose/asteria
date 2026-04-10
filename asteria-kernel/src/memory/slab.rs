use crate::memory;

#[repr(C)]
pub struct Slab {
    page: u64,
    object_size: u64,
    free_list: *mut u8,
}

impl Slab {
    pub fn new(page: u64, object_size: u64) -> Self {
        let count = 4096 / object_size;
        for i in 0..count {
            let slot = (page + i * object_size) as *mut u64;
            let next = if i + 1 < count {
                page + (i + 1) * object_size
            } else {
                0 // End of free list
            };
            unsafe {
                *slot = next;
            }
        }
        Self {
            page,
            object_size,
            free_list: page as *mut u8,
        }
    }

    pub fn allocate(&mut self) -> Option<*mut u8> {
        if self.free_list.is_null() {
            return None;
        }
        let obj = self.free_list;
        unsafe {
            self.free_list = *(obj as *mut u64) as *mut u8;
        }
        Some(obj)
    }

    pub fn free(&mut self, obj: *mut u8) {
        unsafe {
            *(obj as *mut u64) = self.free_list as u64;
        }
        self.free_list = obj;
    }
}

const SIZE_CLASSES: [u64; 7] = [32, 64, 128, 256, 512, 1024, 2048];

pub struct SlabAllocator {
    slabs: [Slab; SIZE_CLASSES.len()],
}

impl SlabAllocator {
    pub fn init(allocator: &mut memory::FrameAllocator) -> SlabAllocator {
        let slabs = core::array::from_fn(|i| {
            let page = allocator.allocate_page().expect("No page for slab");
            Slab::new(page, SIZE_CLASSES[i])
        });
        SlabAllocator { slabs }
    }

    pub fn allocate(&mut self, size: u64) -> Option<*mut u8> {
        for (i, &class_size) in SIZE_CLASSES.iter().enumerate() {
            if size <= class_size {
                return self.slabs[i].allocate();
            }
        }
        None // Size too large
    }

    pub fn free(&mut self, ptr: *mut u8, size: u64) {
        for (i, &class_size) in SIZE_CLASSES.iter().enumerate() {
            if size <= class_size {
                self.slabs[i].free(ptr);
                return;
            }
        }
        // Invalid size, ignore
    }
}
