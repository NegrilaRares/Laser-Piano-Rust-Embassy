#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::Duration;	
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
	let peripherals = embassy_rp::init(Default::default());

	let mut pin = Output::new(peripherals.PIN_0, Level::Low);
	
    let mut value = 1;
    loop {
        value = 1 - value;
        if value == 1
        {
            pin.set_high();
        }
        else if value == 0
        {
            pin.set_low();
        }
		

		let delay = Duration::from_secs(1);
		Timer::after(delay).await;
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
