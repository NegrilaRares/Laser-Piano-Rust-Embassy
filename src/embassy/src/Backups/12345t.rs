#![no_std]
#![no_main]

use core::{
    panic::PanicInfo,
};

use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{
        AnyPin, 
        self, 
        Input, 
        Pull, 
        Pin, 
    },
    peripherals::{PIN_0, PIN_1, PIN_5},
};
use embassy_rp::pwm::Pwm;
use embassy_rp::pwm::Config as ConfigPwm;
use embassy_time::Duration;	
use embassy_time::Timer;
use gpio::{Level, Output};

#[embassy_executor::task]
async fn button_press(output:AnyPin, input:AnyPin){

    let mut led = Output::new(output, Level::High);
    let mut button = Input::new(input, Pull::Up);
    loop{
        button.wait_for_rising_edge().await;
        led.toggle();
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
	let p = embassy_rp::init(Default::default());
    let mut config: ConfigPwm = Default::default();

    config.top = 0x8000; 
    config.compare_a = config.top / 2;

    let mut pwm = Pwm::new_output_a(p.PWM_CH0,p.PIN_0,config.clone());

    config.compare_a += 100;
    pwm.set_config(&config);

    loop {}
}

    

    

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
