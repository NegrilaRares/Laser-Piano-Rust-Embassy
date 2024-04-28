#![no_std]
#![no_main]
#![allow(unused_imports, unused_variables, unused_mut)] // i removed allowing dead_code as i did not know what that entailed

use core::cell::RefCell;
use core::panic::PanicInfo;
use std::process::Output;
use embassy_executor::Spawner;
// use rp2040_hal::sio::Sio;

//GPIO
use embassy_rp::gpio::{
    AnyPin, 
    self, 
    Input, 
    Pull, 
    Pin,
    Level,
    Output, 
};
use embassy_rp::peripherals::{
    PIN_0, 
    PIN_2,
    PIN_3,
    PIN_4,
    PIN_6,
    PIN_12, 
    PIN_13, 
    PIN_14, 
    PIN_15, 
    PIN_16, 
    PIN_17, 
    PIN_26,
    ADC,
    USB,
    SPI0,
};

// PWM
use embassy_rp::pwm::{
    Config as ConfigPWM, 
    Pwm,
};

use embassy_rp::adc::{
    Adc, 
    Async, 
    Channel as ChannelADC, 
    Config as ConfigADC, 
    InterruptHandler as InterruptHandlerADC,
};

// USB
use embassy_rp::usb::{
    Driver, 
    InterruptHandler,
};
use embassy_rp::bind_interrupts;
use log::info;

// Channel
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{
    Channel, 
    Receiver, 
    Sender
};

// Timer
use embassy_time::{
    Delay, 
    Timer, 
    Duration,
};


// Select futures
use embassy_futures::select::select;
use embassy_futures::select::Either::{First, Second};

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

const DISPLAY_FREQ: u32 = 64_000_000;bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    ADC_IRQ_FIFO => InterruptHandlerADC;
});

// enum LedColor {
//     Red,
//     Green,
//     Blue,
// }

pub enum VolLvl{
    ONE,
    TWO,
    THREE,
    FOUR,
    FIVE,
    SIX,
    SEVEN,
    EIGHT,
    NINE,
    TEN,
}

pub enum OctLvl{
    ONE,
    TWO,
    THREE,
}

static VOLUME_CHANNEL: Channel<ThreadModeRawMutex, u16, 64> = Channel::new();
static OCTAVE_CHANNEL: Channel<ThreadModeRawMutex, u16, 64> = Channel::new();

// #[embassy_executor::task]
// async fn blink_rgb(
//     mut red:Output<'static, PIN_4>, 
//     mut green:Output<'static, PIN_3>, 
//     mut blue:Output<'static, PIN_2>
// ) {
//     loop {
//         red.set_high();
//         blue.set_high();
//         green.set_low();
//         Timer::after_secs(2).await;
//         red.set_low();
//         blue.set_high();
//         green.set_high();
//         Timer::after_secs(2).await;
//         blue.set_low();
//         green.set_low();
//         red.set_high();
//         Timer::after_secs(2).await;
//     }
// }

// #[embassy_executor::task]
// async fn clock_signal(mut pin: Output<'static, PIN_6>) {
//     // Desired frequency of clock signal (adjust as needed)
//     let desired_frequency: u64  = 1_000_000; // 1 MHz

//     // Calculate the period of the clock signal
//     let period_ns: u64  = 1_000_000_000 / desired_frequency;

//     // Set the duty cycle (half of the period)
//     let half_period_ns: u64 = period_ns / 2;

//     loop {
//         // Set the pin high (rising edge)
//         pin.set_high();
//         // Delay for half of the period
//         Timer::after(Duration::from_nanos(half_period_ns)).await;
//         // Set the pin low (falling edge)
//         pin.set_low();
//         // Delay for the other half of the period
//         Timer::after(Duration::from_nanos(half_period_ns)).await;
//     }
// }




#[embassy_executor::task]
async fn vol_oct_input(
    //switch:Input<'static, PIN_12>, 
    mut potentiometer: ChannelADC<'static>, 
    mut adc: Adc<'static, Async>, 
    channel_sender_volume: Sender<'static, ThreadModeRawMutex, u16, 64>,
    channel_sender_octave: Sender<'static, ThreadModeRawMutex, u16, 64>,  
    clk:Output<'static, PIN_TEMPLATE>,
    data:Output<'static, PIN_TEMPLATE>,
    rclk:Output<'static, PIN_TEMPLATE>,
    reset:Output<'static, PIN_TEMPLATE>
) {
    loop{

        info!("Waiting!");
        let value = adc.read(&mut potentiometer).await.unwrap(); // (19)20(21) - 4095  
        info!("Result: {:?}", value);
        match value {
            0..=426 => {
                info!("Value is in range 1: <0, 426>");
                let mut max = 1;
                let mut i = 0;
                reset.set_low();
                while i < max
                {
                    data.set_high();
                    Timer::after_nanos(10000).await;
                    data.set_low();
                    Timer::after_nanos(10000).await;
                    clk.set_high();
                    Timer::after_nanos(10000).await;
                    clk.set_low();
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(10000).await;
                latch.set_low();
            } ,
            427..=833 => {
                info!("Value is in range 2: <427, 833>");
                let mut max = 2;
                let mut i = 0;
                reset.set_low();
                while i < max
                {
                    data.set_high();
                    Timer::after_nanos(10000).await;
                    data.set_low();
                    Timer::after_nanos(10000).await;
                    clk.set_high();
                    Timer::after_nanos(10000).await;
                    clk.set_low();
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(10000).await;
                latch.set_low();
            },
            834..=1240 => {
                info!("Value is in range 3: <834, 1240>");
                let mut max = 3;
                let mut i = 0;
                reset.set_low();
                while i < max
                {
                    data.set_high();
                    Timer::after_nanos(10000).await;
                    data.set_low();
                    Timer::after_nanos(10000).await;
                    clk.set_high();
                    Timer::after_nanos(10000).await;
                    clk.set_low();
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(10000).await;
                latch.set_low();
            },
            1241..=1647 => {
                info!("Value is in range 4: <1241, 1647>");
                let mut max = 4;
                let mut i = 0;
                reset.set_low();
                while i < max
                {
                    data.set_high();
                    Timer::after_nanos(10000).await;
                    data.set_low();
                    Timer::after_nanos(10000).await;
                    clk.set_high();
                    Timer::after_nanos(10000).await;
                    clk.set_low();
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(10000).await;
                latch.set_low();
            },
            1648..=2054 => {
                info!("Value is in range 5: <1648, 2054>");
                let mut max = 5;
                let mut i = 0;
                reset.set_low();
                while i < max
                {
                    data.set_high();
                    Timer::after_nanos(10000).await;
                    data.set_low();
                    Timer::after_nanos(10000).await;
                    clk.set_high();
                    Timer::after_nanos(10000).await;
                    clk.set_low();
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(10000).await;
                latch.set_low();
            },
            2055..=2461 => {
                info!("Value is in range 6: <2055, 2461>");
                let mut max = 6;
                let mut i = 0;
                reset.set_low();
                while i < max
                {
                    data.set_high();
                    Timer::after_nanos(10000).await;
                    data.set_low();
                    Timer::after_nanos(10000).await;
                    clk.set_high();
                    Timer::after_nanos(10000).await;
                    clk.set_low();
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(10000).await;
                latch.set_low();
            },
            2462..=2868 => {
                info!("Value is in range 7: <2462, 2868>");
                let mut max = 7;
                let mut i = 0;
                reset.set_low();
                while i < max
                {
                    data.set_high();
                    Timer::after_nanos(10000).await;
                    data.set_low();
                    Timer::after_nanos(10000).await;
                    clk.set_high();
                    Timer::after_nanos(10000).await;
                    clk.set_low();
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(10000).await;
                latch.set_low();
            },
            2869..=3275 => {
                info!("Value is in range 8: <2869, 3275>");
                let mut max = 8;
                let mut i = 0;
                reset.set_low();
                while i < max
                {
                    data.set_high();
                    Timer::after_nanos(10000).await;
                    data.set_low();
                    Timer::after_nanos(10000).await;
                    clk.set_high();
                    Timer::after_nanos(10000).await;
                    clk.set_low();
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(10000).await;
                latch.set_low();
            },
            3276..=3682 => {
                info!("Value is in range 9: <3276, 3682>");
                let mut max = 9;
                let mut i = 0;
                reset.set_low();
                while i < max
                {
                    data.set_high();
                    Timer::after_nanos(10000).await;
                    data.set_low();
                    Timer::after_nanos(10000).await;
                    clk.set_high();
                    Timer::after_nanos(10000).await;
                    clk.set_low();
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(10000).await;
                latch.set_low();
            },
            3683..=4095 => {
                info!("Value is in range 10: <3683, 4095>");
                let mut max = 10;
                let mut i = 0;
                reset.set_low();
                while i < max
                {
                    data.set_high();
                    Timer::after_nanos(10000).await;
                    data.set_low();
                    Timer::after_nanos(10000).await;
                    clk.set_high();
                    Timer::after_nanos(10000).await;
                    clk.set_low();
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(10000).await;
                latch.set_low();
            },
            _ => info!("Value is not in any defined range"),
        }

        channel_sender_volume.send(value).await;
        Timer::after_millis(100).await;
    }
}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let p = embassy_rp::init(Default::default());

    // let clock = Output::new(p.PIN_6, Level::Low);
    // spawner.spawn(clock_signal(clock)).unwrap();


    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    let mut blue = Output::new(p.PIN_2, Level::Low);
    let mut green = Output::new(p.PIN_3, Level::Low);
    let mut red = Output::new(p.PIN_4, Level::Low);

    //let mut switch = Input::new(p.PIN_0, Pull::None);

    let mut adc = Adc::new(p.ADC, Irqs, ConfigADC::default());
    let mut potentiometer = ChannelADC::new_pin(p.PIN_26, Pull::None);

    let mut clock = Output::new(p.PIN_TEMPLATE, Level::Low);
    let mut in_data = Output::new(p.PIN_TEMPLATE, Level::Low);
    let mut latch = Output::new(p.PIN_TEMPLATE, Level::Low);
    let mut clear = Output::new(p.PIN_TEMPLATE, Level::Low);
    
    // spawner.spawn(blink_rgb(red, green,blue)).unwrap();
    spawner.spawn(
        vol_oct_input(
            potentiometer, 
            adc, 
            VOLUME_CHANNEL.sender(), 
            OCTAVE_CHANNEL.sender(), 
            clock, 
            in_data, 
            latch,
            clear
        )
    ).unwrap();
    
    
    let mut volume: u16;
    let mut octave: u16;

    loop {
        let select = select(VOLUME_CHANNEL.receive(), OCTAVE_CHANNEL.receive()).await;

        match select {
            First(volume_value) =>{

                volume = volume_value;
                if volume > 3000
                {
                    red.set_low();
                    blue.set_high();
                    green.set_high();
                }
                else 
                {
                    red.set_high();
                    blue.set_low();
                    green.set_low();
                }
                
            }
            Second(octave_value) =>{

            }
        }



    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}