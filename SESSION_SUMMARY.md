# Asteria OS — Session Summary

Bare-metal x86_64 OS written from scratch in Rust, consisting of a UEFI bootloader and a kernel.

## What's Working

### Bootloader (`asteria-bootloader`)
- UEFI boot flow: opens loaded image protocol, locates file system protocol, reads `/kernel.elf` from the EFI partition
- Allocates separate buffers for the raw ELF bytes and the loaded image (to avoid self-overwriting during load)
- **ELF64 loader** with PT_LOAD segment copying and BSS zeroing
- **Relocation processing** — walks `.rela.dyn` and applies `R_X86_64_RELATIVE` relocations for PIE kernels (required for Rust's `core::fmt` trait dispatch)
- Captures UEFI memory map, calls `exit_boot_services`, jumps to kernel using `extern "sysv64"` calling convention (to match the kernel's ABI)
- Passes `(memory_map_addr, memory_map_size, descriptor_size)` to `kernel_main`

### Kernel (`asteria-kernel`)
- **Assembly entry point** (`boot.s`) — sets up 64KB stack, calls `kernel_main`
- **GDT** with null/kernel code/kernel data segments, loaded with `lgdt` and segment register reload via far return
- **IDT** with handlers for divide-by-zero, invalid opcode, double fault, GP fault, page fault. Assembly stubs push vector/error code and call a common handler that saves all registers and passes an `InteruptFrame` pointer to a Rust handler which prints vector name, error code, and RIP
- **Serial output** via COM1 (port `0x3F8`) with `print!`/`println!` macros using `fmt::Write`
- **4-level paging** — identity-maps all RAM (using `max_address` from the frame allocator), loads CR3
- **Frame allocator** — bitmap-based, parses the UEFI memory map, marks type 7 (EfiConventionalMemory) as free, reserves low 1MB. Has `allocate_page`, `free_page`, `allocate_pages` (contiguous scan), `free_pages`
- **Slab allocator** — each `Slab` manages one 4KB page divided into fixed-size slots with an embedded free list (next pointers stored in free slots themselves). `SlabAllocator` has 7 size classes: 32, 64, 128, 256, 512, 1024, 2048
- **Global allocator** — implements `GlobalAlloc` via a `spin::Mutex<Option<KernelAllocatorInner>>`. Routes allocations ≤ 2048 to the slab allocator, larger to `allocate_pages` on the frame allocator. Integrated with Rust's `alloc` crate — `Vec`, `Box`, `String`, etc. all work
- **Buddy allocator** (in progress) — `src/memory/buddy.rs` has `BuddyAllocator` struct, `FreeBlock`, `size_to_order`, `init` (handles leftover memory with proper alignment), and `allocate` with splitting. `free` not yet written.

## Project Structure

```
asteria/
├── Makefile                   # builds both crates, runs QEMU with -serial stdio
├── README.md
├── asteria-bootloader/
│   ├── .cargo/config.toml     # target = x86_64-unknown-uefi
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # UEFI boot flow
│       ├── lib.rs             # UEFI type definitions
│       └── parser.rs          # ELF loader + relocations
└── asteria-kernel/
    ├── .cargo/config.toml     # target = x86_64-unknown-none
    ├── Cargo.toml             # depends on spin = 0.9
    ├── build.rs               # assembles boot.s, passes linker script
    ├── kernel.ld              # linker script, ENTRY(_start)
    └── src/
        ├── main.rs            # mod declarations, #[global_allocator], kernel_main
        ├── boot.s             # assembly entry point
        ├── gdt.rs
        ├── idt.rs
        ├── serial.rs          # print!/println! macros
        └── memory/
            ├── mod.rs         # pub use frame::*;
            ├── frame.rs       # bitmap frame allocator
            ├── paging.rs      # 4-level page tables
            ├── slab.rs        # Slab + SlabAllocator
            ├── allocator.rs   # KernelAllocator implementing GlobalAlloc
            └── buddy.rs       # BuddyAllocator (in progress — missing free)
```

## Key Bugs Fixed Along the Way

- `core::hint::black_box` was a red herring — the real bootloader bug was `load_buffer` being sized from `kernel_size` (file size) instead of accounting for BSS (`p_memsz`), causing overflow into UEFI data
- The PIE kernel needed `.rela.dyn` relocations processed — without them, `core::fmt` trait dispatch went through GOT entries containing link-time addresses near 0, causing page faults when formatting runtime values
- `extern "C"` in UEFI vs `extern "C"` in bare-metal use different calling conventions (MS x64 vs SysV) — fixed by using `extern "sysv64"` when calling the kernel
- Workspace layout broke rust-analyzer (can't handle multi-target workspaces) — solved by removing the workspace and having independent crates
- IDT handler frame reading required the frame pointer be treated as a raw pointer (not a struct reference) due to how the compiler generated code inside `#[unsafe(naked)]` stubs
- `allocate_pages` failed for sizes > 4096 until the buddy path was added to `dealloc`

## Next Steps

1. **Finish buddy allocator** — write `free` with coalescing via XOR buddy address trick
2. **Replace bitmap frame allocator** with buddy allocator (cleaner, O(1) common case, handles multi-page naturally)
3. **Hardware interrupts** — set up PIC/APIC, write keyboard driver
4. **Higher-half kernel** — move kernel to `0xFFFF800000000000`
5. **Framebuffer graphics** — get GOP framebuffer from bootloader
6. **Filesystem / disk I/O**

## Current Session State

Mid-implementation of the buddy allocator. `allocate` is done, `free` with coalescing is pending. The buddy allocator hasn't replaced the bitmap frame allocator yet — both exist in `memory/`.
