#![no_std]

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
    pub boot_services: usize,
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
pub struct EfiSimpleTextOutputProtocol {
    pub reset: extern "efiapi" fn(*mut EfiSimpleTextOutputProtocol, bool) -> usize,
    pub output_string: extern "efiapi" fn(*mut EfiSimpleTextOutputProtocol, *const u16) -> usize,
}