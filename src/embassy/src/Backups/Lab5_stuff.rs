#![no_std]
#![no_main]
#![allow(unused_imports)]

use core::panic::PanicInfo;
use embassy_executor::Spawner;

// GPIO
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::peripherals::{PIN_12, PIN_13};
use embassy_rp::pwm::{Config as PwmConfig, Pwm};

// Channel
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Sender};

// USB driver
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{bind_interrupts, peripherals::USB};
use log::info;

bind_interrupts!(struct Irqs {
    // Use for the serial over USB driver
    USBCTRL_IRQ => InterruptHandler<USB>;
});

// TODO 2: Create an enum called LedCommand.
//         A LED command can either be: increase or decrease intensity

#[derive(PartialEq)]
enum LedCommand {
    IncreaseIntensity,
    DecreaseIntensity,
}

// TODO 3: Change the data type that can be sent over the channel to LedCommand
static CHANNEL: Channel<ThreadModeRawMutex, LedCommand, 64> = Channel::new();

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::task]
async fn button_a_pressed(mut button_a: Input<'static, PIN_12>) {
    loop {
        button_a.wait_for_falling_edge().await;
        // TODO 4: Send the correct LedCommand for the A button
        CHANNEL.send(LedCommand::IncreaseIntensity).await;
    }
}

// TODO 6: Create another task for button B, similar to the task for button A.

#[embassy_executor::task]
async fn button_b_pressed(mut button_b: Input<'static, PIN_13>) {
    loop {
        button_b.wait_for_falling_edge().await;
        
        CHANNEL.send(LedCommand::DecreaseIntensity).await;
    }
}

// TODO 1: This is a simple example of a button task communicating with the main task through a channel.
//         Test it out!
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    // Start the serial port over USB driver
    let driver = Driver::new(peripherals.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    let button_a = Input::new(peripherals.PIN_12, Pull::Up);
    let button_b = Input::new(peripherals.PIN_13, Pull::Up);
    // TODO 5: Declare button B

    spawner.spawn(button_a_pressed(button_a)).unwrap();
    // TODO 7: Spawn the button B task
    spawner.spawn(button_b_pressed(button_b)).unwrap();

    // TODO 8: Create the PWM for the LED

    let mut config: PwmConfig = Default::default();
    config.top = 0x8000;
    config.compare_a = 0;

    // Initialize PWM
    let mut pwm = Pwm::new_output_a(peripherals.PWM_CH1, peripherals.PIN_2, config.clone());
    let interval = config.top / 10;

    loop {
        let _value = CHANNEL.receive().await;
        // TODO 9: Check what LedCommand was received over the channel.
        //         Depending on the LedCommand, increase or decrease the intensity of the LED.
        if _value == LedCommand::DecreaseIntensity
        {
            config.compare_a -= interval;
            info!("Button B was pressed!");
        }
        else if _value == LedCommand::IncreaseIntensity
        {
            config.compare_a += interval;
            info!("Button A was pressed!");
        }
        // TODO 10: Set the new configuration of the PWM.
        if config.compare_a <= config.top
        {
            pwm.set_config(&config);
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}