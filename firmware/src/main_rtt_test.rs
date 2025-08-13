#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::{Config};
use embassy_time::{Duration, Timer};
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    rtt_init_print!();
    rprintln!("STM32G4 RTT Test Starting!");
    
    let p = embassy_stm32::init(Config::default());
    rprintln!("Hardware initialized");

    // LED blink test
    let mut led = Output::new(p.PC13, Level::Low, Speed::Low);
    rprintln!("LED initialized");
    
    let mut counter = 0;
    loop {
        led.set_high();
        rprintln!("LED ON - Counter: {}", counter);
        Timer::after(Duration::from_millis(500)).await;
        
        led.set_low();
        rprintln!("LED OFF - Counter: {}", counter);
        Timer::after(Duration::from_millis(500)).await;
        
        counter += 1;
        
        if counter > 10 {
            rprintln!("Test completed successfully!");
            break;
        }
    }
    
    rprintln!("Entering infinite loop");
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
