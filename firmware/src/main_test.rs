#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::usb::Driver;
use embassy_stm32::{bind_interrupts, peripherals, usb, Config};
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::Builder;
use panic_halt as _;

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
});

// Static buffers for USB
static mut CONFIG_DESCRIPTOR: [u8; 256] = [0; 256];
static mut BOS_DESCRIPTOR: [u8; 256] = [0; 256];
static mut CONTROL_BUF: [u8; 64] = [0; 64];
static mut USB_STATE: State = State::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize hardware with default config
    let p = embassy_stm32::init(Config::default());

    // Initialize USB
    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);
    
    // Create embassy-usb Config
    let mut usb_config = embassy_usb::Config::new(0xc0de, 0xcafe);
    usb_config.manufacturer = Some("Test Device");
    usb_config.product = Some("Echo Test");
    usb_config.serial_number = Some("123");

    // Create embassy-usb DeviceBuilder using static buffers
    let mut builder = Builder::new(
        driver,
        usb_config,
        unsafe { &mut CONFIG_DESCRIPTOR },
        unsafe { &mut BOS_DESCRIPTOR },
        &mut [], // no msos descriptors
        unsafe { &mut CONTROL_BUF },
    );

    // Create CDC-ACM class
    let cdc_class = CdcAcmClass::new(&mut builder, unsafe { &mut USB_STATE }, 64);
    let usb_device = builder.build();

    // Spawn USB task
    spawner.spawn(usb_task(usb_device)).unwrap();

    // Spawn echo task
    spawner.spawn(echo_task(cdc_class)).unwrap();

    // Main loop - very fast blink to show we're alive
    let mut led = Output::new(p.PC13, Level::Low, Speed::Low);
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(25)).await;
        led.set_low();
        Timer::after(Duration::from_millis(25)).await;
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb_device: embassy_usb::UsbDevice<'static, Driver<'static, peripherals::USB>>) {
    usb_device.run().await;
}

#[embassy_executor::task]
async fn echo_task(
    mut cdc_class: CdcAcmClass<'static, Driver<'static, peripherals::USB>>,
) {
    // No initial delay - start immediately
    let mut buffer = [0u8; 64];
    
    loop {
        // Try to read data
        match cdc_class.read_packet(&mut buffer).await {
            Ok(n) if n > 0 => {
                // Immediate echo - just send back what we received
                let _ = cdc_class.write_packet(&buffer[..n]).await;
            }
            _ => {
                // Continue
            }
        }
        
        // Very short delay
        Timer::after(Duration::from_millis(1)).await;
    }
}
