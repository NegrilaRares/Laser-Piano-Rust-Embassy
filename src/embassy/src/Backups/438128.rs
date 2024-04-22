#![no_std]
#![no_main]
#![allow(unused_imports, unused_variables)]

use core::panic::PanicInfo;

use embassy_executor::Spawner;

use embassy_time::Timer;
use embassy_time::Duration;
use log::info;

// USB driver
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{bind_interrupts, peripherals::USB};

// ADC
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig, InterruptHandler as InterruptHandlerAdc};

// GPIO
use embassy_rp::gpio::Pull;

// PWM
use embassy_rp::pwm::{Config as PwmConfig, Pwm};

// TODO 4: Bind ADC interrupt
bind_interrupts!(struct Irqs {
  ADC_IRQ_FIFO => InterruptHandlerAdc;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // TODO 1: Initialize peripherals
    let peripherals = embassy_rp::init(Default::default());

    // TODO 2: Create initial config for PWM
    let mut config: PwmConfig = Default::default();
    config.top = 0x8000;
    config.compare_a = config.top / 4;

    // TODO 3: Create PWM
    let mut pwm = Pwm::new_output_a(peripherals.PWM_CH0, peripherals.PIN_0, config.clone());

    // TODO 5: Create ADC
    let mut adc = Adc::new(peripherals.ADC, Irqs, AdcConfig::default());

    // TODO 6: Initialize photoresistor pin
    let mut adc_pin = Channel::new_pin(peripherals.PIN_26, Pull::None);

    loop {
        // TODO 7: Read the value of ADC
        let level = adc.read(&mut adc_pin).await.unwrap();

        // TODO 8: Set the duty cycle according to the value of the photoresistor (the brighter the room is, the less bright the led is)
        if level > 100 {
          config.compare_a = 0;
          pwm.set_config(&config);
        } else {
          config.compare_a = config.top;
          pwm.set_config(&config);
        }

        // TODO 9: Wait a bit before reading another value
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}