#![no_std]

pub mod parser;


#[repr(C)]
pub struct EfiSystemTable {
    pub hdr: EfiTableHeader,
    pub firmware_vendor: *const u16,
    pub firmware_revision: u32,
    pub console_in_handle: usize,
    pub con_in: usize,
    pub console_out_handle: usize,
    pub con_out: *mut EfiSimpleTextOutputProtocol,
    pub standard_error_handle: usize,
    pub std_err: *mut EfiSimpleTextOutputProtocol,
    pub runtime_services: usize,
    pub boot_services: *mut EfiBootServices,
    pub number_of_table_entries: usize,
    pub configuration_table: usize,
}

#[repr(C)]
pub struct EfiTableHeader {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub crc32: u32,
    pub reserved: u32,
}


#[repr(C)]
pub struct EfiBootServices {
    pub hdr: EfiTableHeader,
    pub raise_tpl: usize,
    pub restore_tpl: usize,
    pub allocate_pages: extern "efiapi" fn(AllocateType, MemoryType, usize, *mut u64) -> usize, // 3rd param represents the number of pages to allocate, and the 4th param is a pointer to a u64 where the allocated address will be stored
    pub free_pages: extern "efiapi" fn(u64, usize) -> usize, // 1st param is the starting address of the pages to free, and the 2nd param is the number of pages to free
    pub get_memory_map: extern "efiapi" fn(*mut usize, *mut EfiMemoryDescriptor, *mut usize, *mut usize, *mut u32) -> usize, 
    pub allocate_pool: usize,
    pub free_pool: usize,
    pub create_event: usize,
    pub set_timer: usize,
    pub wait_for_event: usize,
    pub signal_event: usize,
    pub close_event: usize,
    pub check_event: usize,
    pub install_protocol_interface: usize,
    pub reinstall_protocol_interface: usize,
    pub uninstall_protocol_interface: usize,
    pub handle_protocol: usize,
    pub reserved: usize,
    pub register_protocol_notify: usize,
    pub locate_handle: usize,
    pub locate_device_path: usize,
    pub install_configuration_table: usize,
    pub image_load: usize,
    pub image_start: usize,
    pub exit: usize,
    pub image_unload: usize,
    pub exit_boot_services: extern "efiapi" fn(usize, usize) -> usize, // 1st param is the image handle of the calling image, and the 2nd param is a pointer to the memory map key returned by GetMemoryMap
    pub get_next_monotonic_count: usize,
    pub stall: usize,
    pub set_watchdog_timer: usize,
    pub connect_controller: usize,
    pub disconnect_controller: usize,
    pub open_protocol: extern "efiapi" fn(usize,  *const EfiGuid, *mut *mut usize, usize, usize, u32) -> usize,
    pub close_protocol: usize,
    pub open_protocol_information: usize,
    pub protocols_per_handle: usize,
    pub locate_handle_buffer: usize,
    pub locate_protocol: usize,
    pub install_multiple_protocol_interfaces: usize,
    pub uninstall_multiple_protocol_interfaces: usize,
    pub calculate_crc32: usize, // 32-bit CRC calculation function
    pub copy_mem: usize, // Memory copy function
    pub set_mem: usize, // Memory set function
    pub create_event_ex: usize, // Extended event creation function
}

#[repr(C)]
pub struct EfiLoadedImageProtocol {
    pub revision: u32,
    pub parent_handle: usize,
    pub system_table: *mut EfiSystemTable,
    // Source location of the image.
    pub device_handle: usize,
    pub file_path: usize, // TODO: Replace with pointer to EFI_DEVICE_PATH_PROTOCOL
    pub reserved: usize,
    // Image's load options.
    pub load_options_size: u32,
    pub load_options: *mut usize,
    // Location of the image in memory.
    pub image_base: usize,
    pub image_size: u64,
    pub image_code_type: usize,
    pub image_data_type: usize,
    pub unload: usize, // TODO: Replace with function pointer to image unload function 
}



#[repr(C)]
pub struct EfiSimpleFileSystemProtocol {
    pub revision: u64,
    pub open_volume:
        extern "efiapi" fn(
            *mut EfiSimpleFileSystemProtocol,
            *mut *mut EfiFileProtocol,
        ) -> usize, 
}

#[repr(C)]
pub struct EfiFileProtocol {
    pub revision: u64,
    pub open: extern "efiapi" fn(
        *mut EfiFileProtocol,
        *mut *mut EfiFileProtocol,
        *const u16, // File name as a null-terminated UTF-16 string
        u64, // Open mode (e.g., read, write, create)
        u64, // Attributes (e.g., read-only, hidden)
        
    )-> usize,
    pub close: extern "efiapi" fn(*mut EfiFileProtocol) -> usize,
    pub delete: extern "efiapi" fn(*mut EfiFileProtocol) -> usize,
    pub read: extern "efiapi" fn(
        *mut EfiFileProtocol,
        *mut usize, // Size of the buffer to read into, and on return, the actual number of bytes read
        *mut u8, // Buffer to read data into
    ) -> usize,
    pub write: extern "efiapi" fn(
        *mut EfiFileProtocol,
        *mut usize, // Size of the buffer to write from, and on return, the actual number of bytes written
        *const u8, // Buffer containing data to write
    ) -> usize, 
    pub get_position: extern "efiapi" fn(*mut EfiFileProtocol, *mut u64) -> usize,
    pub set_position: extern "efiapi" fn(*mut EfiFileProtocol, u64) -> usize,
    pub get_info: extern "efiapi" fn(
        *mut EfiFileProtocol,
        *mut EfiGuid, // GUID of the information class to retrieve
        *mut usize, // Size of the buffer to receive the information, and on return, the actual size of the information returned
        *mut u8, // Buffer to receive the information
    ) -> usize,
    pub set_info: extern "efiapi" fn(
        *mut EfiFileProtocol,
        *mut EfiGuid, // GUID of the information class to set
        usize, // Size of the information being set
        *const u8, // Buffer containing the information to set
    ) -> usize,
     pub flush: extern "efiapi" fn(*mut EfiFileProtocol) -> usize,
}


#[repr(usize)]
pub enum AllocateType {
    AllocateAnyPages = 0,
    AllocateMaxAddress = 1,
    AllocateAddress = 2,
    MaxAllocateType = 3,
}

#[repr(usize)]
pub enum MemoryType {
    EfiReservedMemoryType = 0,
    EfiLoaderCode = 1,
    EfiLoaderData = 2,
    EfiBootServicesCode = 3,
    EfiBootServicesData = 4,
    EfiConventionalMemory = 7,
    EfiUnusableMemory = 8,
    EfiACPIReclaimMemory = 9,
    EfiAPCIMemoryNVS = 10,
    EfiMemoryMappedIO = 11,
    EfiMemoryMappedIOPortSpace = 12,
    EfiPalCode = 13,
    EfiPersistentMemory = 14,
    EfiUnacceptedMemoryType = 15,
    EfiMaxMemoryType = 16,
}

#[repr(C)]
pub struct EfiMemoryDescriptor {
    pub typ: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}

#[repr(C)]
pub struct EfiGuid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

/// Colors for the SetAttribute function of the EfiSimpleTextOutputProtocol
/// The color is a combination of foreground and background colors. The foreground color is in the lower 4 bits, and the background color is in the upper 4 bits.
pub enum ForegroundColors {
    Black = 0x00,
    Blue = 0x01,
    Green = 0x02,
    Cyan = 0x03,
    Red = 0x04,
    Magenta = 0x05,
    Brown = 0x06,
    LightGray = 0x07,
    DarkGray = 0x08,
    LightBlue = 0x09,
    LightGreen = 0x0A,
    LightCyan = 0x0B,
    LightRed = 0x0C,
    LightMagenta = 0x0D,
    Yellow = 0x0E,
    White = 0x0F,
}

pub enum BackgroundColors {
    Black = 0x00,
    Blue = 0x10,
    Green = 0x20,
    Cyan = 0x30,
    Red = 0x40,
    Magenta = 0x50,
    Brown = 0x60,
    LightGray = 0x70,
}

#[repr(C)]
pub struct EfiSimpleTextOutputProtocol {
    pub reset: extern "efiapi" fn(*mut EfiSimpleTextOutputProtocol, bool) -> usize,
    pub output_string: extern "efiapi" fn(*mut EfiSimpleTextOutputProtocol, *const u16) -> usize,
    pub test_string: extern "efiapi" fn(*mut EfiSimpleTextOutputProtocol, *const u16) -> usize,
    pub query_mode: extern "efiapi" fn(*mut EfiSimpleTextOutputProtocol, usize, *mut usize, *mut usize) -> usize,
    pub set_mode: extern "efiapi" fn(*mut EfiSimpleTextOutputProtocol, usize) -> usize,
    pub set_attribute: extern "efiapi" fn(*mut EfiSimpleTextOutputProtocol, usize) -> usize,
    pub clear_screen: extern "efiapi" fn(*mut EfiSimpleTextOutputProtocol) -> usize,
    pub set_cursor_position: extern "efiapi" fn(*mut EfiSimpleTextOutputProtocol, usize, usize) -> usize,
    pub enable_cursor: extern "efiapi" fn(*mut EfiSimpleTextOutputProtocol, bool) -> usize,
}
