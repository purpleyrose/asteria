use core::alloc::GlobalAlloc;

use crate::memory::FrameAllocator;
use crate::memory::slab::SlabAllocator;
use spin;
pub struct KernelAllocatorInner {
    frame_allocator: FrameAllocator,
    slab_allocator: SlabAllocator,
}

pub struct KernelAllocator {
    inner: spin::Mutex<Option<KernelAllocatorInner>>,
}

impl KernelAllocator {
    pub const fn new() -> Self {
        Self {
            inner: spin::Mutex::new(None),
        }
    }

    pub fn init(&self, frame: FrameAllocator, slab: SlabAllocator) {
        *self.inner.lock() = Some(KernelAllocatorInner {
            frame_allocator: frame,
            slab_allocator: slab,
        });
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut guard = self.inner.lock();
        if let Some(inner) = guard.as_mut() {
            if let Some(ptr) = inner.slab_allocator.allocate(layout.size() as u64) {
                return ptr as *mut u8;
            }
            if let Some(page) = inner.frame_allocator.allocate_page() {
                return page as *mut u8;
            }
        }
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let mut guard = self.inner.lock();
        if let Some(inner) = guard.as_mut() {
            inner.slab_allocator.free(ptr, layout.size() as u64);
        }
    }
}
