use crate::memory::buddy::BuddyAllocator;

/// One slab = one 4096-byte page from the buddy. The `Slab` header lives at the
/// very start of that page (so it is 4096-aligned); the rest of the page is
/// divided into `object_size` slots threaded with an embedded free list.
#[repr(C)]
pub struct Slab {
    next: *mut Slab, // next slab in this size class's list (intrusive)
    object_size: u64,
    free_list: *mut u8,
}

impl Slab {
    /// Carve a fresh slab out of a 4096-byte, page-aligned `page`.
    /// Returns a pointer to the in-page `Slab` header.
    pub fn new(page: u64, object_size: u64) -> *mut Slab {
        // Reserve the front of the page for the header, rounded up to a whole
        // slot so the first object stays `object_size`-aligned.
        let header_size =
            ((core::mem::size_of::<Slab>() as u64 + object_size - 1) / object_size) * object_size;
        let data_start = page + header_size;
        let count = (4096 - header_size) / object_size;

        // Thread the embedded free list through every slot.
        for i in 0..count {
            let slot = (data_start + i * object_size) as *mut u64;
            let next = if i + 1 < count {
                data_start + (i + 1) * object_size
            } else {
                0 // end of free list
            };
            unsafe {
                *slot = next;
            }
        }

        let slab = page as *mut Slab;
        unsafe {
            (*slab).next = core::ptr::null_mut();
            (*slab).object_size = object_size;
            (*slab).free_list = data_start as *mut u8;
        }
        slab
    }

    pub fn allocate(&mut self) -> Option<*mut u8> {
        if self.free_list.is_null() {
            return None; // this slab is full
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

/// One size class: a singly linked list of slabs, all carved for `object_size`.
/// Slabs are created lazily — the list starts empty and grows on demand.
pub struct SlabClass {
    head: *mut Slab,
    object_size: u64,
}

impl SlabClass {
    pub fn allocate(&mut self, allocator: &mut BuddyAllocator) -> Option<*mut u8> {
        // Fast path: try every slab already in this class's list.
        let mut slab = self.head;
        while !slab.is_null() {
            if let Some(obj) = unsafe { (*slab).allocate() } {
                return Some(obj);
            }
            slab = unsafe { (*slab).next };
        }

        // Every existing slab is full (or the list is empty). Grow the class:
        // pull a fresh page from the buddy, carve a slab, prepend it, allocate.

        if let Some(page) = allocator.allocate(4096) {
            let new_slab = Slab::new(page, self.object_size);
            unsafe {
                (*new_slab).next = self.head;
            }
            self.head = new_slab;
            unsafe { (*new_slab).allocate() }
        } else {
            None
        }
    }

    pub fn free(&mut self, ptr: *mut u8) {
        // O(1): every slab is one 4096-aligned page with its header at the
        // page base, so masking the low 12 bits of any object pointer yields
        // the owning slab's header.
        let slab = ((ptr as u64) & !0xFFFu64) as *mut Slab;
        unsafe {
            (*slab).free(ptr);
        }
    }
}

pub struct SlabAllocator {
    classes: [SlabClass; SIZE_CLASSES.len()],
}

impl SlabAllocator {
    pub fn init() -> SlabAllocator {
        let classes = core::array::from_fn(|i| SlabClass {
            head: core::ptr::null_mut(),
            object_size: SIZE_CLASSES[i],
        });
        SlabAllocator { classes }
    }

    pub fn allocate(&mut self, size: u64, allocator: &mut BuddyAllocator) -> Option<*mut u8> {
        for class in self.classes.iter_mut() {
            if size <= class.object_size {
                return class.allocate(allocator);
            }
        }
        None // size too large for the slab allocator
    }

    pub fn free(&mut self, ptr: *mut u8, size: u64) {
        for class in self.classes.iter_mut() {
            if size <= class.object_size {
                class.free(ptr);
                return;
            }
        }
        // Invalid size, ignore
    }
}
