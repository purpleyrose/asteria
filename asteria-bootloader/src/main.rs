#![no_std]
#![no_main]

use asteria_bootloader::EfiSystemTable;


#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}




#[unsafe(no_mangle)]
extern "efiapi" fn efi_main(_image: usize, system_table: *mut  EfiSystemTable) -> ! {
    unsafe {
        let stdout = (*system_table).con_out;

        //  UTF-16 string for "Bootloader"
        let hello_world: [u16; 16] = [
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
            0u16
        ]; 
        ((*stdout).output_string)(stdout, hello_world.as_ptr());
    }
    loop{}
}
