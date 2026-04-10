# Asteria

A bare-metal operating system written from scratch in Rust, targeting x86_64.

## Architecture

Asteria consists of two main components:

- **asteria-bootloader** -- A UEFI bootloader that loads the kernel ELF binary, processes relocations, sets up the memory map, exits boot services, and jumps to the kernel entry point.
- **asteria-kernel** -- A bare-metal kernel with its own GDT, IDT, paging, physical frame allocator, slab allocator, and a global allocator integrated with Rust's `alloc` crate.

## Current Features

### Bootloader
- UEFI boot flow with protocol-based file system access
- ELF64 loader with PT_LOAD segment mapping
- R_X86_64_RELATIVE relocation support for PIE kernels
- Memory map capture and handoff to the kernel

### Kernel
- Assembly entry point (`boot.s`) with dedicated kernel stack
- Global Descriptor Table (GDT) with kernel code/data segments
- Interrupt Descriptor Table (IDT) with handlers for CPU exceptions (divide-by-zero, invalid opcode, double fault, GP fault, page fault)
- Serial output via COM1 with `print!`/`println!` macros
- 4-level paging with identity-mapped physical memory
- Physical frame allocator (bitmap-based) over the UEFI memory map
- Slab allocator with multiple size classes (32, 64, 128, 256, 512, 1024, 2048 bytes)
- Multi-page contiguous allocation for large allocations
- `GlobalAlloc` implementation — `Vec`, `Box`, `String`, and other `alloc` collections work in the kernel

## Building

Requires:
- Rust (stable toolchain)
- `x86_64-unknown-uefi` and `x86_64-unknown-none` targets (`rustup target add ...`)
- QEMU with OVMF firmware
- GNU Make

```sh
make build   # Build bootloader and kernel
make run     # Build and run in QEMU (serial output to asteria-serial.log)
make run-gui # Build and run with graphical display
make clean   # Remove build artifacts
```

## Project Structure

```
asteria/
├── Makefile
├── asteria-bootloader/
│   └── src/
│       ├── main.rs       # UEFI boot flow
│       ├── lib.rs        # UEFI type definitions and protocol structs
│       └── parser.rs     # ELF loader with relocation support
├── asteria-kernel/
│   ├── kernel.ld         # Linker script
│   ├── build.rs          # Assembles boot.s, passes linker script
│   └── src/
│       ├── boot.s        # Assembly entry point (_start)
│       ├── main.rs       # kernel_main
│       ├── gdt.rs        # Global Descriptor Table
│       ├── idt.rs        # Interrupt Descriptor Table + exception handlers
│       ├── serial.rs     # COM1 serial output + print macros
│       └── memory/
│           ├── mod.rs        # Module organizer + re-exports
│           ├── frame.rs      # Physical frame allocator (bitmap)
│           ├── paging.rs     # 4-level page tables
│           ├── slab.rs       # Slab allocator with size classes
│           └── allocator.rs  # GlobalAlloc implementation
```

## License

See [LICENSE](LICENSE).
