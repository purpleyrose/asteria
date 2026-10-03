use core::alloc::GlobalAlloc;

use crate::memory::buddy::BuddyAllocator;
use crate::memory::slab::SlabAllocator;
use spin;
pub struct KernelAllocatorInner {
    buddy_allocator: BuddyAllocator,
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

    pub fn init(&self, buddy: BuddyAllocator, slab: SlabAllocator) {
        *self.inner.lock() = Some(KernelAllocatorInner {
            buddy_allocator: buddy,
            slab_allocator: slab,
        });
    }
}

unsafe impl Send for KernelAllocator {}
unsafe impl Sync for KernelAllocator {}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut guard = self.inner.lock();
        if let Some(inner) = guard.as_mut() {
            if layout.size() <= 2048 {
                if let Some(ptr) = inner
                    .slab_allocator
                    .allocate(layout.size() as u64, &mut inner.buddy_allocator)
                {
                    return ptr as *mut u8;
                }
            } else {
                if let Some(addr) = inner.buddy_allocator.allocate(layout.size() as u64) {
                    return addr as *mut u8;
                }
            }
        }
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let mut guard = self.inner.lock();
        if let Some(inner) = guard.as_mut() {
            if layout.size() <= 2048 {
                inner.slab_allocator.free(ptr, layout.size() as u64);
            } else {
                unsafe {
                    inner.buddy_allocator.free(ptr as u64, layout.size() as u64);
                }
            }
        }
    }
}
