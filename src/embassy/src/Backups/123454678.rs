#![no_std]
#![no_main]
#![allow(unused_imports)]

use core::panic::PanicInfo;
use embassy_executor::Spawner;
use embassy_time::Timer;
use log::info;

// USB driver
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_rp::{bind_interrupts, peripherals::USB};

// ADC
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig, InterruptHandler as AdcInterruptHandler};
// GPIO
use embassy_rp::gpio::Pull;

// TODO 1: Bind the ADC_IRQ_FIFO interrupt (be careful with the import names)
bind_interrupts!(struct Irqs {
    // Use for the serial over USB driver
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
    ADC_IRQ_FIFO => AdcInterruptHandler;
});

// The task used by the serial port driver over USB
#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // ---- Exercise 3 ----
    // Initialize peripherals
    let p = embassy_rp::init(Default::default());

    // Start the serial port over USB driver
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    // TODO 2: Create ADC
    let mut adc = Adc::new(p.ADC, Irqs, AdcConfig::default());

    let mut adc_pin = Channel::new_pin(p.PIN_26 , Pull::None);

    // TODO 3: Initialize photoresistor pin on ADC0

    loop {
        
        // TODO 4: Read a value from the ADC
        let level = adc.read(&mut adc_pin).await.unwrap();
        // TODO 5: Print the value to the console (over serial port)
        info!("Light sensor reading: {}", level);

        // TODO 6: Wait a while before reading again
        Timer::after_secs(1).await;
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
