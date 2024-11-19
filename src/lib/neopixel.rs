// user defined includes
use esp_hal::{
    delay::Delay,
    dma::{Dma, DmaPriority, DmaRxBuf, DmaTxBuf},
    spi::{master::{self, SpiDmaBus}, SpiMode},
    peripherals::{self, Peripherals, ADC1, SPI2, TIMG0},
    timer::timg::Wdt, Blocking
};

// IMPORTANT NOTE: Please edit these variables in order to best fit your
// neopixel array! :)
pub const NUM_LEDS: usize = 8;
pub const PACKET_SIZE: usize = 24;
pub const ARR_SIZE: usize = NUM_LEDS * PACKET_SIZE;
pub const NEOPIXEL_LOGIC_0: u8 = 0x80;
pub const NEOPIXEL_LOGIC_1: u8 = 0xF8;
const TURN_OFF: u32 = 0x00;

////////////////////////////// START OF NEOPIXEL FUNCTIONS /////////////////////////////////////////////

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

pub fn neopixel_set_entire_data(rgb_color: u32, arr: &mut [u8 ; ARR_SIZE]) {

    for i in 0..NUM_LEDS {
        neopixel_set_data(rgb_color, i, arr).unwrap();
    }

}

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
        Ok(t) => wdt.feed(),
        Err(e) => () ,
    }
    
}