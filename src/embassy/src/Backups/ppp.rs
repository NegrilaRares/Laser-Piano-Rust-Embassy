#![no_std]
#![no_main]
#![allow(unused_imports, dead_code, unused_variables, unused_mut)]

use core::cell::RefCell;
use core::panic::PanicInfo;
use embassy_executor::Spawner;

// GPIO
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIN_0, PIN_12, PIN_13, PIN_14, PIN_15, PIN_16, PIN_17, PIN_26,ADC,SPI0};

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

// Display
use core::fmt::Write;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_rp::spi;
use embassy_rp::spi::{Blocking, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embedded_graphics::mono_font::iso_8859_16::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::text::Text;
use heapless::String;
use embassy::SPIDeviceInterface;
use st7789::{Orientation, ST7789};

const DISPLAY_FREQ: u32 = 64_000_000;bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    ADC_IRQ_FIFO => InterruptHandlerAdc;
});

// TODO 11
// (Method 1) Use a different command enum that encapsulates both the command for changing the color AND changing the intensity.
// Hint: You can use something like this:
// ```rust
// enum LedCommand {
//     ChangeColor(Option<LedColor>),
//     ChangeIntensity(u16)
// }
// Don't forget to change the CHANNEL data type to use LedCommand instead of just LedColor!
// ```
// (Method 2) Use another channel for changing the intensity, which will hold the value sampled by the ADC.
//            The ADC sampling task will use this new channel instead, and the button tasks will continue to use the old channel.

enum LedCommand {
    ChangeColor(Option<LedColor>),
    ChangeIntensity(u16)
}

// TODO 2: Create an enum called LedColor.
//         This is the datatype that will be sent over the channel and will define what color the RGB LED should be.
enum LedColor {
    Red,
    Green,
    Blue,
}

// You can use this to declare the `compare_top` for each PWM
static TOP: u16 = 0x8000;


// TODO 3: Create a channel that can hold Option<LedColor>:
//         - Some(LedColor) - command for RGB LED to turn on and display the color LedColor;
//         - None           - command for RGB LED to turn off.

static CHANNEL: Channel<ThreadModeRawMutex, LedCommand, 64> = Channel::new();

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

// TODO 4: Create 4 separate tasks, one for each button.
//         Each task will wait for the button press and send an Option<LedColor> command over the channel depending on the button's function:
//         - button A: make the RGB LED red;
//         - button B: make the RGB LED green;
//         - button X: make the RGB LED blue;
//         - button Y: turn the RGB LED off.

#[embassy_executor::task]
async fn button_a_pressed(mut button_a: Input<'static, PIN_12>) {
    loop {
        button_a.wait_for_falling_edge().await;
        CHANNEL.send(LedCommand::ChangeColor(Some(LedColor::Red))).await;
    }
}


#[embassy_executor::task]
async fn button_b_pressed(mut button_b: Input<'static, PIN_13>) {
    loop {
        button_b.wait_for_falling_edge().await;
        CHANNEL.send(LedCommand::ChangeColor(Some(LedColor::Green))).await;
    }
}


#[embassy_executor::task]
async fn button_x_pressed(mut button_x: Input<'static, PIN_14>) {
    loop {
        button_x.wait_for_falling_edge().await;
        CHANNEL.send(LedCommand::ChangeColor(Some(LedColor::Blue))).await;
    }
}


#[embassy_executor::task]
async fn button_y_pressed(mut button_y: Input<'static, PIN_15>) {
    loop {
        button_y.wait_for_falling_edge().await;
        CHANNEL.send(LedCommand::ChangeColor(None)).await;
    }
}


// TODO 12: Create another task for sampling the potentiometer analog value and sending them over the channel.
// You should wait a while in between samples (around 200ms should suffice).
// Your task should have 2 parameters: Adc (the ADC driver itself) and AdcChannel (the potentiometer).

// #[embassy_executor::task]
// async fn data(mut potentiometer: AdcChannel<'static>,mut adc: Adc<'static,Async> ) {
//     loop {
//         Timer::after_millis(200).await;
//         let value = adc.read(&mut potentiometer).await.unwrap();
        
//         CHANNEL.send(LedCommand::ChangeIntensity(value));

//     }
// }


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    // The USB driver, for serial debugging, you might need it ;)
    let driver = Driver::new(peripherals.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    // ------------------------ DISPLAY ----------------------------

    // FONT STYLE
    let mut style = MonoTextStyle::new(&FONT_10X20, Rgb565::GREEN);
    style.set_background_color(Some(Rgb565::BLACK));

    // ************** Display initialization - DO NOT MODIFY! *****************
    let miso = peripherals.PIN_4;
    let display_cs = peripherals.PIN_17;
    let mosi = peripherals.PIN_19;
    let clk = peripherals.PIN_18;
    let rst = peripherals.PIN_0;
    let dc = peripherals.PIN_16;
    let mut display_config = spi::Config::default();
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;

    // Init SPI
    let spi: Spi<'_, _, Blocking> =
        Spi::new_blocking(peripherals.SPI0, clk, mosi, miso, display_config.clone());
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));

    let display_spi = SpiDeviceWithConfig::new(
        &spi_bus,
        Output::new(display_cs, Level::High),
        display_config,
    );

    let dc = Output::new(dc, Level::Low);
    let rst = Output::new(rst, Level::Low);
    let di = SPIDeviceInterface::new(display_spi, dc);

    // Init ST7789 LCD
    let mut display = ST7789::new(di, rst, 240, 240);
    display.init(&mut Delay).unwrap();
    display.set_orientation(Orientation::Portrait).unwrap();
    display.clear(Rgb565::BLACK).unwrap();
    // ************************************************************************

    // Clear display
    display.clear(Rgb565::BLACK).unwrap();

    // ------------------------------------------------------------------------

    // (START OF EXERCISE 3)
    // TODO 1: Declare buttons A, B, X, Y (check the back of the Pico Explorer for the pins)
    let button_a = Input::new(peripherals.PIN_12, Pull::Up);
    let button_b = Input::new(peripherals.PIN_13, Pull::Up);
    let button_x = Input::new(peripherals.PIN_14, Pull::Up);
    let button_y = Input::new(peripherals.PIN_15, Pull::Up);

    // (START OF EXERCISE 4)
    // TODO 10: Declare ADC and potentiometer on ADC0
    let mut adc = Adc::new(peripherals.ADC, Irqs, AdcConfig::default());
    let mut potentiometer = AdcChannel::new_pin(peripherals.PIN_26, Pull::None);

    // TODO 6: Declare 3 PWMs, one for each RGB LED pin color (refer to Lab 04)
    let mut config_red: PwmConfig = Default::default();
    config_red.top = TOP;
    config_red.compare_a = config_red.top;

    
    let mut config_green: PwmConfig = Default::default();
    config_green.top = TOP;
    config_green.compare_a = 0;

    
    let mut config_blue: PwmConfig = Default::default();
    config_blue.top = TOP;
    config_blue.compare_a = 0;

    
    let mut pwm_red = Pwm::new_output_b(peripherals.PWM_CH0, peripherals.PIN_1, config_red.clone());
    
    let mut pwm_green = Pwm::new_output_a(peripherals.PWM_CH1, peripherals.PIN_2, config_green.clone());
    
    let mut pwm_blue = Pwm::new_output_b(peripherals.PWM_CH2, peripherals.PIN_5, config_blue.clone());


    // TODO 5: Spawn all the button tasks.
    spawner.spawn(button_a_pressed(button_a)).unwrap();
    spawner.spawn(button_b_pressed(button_b)).unwrap();
    spawner.spawn(button_x_pressed(button_x)).unwrap();
    spawner.spawn(button_y_pressed(button_y)).unwrap();
    //spawner.spawn(data(potentiometer,adc)).unwrap();

    let mut led_color: Option<LedColor> = None;
    let mut led_intensity: u16 = TOP;

    loop {
        let mut disp_color = "None";
        let mut disp_intensity = 0;
        //todo!(); // remove this or else it will panic
                 // TODO 7: Receive the command from the channel

        let _command = CHANNEL.receive().await;

        // TODO 8: Check what command was received.
        // Depending on the command, change the PWM config of the correct color pin (in this case you will set it at max intensity).
        // The rest of the colors will be set to 0.
        // Hint: To get the value out of the `Option`, you can do it this way:
        // ```rust
        // if let Some(ref color) = color_option { /* use color */ }
        // ```

        match _command{
            LedCommand::ChangeIntensity(ref value)=>{
                info!("Result: {}", value);
                disp_intensity = *value;
            }
            LedCommand::ChangeColor(Some(ref color)) => {
                match color{
                    LedColor::Red => {
                        config_blue.compare_a = 0;
                        config_green.compare_a = 0;
                        config_red.compare_a = TOP;
                        disp_color = "Red";
                    }
                    LedColor::Green => {
                        config_blue.compare_a = 0;
                        config_green.compare_a = TOP;
                        config_red.compare_a = 0;
                        disp_color = "Green";
                    }
                    LedColor::Blue => {
                        config_blue.compare_a = TOP;
                        config_green.compare_a = 0;
                        config_red.compare_a = 0;
                        disp_color = "Blue";
                    }
                }
            }
            LedCommand::ChangeColor(None) => {
                    config_blue.compare_a = 0;
                    config_green.compare_a = 0;
                    config_red.compare_a = 0;
                    disp_color = "None";
            }
        
        }


        // TODO 14:
        // (Method 1) Check for the new type of command.
        //            - If it's ChangeIntensity, modify the intensity of the active color.
        //            - If it's ChangeColor, change the active color (using the code you have already written for setting the color)
        // (Method 2) Check which channel receives a value first by using `select`.
        //            - If we get a value over the COLOR_CHANNEL first, we set the color.
        //            - If we get a value over the INTENSITY_CHANNEL first, we set the intensity.
        // Hint: We need to know the maximum value of the potentiometer in order to calculate the intensity based on what
        //       potentiometer reading we get. To find out the maximum value of the potentiometer, you could print it to the
        //       serial (by manually turning the knob to the maximum value and seeing what value it prints).
        //       Then, we would calculate the intensity as MAX_INTENSITY / MAX_POTENTIOMETER_VAL * CURRENT_POTENTIOMETER_VALUE. (Rule of 3)
        // (END OF EXERCISE 4)

        // TODO 9: Set the configs of all PWM pins.
        // (END OF EXERCISE 3)

        pwm_red.set_config(&config_red);
        pwm_green.set_config(&config_green);
        pwm_blue.set_config(&config_blue);

        // (START OF EXERCISE 5)
        // TODO 15: Print the intensity and the color of the RGB LED to the screen of the Pico Explorer.

        let mut text1 = String::<64>::new();
        let mut text2 = String::<64>::new();


        write!(text1, "Color: {}", disp_color).unwrap();
        write!(text2, "Intensity: {}", disp_intensity).unwrap();

        
        Text::new(&text1, Point::new(40, 110), style).draw(&mut display).unwrap();

        Text::new(&text2, Point::new(40, 110), style).draw(&mut display).unwrap();

        Timer::after_millis(1000).await;

        // (END OF EXERCISE 5)
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
