#![no_std]
#![no_main]



#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

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
    pub reset: usize,
    pub output_string: extern "efiapi" fn(*mut EfiSimpleTextOutputProtocol, *const u16) -> usize,

}
#[unsafe(no_mangle)]
extern "efiapi" fn efi_main(_image: usize, system_table: *mut  EfiSystemTable) -> usize {
    unsafe {
        let stdout = (*system_table).con_out;

        //  UTF-16 string for "Bootloader"
        let hello_world: [u16; 15] = [
            'B' as u16,
            'o' as u16,
            'o' as u16,
            't' as u16,
            'i' as u16,
            'n' as u16,
            'g' as u16,
            ' ' as u16,
            'A' as u16,
            's' as u16,
            't' as u16,
            'e' as u16,
            'r' as u16,
            'i' as u16,
            'a' as u16,
        ]; 
        ((*stdout).output_string)(stdout, hello_world.as_ptr());
    }
    loop{}
    0
}
