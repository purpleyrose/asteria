use crate::memory;

#[repr(transparent)]
pub struct PageTableEntry(u64);

#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

pub const PRESENT: u64 = 1 << 0;
pub const WRITABLE: u64 = 1 << 1;

impl PageTableEntry {
    pub fn new() -> Self {
        PageTableEntry(0)
    }

    pub fn is_present(&self) -> bool {
        (self.0 & PRESENT) != 0
    }

    pub fn set(&mut self, phys_addr: u64, flags: u64) {
        self.0 = (phys_addr & 0x000fffff_fffff000) | flags;
    }

    pub fn addr(&self) -> u64 {
        self.0 & 0x000fffff_fffff000
    }
}

pub fn map_page(
    pml4: &mut PageTable,
    virt_addr: u64,
    phys_addr: u64,
    flags: u64,
    allocator: &mut memory::FrameAllocator,
) {
    // Excract indices
    let pml4_index = (virt_addr >> 39) & 0x1FF;
    let pdpt_index = (virt_addr >> 30) & 0x1FF;
    let pd_index = (virt_addr >> 21) & 0x1FF;
    let pt_index = (virt_addr >> 12) & 0x1FF;

    // Traverse or create page tables
    // PML4
    let pml4_entry = &mut pml4.entries[pml4_index as usize];
    if !pml4_entry.is_present() {
        let new_table = allocator
            .allocate_page()
            .expect("Out of memory for page tables");
        unsafe {
            core::ptr::write_bytes(new_table as *mut u8, 0, 4096);
        }
        pml4_entry.set(new_table, PRESENT | WRITABLE);
    }
    let pdpt = unsafe { &mut *(pml4_entry.addr() as *mut PageTable) };
    if !pdpt.entries[pdpt_index as usize].is_present() {
        let new_table = allocator
            .allocate_page()
            .expect("Out of memory for page tables");
        unsafe {
            core::ptr::write_bytes(new_table as *mut u8, 0, 4096);
        }
        pdpt.entries[pdpt_index as usize].set(new_table, PRESENT | WRITABLE);
    }
    let pd = unsafe { &mut *(pdpt.entries[pdpt_index as usize].addr() as *mut PageTable) };
    if !pd.entries[pd_index as usize].is_present() {
        let new_table = allocator
            .allocate_page()
            .expect("Out of memory for page tables");
        unsafe {
            core::ptr::write_bytes(new_table as *mut u8, 0, 4096);
        }
        pd.entries[pd_index as usize].set(new_table, PRESENT | WRITABLE);
    }
    let pt = unsafe { &mut *(pd.entries[pd_index as usize].addr() as *mut PageTable) };
    pt.entries[pt_index as usize].set(phys_addr, flags | PRESENT);
}

pub fn init(allocator: &mut memory::FrameAllocator) {
    let pml4_addr = allocator.allocate_page().expect("Failed to allocate PML4");
    unsafe {
        core::ptr::write_bytes(pml4_addr as *mut u8, 0, 4096);
    }
    let pml4 = unsafe { &mut *(pml4_addr as *mut PageTable) };

    let pages = allocator.max_address / 4096;
    for i in 0..pages {
        let addr = i as u64 * 4096;
        map_page(pml4, addr, addr, PRESENT | WRITABLE, allocator);
    }

    // Load CR3
    unsafe { core::arch::asm!("mov cr3, {0}", in(reg) pml4_addr) }
}
