#![no_std]
#![no_main]
#![allow(unused_imports)]

use core::cell::RefCell;
use core::panic::PanicInfo;
use embassy_executor::Spawner;

// GPIO
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIN_0, PIN_12, PIN_13, PIN_14, PIN_15, PIN_16, PIN_17, SPI0};

// PWM
use embassy_rp::pwm::{Config as PwmConfig, Pwm};

// ADC
use embassy_rp::adc::{
    Adc, Async, Channel as AdcChannel, Config as AdcConfig, InterruptHandler as InterruptHandlerAdc,
};

// USB
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{bind_interrupts, peripherals::USB};
use log::info;

// Channel
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};

// Timer
use embassy_time::{Delay, Timer};

// Select futures
use embassy_futures::select::select;
use embassy_futures::select::Either::{First, Second};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    ADC_IRQ_FIFO => InterruptHandlerAdc;
});

// TODO 11:
// (Method 1) Use a different command enum that also encapsulates the command for changing the intensity.
// Hint: You can use something like this:
// ```rust
// enum LedCommand {
//     ChangeColor(Option<LedColor>),
//     ChangeIntensity(u16)
// }
// Don't forget to change the CHANNEL data type as well!
// ```
// (Method 2) Use another channel for changing the intensity, which will hold the value sampled by the ADC.

// TODO 2: Create an enum called LedColor
//         This will define what color the RGB LED should be
#[derive(Debug)]
enum LedColor {
    Red,
    Green,
    Blue,
}

// You can use this to declare the `compare_top` for each PWM
static TOP: u16 = 0x8000;

// TODO 3: Create a channel that can hold Option<LedColor>:
//         - Some(LedColor) - command for RGB LED to turn on and display the color LedColor
//         - None           - command for RGB LED to turn off
static INTENSITY_CHANNEL: Channel<ThreadModeRawMutex, u16, 64> = Channel::new();

// TODO 4: Create 4 separate tasks, one for each button.
//         Each task will wait for the button press and send an Option<LedColor> command over the channel depending on the button's function:
//         - button A: make the RGB LED red
//         - button B: make the RGB LED green
//         - button X: make the RGB LED blue
//         - button Y: turn the RGB LED off

// TODO 12: Create another task for sampling the potentiometer analog value and sending them over the channel as a ChangeIntensity command.
// You should wait a while in between samples (around 200ms should suffice).
// Your task should have 3 parameters: Adc, AdcChannel and Sender.
#[embassy_executor::task]
async fn potentiometer_read(
    mut adc: Adc<'static, Async>,
    mut potentiometer: AdcChannel<'static>,
) {
    loop {
        info!("before read");
        Timer::after_millis(200).await;
        let value = adc.read(&mut potentiometer).await.unwrap();
        info!("after read");
        info!("Potentiometer reading: {}", value);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let adc = Adc::new(peripherals.ADC, Irqs, AdcConfig::default());
    let potentiometer = AdcChannel::new_pin(peripherals.PIN_26, Pull::None);

    spawner
        .spawn(potentiometer_read(
            adc,
            potentiometer,
        ))
        .unwrap();
        loop{

        }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}