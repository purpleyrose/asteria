#![no_std]
#![no_main]

use asteria_bootloader::parser;
use asteria_bootloader::{
    AllocateType, EFI_FILE_INFO_GUID, EFI_LOADED_IMAGE_PROTOCOL_GUID,
    EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID, EfiFileInfo, EfiFileProtocol, EfiLoadedImageProtocol,
    EfiSimpleFileSystemProtocol, EfiSystemTable, MemoryType,
};
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

unsafe fn serial_print(msg: &[u8]) {
    for &b in msg {
        unsafe {
            core::arch::asm! {
                "out dx, al",
                in("dx") 0x3F8u16, // COM1 port
                in("al") b,
            }
        }
    }
}

#[unsafe(no_mangle)]
extern "efiapi" fn efi_main(image: usize, system_table: *mut EfiSystemTable) -> ! {
    unsafe {
        let _stdout = (*system_table).con_out;
        let boot_services = (*system_table).boot_services;
        let mut loaded_image: *mut usize = core::ptr::null_mut();
        ((*boot_services).open_protocol)(
            image,
            &EFI_LOADED_IMAGE_PROTOCOL_GUID,
            &mut loaded_image as *mut *mut usize,
            image,
            0,
            0x00000001,
        );
        serial_print(b"1: open_protocol(loaded_image) done\n");

        let loaded_image = loaded_image as *mut EfiLoadedImageProtocol;
        let mut file_system: *mut usize = core::ptr::null_mut();
        ((*boot_services).locate_protocol)(
            &EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID,
            core::ptr::null_mut(),
            &mut file_system,
        );
        serial_print(b"2: locate_protocol(file_system) done\n");

        let file_system = file_system as *mut EfiSimpleFileSystemProtocol;
        let mut root: *mut EfiFileProtocol = core::ptr::null_mut();
        ((*file_system).open_volume)(file_system, &mut root);
        serial_print(b"3: open_volume done\n");

        const KERNEL_PATH: [u16; 12] = [
            '\\' as u16,
            'k' as u16,
            'e' as u16,
            'r' as u16,
            'n' as u16,
            'e' as u16,
            'l' as u16,
            '.' as u16,
            'e' as u16,
            'l' as u16,
            'f' as u16,
            0,
        ];
        let mut kernel_file: *mut EfiFileProtocol = core::ptr::null_mut();
        ((*root).open)(
            root,
            &mut kernel_file,
            KERNEL_PATH.as_ptr(),
            0x0000000000000001,
            0,
        );
        serial_print(b"4: open(kernel.elf) done\n");

        let mut file_info_buffer: [u8; 512] = [0; 512];
        let mut file_info_size: usize = core::mem::size_of_val(&file_info_buffer);
        ((*kernel_file).get_info)(
            kernel_file,
            &EFI_FILE_INFO_GUID as *const _ as *mut _,
            &mut file_info_size,
            file_info_buffer.as_mut_ptr(),
        );
        serial_print(b"5: get_info done\n");

        let file_info = file_info_buffer.as_ptr() as *const EfiFileInfo;
        let kernel_size = (*file_info).file_size;

        let pages = (kernel_size as usize + 0xFFF) / 0x1000;
        let mut kernel_buffer: u64 = 0;
        let mut load_buffer: u64 = 0;
        let load_pages = pages + 16; // extra pages for BSS (stack, zero-initialized data)
        ((*boot_services).allocate_pages)(
            AllocateType::AllocateAnyPages,
            MemoryType::EfiLoaderData,
            load_pages,
            &mut load_buffer,
        );
        ((*boot_services).allocate_pages)(
            AllocateType::AllocateAnyPages,
            MemoryType::EfiLoaderData,
            pages,
            &mut kernel_buffer,
        );

        serial_print(b"6: allocate_pages done\n");

        let mut read_size = kernel_size as usize;
        ((*kernel_file).read)(kernel_file, &mut read_size, kernel_buffer as *mut u8);
        serial_print(b"7: read done\n");

        let entry_point = match parser::load_elf(kernel_buffer as *const u8, load_buffer) {
            Some(entry) => entry,
            None => {
                serial_print(b"Failed to load ELF file\n");
                loop {}
            }
        };
        serial_print(b"8: load_elf done\n");

        let mut memory_map_size: usize = 0;
        let mut map_key: usize = 0;
        let mut descriptor_size: usize = 0;
        let mut descriptor_version: u32 = 0;
        ((*boot_services).get_memory_map)(
            &mut memory_map_size,
            core::ptr::null_mut(),
            &mut map_key,
            &mut descriptor_size,
            &mut descriptor_version,
        );

        let mut memory_map_buffer: u64 = 0;
        ((*boot_services).allocate_pages)(
            AllocateType::AllocateAnyPages,
            MemoryType::EfiLoaderData,
            4,
            &mut memory_map_buffer,
        );
        memory_map_size = 4 * 0x1000;
        ((*boot_services).get_memory_map)(
            &mut memory_map_size,
            memory_map_buffer as *mut _,
            &mut map_key,
            &mut descriptor_size,
            &mut descriptor_version,
        );
        serial_print(b"9: get_memory_map done\n");

        ((*boot_services).exit_boot_services)(image, map_key);
        serial_print(b"10: exit_boot_services done\n");

        let entry_fn: extern "C" fn() -> ! = core::mem::transmute(entry_point);
        entry_fn();
    }
}
