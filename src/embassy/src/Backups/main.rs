#![no_std]
#![no_main]
#![allow(unused_imports, dead_code)]

use core::panic::PanicInfo;

use embassy_executor::Spawner;

// GPIO
use embassy_rp::gpio::{Input, Pull};

// PWM
use embassy_rp::pwm::{Config as PwmConfig, Pwm};

#[derive(PartialEq, Copy, Clone)]
enum LedColor {
    Red,
    Green,
    Blue,
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // TODO 1: Initialize peripherals
    let p = embassy_rp::init(Default::default());

    // TODO 2: Create configuration for red LED
    let mut config_red: PwmConfig = Default::default();
    config_red.top = 0x8000;
    config_red.compare_a = 0;

    // TODO 3: Create configuration for green LED
    let mut config_green: PwmConfig = Default::default();
    config_green.top = 0x8000;
    config_green.compare_a = 0;

    // TODO 4: Create configuration for blue LED
    let mut config_blue: PwmConfig = Default::default();
    config_blue.top = 0x8000;
    config_blue.compare_a = 0;

    // TODO 5: Initialize PWM for red LED
    let mut pwm_red = Pwm::new_output_a(p.PWM_CH0, p.PIN_0, config_red.clone());

    // TODO 6: Initialize PWM for green LED
    let mut pwm_green = Pwm::new_output_a(p.PWM_CH1, p.PIN_2, config_green.clone());

    // TODO 7: Initialize PWM for blue LED
    let mut pwm_blue = Pwm::new_output_a(p.PWM_CH2, p.PIN_4, config_blue.clone());

    // TODO 8: Initialize button
    let mut button = Input::new(p.PIN_12, Pull::Up);

    // Variable for keeping track of current color
    config_red.compare_a = config_red.top;
    pwm_red.set_config(&config_red);
    let mut color: LedColor = LedColor::Red;

    loop {
        // TODO 9: Wait for button press
        button.wait_for_rising_edge().await;
        // TODO 10: Check what the current color is and change configurations of PWMs to match next color
        match color {
            LedColor::Red => {
                config_red.compare_a = 0;
                config_blue.compare_a = 0;
                config_green.compare_a = config_green.top;
                color = LedColor::Green;
                // to do
            }
            LedColor::Green => {
                config_red.compare_a = 0;
                config_blue.compare_a = config_blue.top;
                config_green.compare_a = 0;
                color = LedColor::Blue;
                // to do
            }
            LedColor::Blue => {
                config_red.compare_a = config_red.top;
                config_blue.compare_a = 0;
                config_green.compare_a = 0;
                color = LedColor::Red;
                // to do
            }
        }
        // TODO 11: Set new configurations for each PWM

        // TODO 12: Modify variable that keeps track of color
        pwm_red.set_config(&config_red);
        pwm_green.set_config(&config_green);
        pwm_blue.set_config(&config_blue);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
