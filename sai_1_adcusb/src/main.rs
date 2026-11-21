#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{gpio::{Level, Output, Speed}, pac::spi::vals::Sp};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async  fn main(_spawner: Spawner){
    let p = embassy_stm32::init(Default::default());
    info!("Hello world");

    let mut led = Output::new(p.PE3, Level::High, Speed::Low);

    loop{
        info!("LED on!");
        led.set_high();
        Timer::after_millis(500).await;

        info!("led_off!");
        led.set_low();
        Timer::after_millis(500).await;
    }
}
