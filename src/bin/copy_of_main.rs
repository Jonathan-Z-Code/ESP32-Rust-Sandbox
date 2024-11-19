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

use esp_println::println;



#[entry]
fn main() -> ! {

    //////////////////////// INIT PERIPHERAL AND IO MAPS //////////////////////////////////////////////////////
    
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let str = esp_hal::reset::get_reset_reason();
    println!("Reset Reason [Debug Purposes] => {:?}", str);

    // Initialize the Delay peripheral
    let mut delay = Delay::new();    


    //////////////////////////  INIT WDT   ////////////////////////////////////////////////////////////////////////////
  
    // initialize TIM0 group and watchdog timer  
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let mut wdt = timg0.wdt;
    
    // timeout set at 5 seconds and enable WDT
    wdt.set_timeout(5_000.millis());
    wdt.enable();
    // reset wdt timer (starts back at 5 second value)
    wdt.feed();


    //////////////////////// INIT SPI BUS AND PASS INTO FUNCTION //////////////////////////////////////////////////////
    
    // initialize the neopixel buffer with all LOGIC_0's in the buffer
    let mut arr: [u8 ; neopixel::ARR_SIZE] = neopixel::neopixel_init_buffer();

    let sclk = io.pins.gpio18;
    let miso = io.pins.gpio19;
    let mosi = io.pins.gpio23;
    let cs = io.pins.gpio5;

    let (rx_buf, rx_desc, tx_buf, tx_desc) = esp_hal::dma_buffers!(neopixel::ARR_SIZE);

    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.spi2channel;
    
    // creates a SpiDma Instance
    let spi = master::Spi::new(peripherals.SPI2, 4000.kHz(), esp_hal::spi::SpiMode::Mode0)
        .with_sck(sclk)
        .with_mosi(mosi)
        .with_miso(miso)
        .with_cs(cs)
        .with_dma(
            dma_channel.configure(false, DmaPriority::Priority0)
        );

    // now create a SpiDmaBus instance given the SpiDma
    let dma_rx_buf = DmaRxBuf::new(rx_desc, rx_buf).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_desc, tx_buf).unwrap();
    let mut spi_dma = spi.with_buffers(dma_rx_buf, dma_tx_buf);

    // sets the RGB value for a desired NeoPixel LED (indexes from 0)!!
    //neopixel_set_data(0x00000F, 7, &mut arr);

    // call the SPI dma transfer functions
    // also include the pass watchdog timer test
    neopixel::spi_dma_send(&mut spi_dma, &mut arr, &mut wdt);
    neopixel::spi_dma_send_breathing(&mut spi_dma, &mut arr, &mut delay, &mut wdt);

    // init BLUE LED GPIO pin
    let mut led = Output::new(io.pins.gpio2, Level::Low);


    //////////////////////// INIT ADC AND PASS INTO FUNCTION!! //////////////////////////////////////////////////////
    
    // you can also initialize generic GPIO pins
    let analog_pin = io.pins.gpio36;

    // ADC configuration using the GPIO32 generic init 
    let mut adc1_config = AdcConfig::new(); // start ADC1 config process
    let mut adc1_pin = adc1_config.enable_pin(analog_pin, Attenuation::Attenuation11dB); // init GPIO32 to be ADC1 input pin
    let mut adc1 = Adc::new(peripherals.ADC1, adc1_config); // init the ADC1 module for use

    // test function passing in ADC parameters as mutable pointers
    //let mut adc1_val = test_function2(&mut adc1, &mut adc1_pin);


    //////////////////////// START OF INFINITE LOOP!! ///////////////////////////////////////////////////////////////
    
    loop {
        
       // test_function(&mut led);
        println!("test string");
        //adc1_val = test_function2(&mut adc1, &mut adc1_pin);

        // read ADC1_Channel0 value
        let adc1_val = nb::block!(adc1.read_oneshot(&mut adc1_pin)).unwrap();
        println!("{}", adc1_val);
        delay.delay_millis(250 as u32);
        
        // set ADC1_Channel0 hex value into an RGB color for neoPixel!
        neopixel::neopixel_set_entire_data(adc1_val as u32 , &mut arr);
        neopixel::spi_dma_send(&mut spi_dma, &mut arr, &mut wdt);

        // Error testing if you want to (wrong RGB value or wrong neoPixel position)
        // neopixel::neopixel_set_data(0xFFFFFFFF, (neopixels::NUM_LEDS + 1), &mut arr).unwrap();
    
    }

    
    //////////////////////// END OF MAIN LOOP ///////////////////////////////////////////////////////////////
    
}


// IMPROVING ROBUSTNESS
//
// If system goes into unrecoverable state:
// esp_hal::reset::software_reset_cpu();
// esp_hal::reset::software_reset();