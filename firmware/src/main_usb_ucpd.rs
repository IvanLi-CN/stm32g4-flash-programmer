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
use rtt_target::{rprintln, rtt_init_print};

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
    rtt_init_print!();
    rprintln!("STM32G4 USB with UCPD Test Starting!");
    
    // Configure STM32 with UCPD dead battery support
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
        config.rcc.sys = Sysclk::HSI;
        config.rcc.mux.clk48sel = mux::Clk48sel::HSI48;
    }
    
    // CRITICAL: Enable UCPD dead battery support
    config.enable_ucpd1_dead_battery = true;
    rprintln!("UCPD dead battery support enabled");
    
    let p = embassy_stm32::init(config);
    rprintln!("Hardware initialized with UCPD");

    // Create the USB driver
    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);
    rprintln!("USB driver created");

    // Create embassy-usb Config
    let mut usb_config = embassy_usb::Config::new(0xc0de, 0xcafe);
    usb_config.manufacturer = Some("STM32G4");
    usb_config.product = Some("UCPD Test");
    usb_config.serial_number = Some("12345678");

    // Required for windows compatibility
    usb_config.device_class = 0xEF;
    usb_config.device_sub_class = 0x02;
    usb_config.device_protocol = 0x01;
    usb_config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder
    let mut builder = Builder::new(
        driver,
        usb_config,
        unsafe { &mut CONFIG_DESCRIPTOR },
        unsafe { &mut BOS_DESCRIPTOR },
        &mut [], // no msos descriptors
        unsafe { &mut CONTROL_BUF },
    );

    // Create CDC-ACM class
    let class = CdcAcmClass::new(&mut builder, unsafe { &mut USB_STATE }, 64);
    let usb = builder.build();
    rprintln!("USB device built");

    // Spawn tasks
    spawner.spawn(usb_task(usb)).unwrap();
    spawner.spawn(echo_task(class)).unwrap();
    spawner.spawn(led_task(p.PC13)).unwrap();
    rprintln!("All tasks spawned");
    
    // Main loop
    loop {
        Timer::after(Duration::from_secs(5)).await;
        rprintln!("Main loop heartbeat");
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb: embassy_usb::UsbDevice<'static, Driver<'static, peripherals::USB>>) {
    rprintln!("USB task started");
    usb.run().await;
}

#[embassy_executor::task]
async fn echo_task(mut class: CdcAcmClass<'static, Driver<'static, peripherals::USB>>) {
    rprintln!("Echo task started");
    
    // Wait for USB to be ready
    Timer::after(Duration::from_secs(3)).await;
    rprintln!("Echo task ready for data");
    
    let mut buf = [0; 64];
    loop {
        match class.read_packet(&mut buf).await {
            Ok(n) if n > 0 => {
                rprintln!("Received {} bytes: {:?}", n, &buf[..n]);
                // Echo back immediately
                match class.write_packet(&buf[..n]).await {
                    Ok(_) => rprintln!("Echoed {} bytes", n),
                    Err(e) => rprintln!("Echo failed: {:?}", e),
                }
            }
            Ok(_) => {
                // No data, continue
            }
            Err(e) => {
                rprintln!("Read error: {:?}", e);
                Timer::after(Duration::from_millis(100)).await;
            }
        }
        Timer::after(Duration::from_millis(1)).await;
    }
}

#[embassy_executor::task]
async fn led_task(pin: embassy_stm32::Peri<'static, peripherals::PC13>) {
    let mut led = Output::new(pin, Level::Low, Speed::Low);
    rprintln!("LED task started");
    
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(200)).await;
        led.set_low();
        Timer::after(Duration::from_millis(200)).await;
    }
}
