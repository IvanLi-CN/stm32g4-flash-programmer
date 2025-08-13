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
    let p = embassy_stm32::init(Config::default());

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial example");
    config.serial_number = Some("12345678");

    // Required for windows compatibility.
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    let mut builder = Builder::new(
        driver,
        config,
        unsafe { &mut CONFIG_DESCRIPTOR },
        unsafe { &mut BOS_DESCRIPTOR },
        &mut [], // no msos descriptors
        unsafe { &mut CONTROL_BUF },
    );

    // Create classes on the builder.
    let class = CdcAcmClass::new(&mut builder, unsafe { &mut USB_STATE }, 64);

    // Build the builder.
    let usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let echo_fut = echo_task(class);

    // LED task
    let led_fut = led_task(p.PC13);

    // Run everything concurrently.
    embassy_futures::join::join3(usb_fut, echo_fut, led_fut).await;
}

async fn echo_task(mut class: CdcAcmClass<'static, Driver<'static, peripherals::USB>>) {
    loop {
        class.wait_connection().await;
        
        let _ = class.write_packet(b"Hello from STM32G4!\r\n").await;
        
        let mut buf = [0; 64];
        loop {
            let n = match class.read_packet(&mut buf).await {
                Ok(n) => n,
                Err(_) => break,
            };

            let data = &buf[..n];
            let _ = class.write_packet(data).await;
        }
    }
}

async fn led_task(pin: peripherals::PC13) {
    let mut led = Output::new(pin, Level::High, Speed::Low);
    
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(300)).await;
        led.set_low();
        Timer::after(Duration::from_millis(300)).await;
    }
}
