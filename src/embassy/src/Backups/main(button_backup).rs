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
use embassy_time::Duration;	
use embassy_time::Timer;
use gpio::{Level, Output};

#[embassy_executor::task]
async fn button_press(output:AnyPin, input:AnyPin){

    let mut led = Output::new(output, Level::High);
    let mut button = Input::new(input, Pull::Up);
    loop{
        button.wait_for_falling_edge().await;
        led.toggle();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let p = embassy_rp::init(Default::default());

    spawner.spawn(button_press(p.PIN_1.degrade(), p.PIN_5.degrade())).unwrap();

     // LED 1
	let mut pin_0 = Output::new(p.PIN_0, Level::Low);

    let mut var = 0;    
    loop {

        
        var = var + 1;
        if var < 20 {
            pin_0.toggle();
	
        Timer::after_millis(800).await;
        }
        else {
            pin_0.set_low();
        }
        
        
    }

    

    
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
