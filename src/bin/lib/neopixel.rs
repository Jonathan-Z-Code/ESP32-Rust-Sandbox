//! # Welcome to the neopixel library source code!
//! - By: Jonathan Zurita
//! - Date: 11/18/2024
//! - Version: 0.1.0
//! 
//! ## What is the function of this library?
//! This library allows the user to call user-friendly functions
//! in order to manipulate WS2812 neopixels. 
//! 
//! ## IMPORTANT NOTES
//! Please edit the `pub const` variables as needed
//! - (e.g. change `NUM_LEDS` to 24 instead of default value of 8.)
//! 
//! ## Why do use a watchdog timer as an argument?
//! I decided to include a watchdog timer in order
//! to protect against possible lockups in DMA transmission.
//! Thankfully utilizing a WDT requires minimal overhead. :)
//! 
//! ## Release Notes
//! V0.1.0 - Released neopixel library to the public 



// user defined includes
use esp_hal::{
    delay::Delay,
    spi::{master::{self, SpiDmaBus}, SpiMode},
    peripherals::{self, Peripherals, SPI2, TIMG0},
    timer::timg::Wdt, Blocking
};
use core::result::Result::Ok;
use core::result::Result::Err;

// IMPORTANT NOTE: Please edit these variables in order to best fit your
// neopixel array! :)

/// Desired number of neopixels to control
pub const NUM_LEDS: usize = 8;
/// Number of color bits per neopixel
pub const PACKET_SIZE: usize = 24;
/// ARR_SIZE = NUM_LEDS * PACKET_SIZE
pub const ARR_SIZE: usize = NUM_LEDS * PACKET_SIZE;
/// SPI generate waveforms (LOGIC_1)
pub const NEOPIXEL_LOGIC_0: u8 = 0x80;
/// SPI generate waveforms (LOGIC_0)
pub const NEOPIXEL_LOGIC_1: u8 = 0xF8;
/// Essentially hex color code for black (no light)
const TURN_OFF: u32 = 0x000000;

////////////////////////////// START OF NEOPIXEL FUNCTIONS /////////////////////////////////////////////


/// # <<< Click on me to see function desc!
/// # Purpose:
/// Sets a specific neopixel to a desired color code
/// - Has error checking such as wrong color code size and wrong position argument given
pub fn neopixel_set_data(rgb_color: u32, position: usize, arr: &mut [u8 ; ARR_SIZE]) -> Result<u8, &str>{

    // error checking for RGB value
    if rgb_color > 0xFFFFFF {
        // color code exceeds RGB max value
        return Err("MAX VAL EXCEEDED FOR RGB VALUE");
    }

    // subtract one because we are indexing from 0!
    if position > (NUM_LEDS - 1) {
        return Err("EXCEEDED NEOPIXEL POSITION BOUNDS");
    }

    let mut bit_mask: u32 = 0x800000;
    let mut arr_idx = position * PACKET_SIZE;
    let mut result: u32;

    // iterate through every color bit in 24-bit RGB code
    for _i in 0..24 {    
        
        result = rgb_color & bit_mask;

        if result > 0 {
            arr[arr_idx] = NEOPIXEL_LOGIC_1;
        }
        else {
            arr[arr_idx] = NEOPIXEL_LOGIC_0;
        }

        arr_idx += 1; // increment to next element in array
        bit_mask >>= 1; // shift to next bit
    }

    return Ok(0);

}

/// # <<< Click on me to see function desc!
/// # Purpose:
/// Sets the entire user-defined neopixel array a particular color code
/// - Has error checking such as wrong color code size and wrong position argument given
pub fn neopixel_set_entire_data(rgb_color: u32, arr: &mut [u8 ; ARR_SIZE]) {

    for i in 0..NUM_LEDS {
        neopixel_set_data(rgb_color, i, arr).unwrap();
    }

}

/// # <<< Click on me to see function desc!
/// # Purpose:
/// Returns a mutable array that stores the data for the desired
/// number of neopixels (determine by `NUM_LEDS`)
pub fn neopixel_init_buffer() -> [u8; ARR_SIZE] {
    
    // init to all zeroes
    let mut arr: [u8 ; ARR_SIZE] = [0 ; ARR_SIZE];
    
    // final init array stage is to prepare entire array with NEOPIXEL_LOGIC_0
    for i in 0..ARR_SIZE {
        arr[i] = NEOPIXEL_LOGIC_0;
    }

    return arr;
        
}

pub fn spi_dma_send_breathing(spi: &mut SpiDmaBus<SPI2, esp_hal::spi::FullDuplexMode, Blocking>, 
                            arr: &mut [u8 ; ARR_SIZE], 
                            delay: &mut Delay, 
                            wdt: &mut Wdt<TIMG0>) 
{
    
    // declare rgb_val variable in order to synchronize what the color is for entire NeoPixel LED array
    let mut rgb_val: u32 = 0x00;    
    
    // increment up from d0 to d20 and back down from d20 to d1
    for _i in 0..20  {
        
        neopixel_set_entire_data(rgb_val, arr);
        
        spi_dma_send(spi, arr, wdt);
        
        delay.delay_millis(125 as u32);

        rgb_val += 1;

    }
    for _i in 0..20 {
        
        neopixel_set_entire_data(rgb_val, arr);
        
        spi_dma_send(spi, arr, wdt);
        
        delay.delay_millis(100 as u32);

        rgb_val -= 1;

    }
    
    // reset neoPixels back to zero state
    neopixel_set_entire_data(TURN_OFF, arr);
    spi_dma_send(spi, arr, wdt);

}

pub fn spi_dma_send(spi: &mut SpiDmaBus<SPI2, esp_hal::spi::FullDuplexMode, Blocking>, 
                arr: &mut [u8], wdt: &mut Wdt<TIMG0> ) 
{
    // feed watchdog when spi transfer is complete!
    match spi.write(arr) {
        Ok(_t) => wdt.feed(),
        Err(_e) => () ,
    }    
}