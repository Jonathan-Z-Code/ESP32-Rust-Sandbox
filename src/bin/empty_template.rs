#![no_std]
#![no_main]

mod lib;
use lib::neopixel;

const ADC1_0_GPIO: u8 = 36;

use esp_backtrace as _;

use esp_hal::{
    analog::adc::{self, Adc, AdcConfig, Attenuation}, 
    gpio::{GpioPin, Io, Level, Output}, 
    peripheral::{self, Peripheral}, 
    peripherals::{ADC1, SPI2, TIMG0}, 
    prelude::*, 
    timer::timg::{self, Wdt,TimerGroup}, Blocking, Async
};

use esp_hal::{
    delay::Delay,
    dma::{Dma, DmaPriority, DmaRxBuf, DmaTxBuf},
    spi::{master, SpiMode}
};


#[entry]
fn main() -> ! {

    loop {}
}


// IMPROVING ROBUSTNESS
//
// If system goes into unrecoverable state:
// esp_hal::reset::software_reset_cpu();
// esp_hal::reset::software_reset();