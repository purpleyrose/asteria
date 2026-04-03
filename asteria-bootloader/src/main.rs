#![no_std]
#![no_main]

use asteria_bootloader::{EfiSystemTable, EfiSimpleTextOutputProtocol, ForegroundColors, BackgroundColors};
use asteria_bootloader::parser;
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}




#[unsafe(no_mangle)]
extern "efiapi" fn efi_main(_image: usize, system_table: *mut  EfiSystemTable) -> ! {
    unsafe {
        let stdout = (*system_table).con_out;

        //  UTF-16 string for "Bootloader"
        let bootloader: [ u16; 11] = [
            'B' as u16, 'o' as u16, 'o' as u16, 't' as u16, 'l' as u16, 'o' as u16, 'a' as u16, 'd' as u16, 'e' as u16, 'r' as u16, 0
        ];
        ((*stdout).set_attribute)(stdout, ForegroundColors::LightGreen as usize | BackgroundColors::Black as usize); // Set the text color to light green on black background
         ((*stdout).output_string)(stdout, bootloader.as_ptr()); // Output the string "Bootloader" to the console
    }
    loop{}
}
