#[repr(C)]
pub struct ELF64Header {
    pub e_ident: [u8; 16], // ELF identification bytes
    pub e_type: u16,       // Object file type (2 for executable)
    pub e_machine: u16,    // Machine type (0x3E for x86-64)
    pub e_version: u32,    // ELF version (1 for the original version)
    pub e_entry: u64,      // Entry point address
    pub e_phoff: u64,      // Program header table offset
    pub e_shoff: u64,      // Section header table offset
    pub e_flags: u32,      // Processor-specific flags
    pub e_ehsize: u16,     // ELF header size
    pub e_phentsize: u16,  // Size of a program header entry
    pub e_phnum: u16,      // Number of program header entries
    pub e_shentsize: u16,  // Size of a section header entry
    pub e_shnum: u16,      // Number of section header entries
    pub e_shstrndx: u16,   // Section header string table index
}

#[repr(C)]
pub struct ELF64Rela {
    pub r_offset: u64, // Address of the relocation
    pub r_info: u64,   // Relocation type and symbol index
    pub r_addend: i64, // Addend for the relocation
}

#[repr(C)]
struct ELF64Dyn {
    pub d_tag: i64, // Dynamic entry type (e.g., DT_NULL, DT_RELA, etc.)
    pub d_un: u64,  // Union of values (e.g., pointer or integer)
}
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F']; // ELF magic number for identification, go at ELF64Header.e_ident[0..4]
const ELF_CLASS_64: u8 = 2; // ELF class for 64-bit objects, go at ELF64Header.e_ident[4]
const ELF_DATA_LSB: u8 = 1; // Data encoding for little-endian
const ELF_VERSION_CURRENT: u8 = 1; // Current ELF version
const ELF_TYPE_EXECUTABLE: u16 = 2; // Object file type for executable files
const ELF_TYPE_DYNAMIC: u16 = 3; // Object file type for shared objects (dynamic libraries)

#[repr(C)]
pub struct ELF64ProgramHeader {
    pub p_type: u32,   // Segment type (e.g., PT_LOAD for loadable segments)
    pub p_flags: u32,  // Segment flags (e.g., PF_R for readable, PF_W for writable)
    pub p_offset: u64, // Offset of the segment in the file
    pub p_vaddr: u64,  // Virtual address of the segment in memory
    pub p_paddr: u64,  // Physical address of the segment (unused on many platforms)
    pub p_filesz: u64, // Size of the segment in the file
    pub p_memsz: u64,  // Size of the segment in memory
    pub p_align: u64,  // Alignment of the segment in memory
}

const PT_LOAD: u32 = 1; // Segment type for loadable segments

pub fn load_elf(data: *const u8, load_base: u64) -> Option<u64> {
    let header = unsafe { &*(data as *const ELF64Header) };

    if header.e_ident[0..4] != ELF_MAGIC {
        return None; // Not a valid ELF file
    }
    if header.e_ident[4] != ELF_CLASS_64 {
        return None; // Not a 64-bit ELF file
    }
    if header.e_ident[5] != ELF_DATA_LSB {
        return None; // Not a little-endian ELF file
    }
    if header.e_ident[6] != ELF_VERSION_CURRENT {
        return None; // Unsupported ELF version
    }
    if header.e_type != ELF_TYPE_EXECUTABLE && header.e_type != ELF_TYPE_DYNAMIC {
        return None; // Not an executable or dynamic ELF file
    }

    for i in 0..header.e_phnum {
        let ph_offset = header.e_phoff + (i as u64 * header.e_phentsize as u64);
        let program_header =
            unsafe { &*(data.offset(ph_offset as isize) as *const ELF64ProgramHeader) };

        if program_header.p_type == PT_LOAD {
            let dest = (program_header.p_paddr + load_base) as *mut u8;
            unsafe {
                let src = data.offset(program_header.p_offset as isize);
                core::ptr::copy_nonoverlapping(src, dest, program_header.p_filesz as usize);
                if program_header.p_memsz > program_header.p_filesz {
                    core::ptr::write_bytes(
                        dest.add(program_header.p_filesz as usize),
                        0,
                        (program_header.p_memsz - program_header.p_filesz) as usize,
                    );
                }
            }
        } else if program_header.p_type == 2 {
            let dyn_addr = (load_base + program_header.p_vaddr) as *const ELF64Dyn;
            let dyn_count = program_header.p_filesz / core::mem::size_of::<ELF64Dyn>() as u64;

            let mut rela_addr: u64 = 0;
            let mut rela_size: u64 = 0;
            let mut rela_ent: u64 = 0;

            // Scan the dynamic entries to find the relocation information
            for j in 0..dyn_count {
                let dyn_entry = unsafe { &*dyn_addr.add(j as usize) };
                match dyn_entry.d_tag {
                    0 => break,                      // DT_NULL, end of dynamic entries
                    7 => rela_addr = dyn_entry.d_un, // DT_RELA
                    8 => rela_size = dyn_entry.d_un, // DT_RELASZ
                    9 => rela_ent = dyn_entry.d_un,  // DT_RELAENT
                    _ => {}
                }
            }

            // Apply relocations if we found the relocation information
            if rela_addr != 0 && rela_ent != 0 {
                let count = rela_size / rela_ent;
                for j in 0..count {
                    let rela =
                        unsafe { &*((load_base + rela_addr + j * rela_ent) as *const ELF64Rela) };
                    // R_X86_64_RELATIVE (type 8) = 0
                    if (rela.r_info & 0xFF) == 8 {
                        let ptr = (load_base + rela.r_offset) as *mut u64;
                        unsafe {
                            *ptr = load_base as i64 as u64 + rela.r_addend as u64;
                        }
                    }
                }
            }
        }
    }
    Some(load_base + header.e_entry)
}

