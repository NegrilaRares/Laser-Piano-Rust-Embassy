#![no_std]
#![no_main]
#![allow(unused_imports, dead_code, unused_variables, unused_mut)] // i removed allowing dead_code as i did not know what that entailed


use core::cell::RefCell;
use core::panic::PanicInfo;
use embassy_executor::Spawner;

//SD
use embedded_sdmmc::{SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use embedded_sdmmc::Volume;
use embedded_sdmmc::VolumeType::Fat;
use embedded_sdmmc::filesystem::Mode;
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embedded_hal::delay::DelayNs;


#[derive(Default)]
pub struct DummyTimesource();

impl TimeSource for DummyTimesource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

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

//Peripherals
use embassy_rp::peripherals::{
    ADC, DMA_CH0, DMA_CH1, PIN_0, PIN_1, PIN_10, PIN_11, PIN_12, PIN_13, PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_2, PIN_20, PIN_21, PIN_22, PIN_23, PIN_24, PIN_25, PIN_26, PIN_27, PIN_28, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, SPI0, USB
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
use log::{info, error};

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
use embassy_futures::select::Either::{First as First1, Second as Second1};
use embassy_futures::select::Either3::{First as First3, Second as Second3, Third as Third3};
use embassy_futures::select::select3;

use embassy_futures::select::select_array;
use embassy_futures::select::SelectArray;

use core::future::Future;
use core::pin::Pin as corePin;
use core::task::{Context, Poll};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Either7<A, B, C, D, E, F, G> {
    First(A),
    Second(B),
    Third(C),
    Forth(D),
    Fifth(E),
    Sixth(F),
    Seventh(G)
}

pub fn select7<A, B, C, D, E, F, G>(a: A, b: B, c: C, d: D, e: E, f: F, g: G) -> Select7<A, B, C, D, E, F, G>
where
    A: Future,
    B: Future,
    C: Future,
    D: Future,
    E: Future,
    F: Future,
    G: Future,
{
    Select7 { a, b, c, d, e, f, g }
}


#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Select7<A, B, C, D, E, F, G> {
    a: A,
    b: B,
    c: C,
    d: D, 
    e: E, 
    f: F, 
    g: G,
}

impl<A, B, C, D, E, F, G> Future for Select7<A, B, C, D, E, F, G>
where
    A: Future,
    B: Future,
    C: Future,
    D: Future,
    E: Future,
    F: Future,
    G: Future,
{
    type Output = Either7<A::Output, B::Output, C::Output, D::Output, E::Output, F::Output, G::Output>;

    fn poll(self: corePin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let a = unsafe { corePin::new_unchecked(&mut this.a) };
        let b = unsafe { corePin::new_unchecked(&mut this.b) };
        let c = unsafe { corePin::new_unchecked(&mut this.c) };
        let d = unsafe { corePin::new_unchecked(&mut this.d) };
        let e = unsafe { corePin::new_unchecked(&mut this.e) };
        let f = unsafe { corePin::new_unchecked(&mut this.f) };
        let g = unsafe { corePin::new_unchecked(&mut this.g) };
        if let Poll::Ready(x) = a.poll(cx) {
            return Poll::Ready(Either7::First(x));
        }
        if let Poll::Ready(x) = b.poll(cx) {
            return Poll::Ready(Either7::Second(x));
        }
        if let Poll::Ready(x) = c.poll(cx) {
            return Poll::Ready(Either7::Third(x));
        }
        if let Poll::Ready(x) = d.poll(cx) {
            return Poll::Ready(Either7::Forth(x));
        }
        if let Poll::Ready(x) = e.poll(cx) {
            return Poll::Ready(Either7::Fifth(x));
        }
        if let Poll::Ready(x) = f.poll(cx) {
            return Poll::Ready(Either7::Sixth(x));
        }
        if let Poll::Ready(x) = g.poll(cx) {
            return Poll::Ready(Either7::Seventh(x));
        }
        Poll::Pending
    }
}

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

const DISPLAY_FREQ: u32 = 64_000_000;bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    ADC_IRQ_FIFO => InterruptHandlerADC;
});

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

pub enum LsrLvl{
    ONE,
    TWO,
    THREE,
    FOUR,
    FIVE,
    SIX,
    SEVEN,
}

static VOLUME_CHANNEL: Channel<ThreadModeRawMutex, VolLvl, 64> = Channel::new();
static OCTAVE_CHANNEL: Channel<ThreadModeRawMutex, OctLvl, 64> = Channel::new();
static LASER_CHANNEL: Channel<ThreadModeRawMutex, LsrLvl, 64> = Channel::new();

#[embassy_executor::task]
async fn oct_input(
    channel_sender_octave: Sender<'static, ThreadModeRawMutex, OctLvl, 64>,
    mut sw_1:Input<'static, PIN_27>,
    mut sw_2:Input<'static, PIN_28>,
    mut sw_3:Input<'static, PIN_22>,
){
    loop{
        let select = select3(sw_1.wait_for_rising_edge(), sw_2.wait_for_rising_edge(), sw_3.wait_for_rising_edge()).await;

        match select {
            First3(switch_1) => {
                channel_sender_octave.send(OctLvl::ONE).await;
            }
            Second3(switch_2) => {
                channel_sender_octave.send(OctLvl::TWO).await;
            }
            Third3(switch_3) => {
                channel_sender_octave.send(OctLvl::THREE).await;
            }
        }
    }
}

#[embassy_executor::task]
async fn vol_input(
    mut potentiometer: ChannelADC<'static>, 
    mut adc: Adc<'static, Async>, 
    channel_sender_volume: Sender<'static, ThreadModeRawMutex, VolLvl, 64>,
    mut clk:Output<'static, PIN_20>,
    mut data:Output<'static, PIN_18>,
    mut latch:Output<'static, PIN_19>,
    mut reset:Output<'static, PIN_21>,
) {
    loop{
        //info!("Waiting!");
        let value = adc.read(&mut potentiometer).await.unwrap(); // (19)20(21) - 4095  
        //info!("Result: {:?}", value);
        match value {
            0..=426 => {
                info!("Value is in range 1: <0, 426>");
                channel_sender_volume.send(VolLvl::ONE).await;
                let mut max = 1;
                let mut i = 0;
                reset.set_low();
                Timer::after_nanos(100).await;
                reset.set_high();
                Timer::after_nanos(100).await;
                while i < max
                {
                    data.set_high();
                    clk.set_high();
                    Timer::after_nanos(100).await;
                    clk.set_low();
                    data.set_low();
                    Timer::after_nanos(100).await;
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(100).await;
                latch.set_low();
                Timer::after_nanos(100).await;
            } ,
            427..=833 => {
                info!("Value is in range 2: <427, 833>");
                channel_sender_volume.send(VolLvl::TWO).await;
                let mut max = 2;
                let mut i = 0;
                reset.set_low();
                Timer::after_nanos(100).await;
                reset.set_high();
                Timer::after_nanos(100).await;
                while i < max
                {
                    data.set_high();
                    clk.set_high();
                    Timer::after_nanos(100).await;
                    clk.set_low();
                    data.set_low();
                    Timer::after_nanos(100).await;
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(100).await;
                latch.set_low();
                Timer::after_nanos(100).await;
            },
            834..=1240 => {
                info!("Value is in range 3: <834, 1240>");
                channel_sender_volume.send(VolLvl::THREE).await;
                let mut max = 3;
                let mut i = 0;
                reset.set_low();
                Timer::after_nanos(100).await;
                reset.set_high();
                Timer::after_nanos(100).await;
                while i < max
                {
                    data.set_high();
                    clk.set_high();
                    Timer::after_nanos(100).await;
                    clk.set_low();
                    data.set_low();
                    Timer::after_nanos(100).await;
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(100).await;
                latch.set_low();
                Timer::after_nanos(100).await;
            },
            1241..=1647 => {
                info!("Value is in range 4: <1241, 1647>");
                channel_sender_volume.send(VolLvl::FOUR).await;
                let mut max = 4;
                let mut i = 0;
                reset.set_low();
                Timer::after_nanos(100).await;
                reset.set_high();
                Timer::after_nanos(100).await;
                while i < max
                {
                    data.set_high();
                    clk.set_high();
                    Timer::after_nanos(100).await;
                    clk.set_low();
                    data.set_low();
                    Timer::after_nanos(100).await;
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(100).await;
                latch.set_low();
                Timer::after_nanos(100).await;
            },
            1648..=2054 => {
                info!("Value is in range 5: <1648, 2054>");
                channel_sender_volume.send(VolLvl::FIVE).await;
                let mut max = 5;
                let mut i = 0;
                reset.set_low();
                Timer::after_nanos(100).await;
                reset.set_high();
                Timer::after_nanos(100).await;
                while i < max
                {
                    data.set_high();
                    clk.set_high();
                    Timer::after_nanos(100).await;
                    clk.set_low();
                    data.set_low();
                    Timer::after_nanos(100).await;
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(100).await;
                latch.set_low();
                Timer::after_nanos(100).await;
            },
            2055..=2461 => {
                info!("Value is in range 6: <2055, 2461>");
                channel_sender_volume.send(VolLvl::SIX).await;
                let mut max = 6;
                let mut i = 0;
                reset.set_low();
                Timer::after_nanos(100).await;
                reset.set_high();
                Timer::after_nanos(100).await;
                while i < max
                {
                    data.set_high();
                    clk.set_high();
                    Timer::after_nanos(100).await;
                    clk.set_low();
                    data.set_low();
                    Timer::after_nanos(100).await;
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(100).await;
                latch.set_low();
                Timer::after_nanos(100).await;
            },
            2462..=2868 => {
                info!("Value is in range 7: <2462, 2868>");
                channel_sender_volume.send(VolLvl::SEVEN).await;
                let mut max = 7;
                let mut i = 0;
                reset.set_low();
                Timer::after_nanos(100).await;
                reset.set_high();
                Timer::after_nanos(100).await;
                while i < max
                {
                    data.set_high();
                    clk.set_high();
                    Timer::after_nanos(100).await;
                    clk.set_low();
                    data.set_low();
                    Timer::after_nanos(100).await;
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(100).await;
                latch.set_low();
                Timer::after_nanos(100).await;
            },
            2869..=3275 => {
                info!("Value is in range 8: <2869, 3275>");
                channel_sender_volume.send(VolLvl::EIGHT).await;
                let mut max = 8;
                let mut i = 0;
                reset.set_low();
                Timer::after_nanos(100).await;
                reset.set_high();
                Timer::after_nanos(100).await;
                while i < max
                {
                    data.set_high();
                    clk.set_high();
                    Timer::after_nanos(100).await;
                    clk.set_low();
                    data.set_low();
                    Timer::after_nanos(100).await;
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(100).await;
                latch.set_low();
                Timer::after_nanos(100).await;
            },
            3276..=3682 => {
                info!("Value is in range 9: <3276, 3682>");
                channel_sender_volume.send(VolLvl::NINE).await;
                let mut max = 9;
                let mut i = 0;
                reset.set_low();
                Timer::after_nanos(100).await;
                reset.set_high();
                Timer::after_nanos(100).await;
                while i < max
                {
                    data.set_high();
                    clk.set_high();
                    Timer::after_nanos(100).await;
                    clk.set_low();
                    data.set_low();
                    Timer::after_nanos(100).await;
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(100).await;
                latch.set_low();
                Timer::after_nanos(100).await;
            },
            3683..=4095 => {
                info!("Value is in range 10: <3683, 4095>");
                channel_sender_volume.send(VolLvl::TEN).await;
                let mut max = 10;
                let mut i = 0;
                reset.set_low();
                Timer::after_nanos(100).await;
                reset.set_high();
                Timer::after_nanos(100).await;
                while i < max
                {
                    data.set_high();
                    clk.set_high();
                    Timer::after_nanos(100).await;
                    clk.set_low();
                    data.set_low();
                    Timer::after_nanos(100).await;
                    i = i + 1;
                }
                latch.set_high();
                Timer::after_nanos(100).await;
                latch.set_low();
                Timer::after_nanos(100).await;
            },
            _ => info!("Value is not in any defined range"),
        }
        Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn lsr_input(
    channel_sender_lsr: Sender<'static, ThreadModeRawMutex, LsrLvl, 64>,
    mut lsr_in_1:Input<'static, PIN_9>,
    mut lsr_in_2:Input<'static, PIN_10>,
    mut lsr_in_3:Input<'static, PIN_11>,
    mut lsr_in_4:Input<'static, PIN_12>,
    mut lsr_in_5:Input<'static, PIN_13>,
    mut lsr_in_6:Input<'static, PIN_14>,
    mut lsr_in_7:Input<'static, PIN_15>,
){

    let mut previous_input = 0;
    loop{

        let select = select7(
            lsr_in_1.wait_for_rising_edge(), 
            lsr_in_2.wait_for_rising_edge(), 
            lsr_in_3.wait_for_rising_edge(),
            lsr_in_4.wait_for_rising_edge(),
            lsr_in_5.wait_for_rising_edge(),
            lsr_in_6.wait_for_rising_edge(),
            lsr_in_7.wait_for_rising_edge(),
        ).await;

        match select {
            Either7::First(laser_1) => {
                if previous_input == 1
                {
                    Timer::after_millis(160).await;
                    channel_sender_lsr.send(LsrLvl::ONE).await;
                    previous_input = 1;
                }
                else
                {
                    channel_sender_lsr.send(LsrLvl::ONE).await;
                    previous_input = 1;
                }
                 
            }
            Either7::Second(laser_2) => {
                if previous_input == 2
                {
                    Timer::after_millis(160).await;
                    channel_sender_lsr.send(LsrLvl::TWO).await;
                    previous_input = 2;
                }
                else 
                {
                    channel_sender_lsr.send(LsrLvl::TWO).await;    
                    previous_input = 2;
                }
                   
            }
            Either7::Third(laser_3) => {
                if previous_input == 3
                {
                    Timer::after_millis(160).await;
                    channel_sender_lsr.send(LsrLvl::THREE).await;
                    previous_input = 3;
                }
                else 
                {
                    channel_sender_lsr.send(LsrLvl::THREE).await;  
                    previous_input = 3;
                }
                  
            }
            Either7::Forth(laser_4) => {
                if previous_input == 4
                {
                    Timer::after_millis(160).await;
                    channel_sender_lsr.send(LsrLvl::FOUR).await;
                    previous_input = 4;
                }
                else 
                {
                    channel_sender_lsr.send(LsrLvl::FOUR).await; 
                    previous_input = 4;
                }
                
            }
            Either7::Fifth(laser_5) => {
                if previous_input == 5
                {
                    Timer::after_millis(160).await;
                    channel_sender_lsr.send(LsrLvl::FIVE).await;
                    previous_input = 5;
                }
                else 
                {
                    channel_sender_lsr.send(LsrLvl::FIVE).await;
                    previous_input = 5;
                }
                
            }
            Either7::Sixth(laser_6) => {
                if previous_input == 6
                {
                    Timer::after_millis(160).await;
                    channel_sender_lsr.send(LsrLvl::SIX).await;
                    previous_input = 6;
                }
                else 
                {
                    channel_sender_lsr.send(LsrLvl::SIX).await;
                    previous_input = 6;
                }
              
            }
            Either7::Seventh(laser_7) => {
                if previous_input == 7
                {
                    Timer::after_millis(160).await;
                    channel_sender_lsr.send(LsrLvl::SEVEN).await;
                    previous_input = 7;
                }
                else 
                {
                    channel_sender_lsr.send(LsrLvl::SEVEN).await;
                    previous_input = 7;
                }  
            }
        }
    }
}


#[embassy_executor::task]
async fn sd_card(
    mut miso: PIN_4,
    mut mosi: PIN_3,
    mut clk: PIN_2,
    mut cs:Output<'static, PIN_5>,
    mut channel_spi: SPI0,
    mut dma_spi_1: DMA_CH0,
    mut dma_spi_2: DMA_CH1,
){
    

    let mut microsd_config = spi::Config::default();
    microsd_config.frequency = 2000000;

    let mut spi = Spi::new(
        channel_spi,
        clk,
        mosi,
        miso,
        dma_spi_1,
        dma_spi_2,
        microsd_config.clone(),
    );

    

    let mut delay_sd = Delay;
    delay_sd.delay_ms(200);

    let sdcard = SdCard::new(spi, cs, delay_sd);
    let mut volume_mgr = VolumeManager::new(sdcard, DummyTimesource::default());

    Timer::after_secs(3).await;
    info!("Init SD card controller and retrieve card size...");
    match volume_mgr.device().num_bytes() {
        Ok(size) => info!("card size is {} bytes", size),
        Err(e) => {
            error!("Error retrieving card size!");
        }
    }

    info!("Getting Volume 0...");
    let mut volume_sd=volume_mgr.get_volume(VolumeIdx(0)).unwrap();
    info!("Volume : {:?}", &volume_sd);
    
    let volume_result = volume_mgr.open_root_dir(&volume_sd);

    let dir =  volume_mgr.open_root_dir(&volume_sd).unwrap();

    info!("Root directory opened!");

    volume_mgr.iterate_dir(&volume_sd, &dir, |ent| {
        info!(
            "/{}.{}",
            core::str::from_utf8(ent.name.base_name()).unwrap(),
            core::str::from_utf8(ent.name.extension()).unwrap()
        );
    }).unwrap();

    let mut successful_read = false;

    if let Ok(mut file) = volume_mgr.open_file_in_dir(&mut volume_sd, &dir, "a312.bin", Mode::ReadOnly) {
        let mut buf = [0u8; 32];
        let read_count = volume_mgr.read(&volume_sd, &mut file, &mut buf).unwrap();
        volume_mgr.close_file(&volume_sd, file).unwrap();

        if read_count >= 2 {
            info!("READ {} bytes: {:?}", read_count, buf);

            // If we read what we wrote before the last reset,
            // we set a flag so that the success blinking at the end
            // changes it's pattern.
        }
    }

    volume_mgr.free();

    if successful_read {
        info!("Successfully read previously written file 'O.TST'");
    } else {
        info!("Could not read file, which is ok for the first run.");
        info!("Reboot the pico!");
    }

}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    let mut adc = Adc::new(p.ADC, Irqs, ConfigADC::default());
    let mut potentiometer = ChannelADC::new_pin(p.PIN_26, Pull::None);

    let mut clock = Output::new(p.PIN_20, Level::Low);
    let mut in_data = Output::new(p.PIN_18, Level::Low);
    let mut latch = Output::new(p.PIN_19, Level::Low);
    let mut clear = Output::new(p.PIN_21, Level::High);

    let mut oct_1_led = Output::new(p.PIN_0, Level::Low);
    let mut oct_2_led = Output::new(p.PIN_1, Level::Low);
    let mut oct_3_led = Output::new(p.PIN_8, Level::Low);

    let mut sw_oct_1 = Input::new(p.PIN_27, Pull::Down);
    let mut sw_oct_2 = Input::new(p.PIN_28, Pull::Down);
    let mut sw_oct_3 = Input::new(p.PIN_22, Pull::Down);

    let mut input_1 = Input::new(p.PIN_9, Pull::Up);
    let mut input_2 = Input::new(p.PIN_10, Pull::Up);
    let mut input_3 = Input::new(p.PIN_11, Pull::Up);
    let mut input_4 = Input::new(p.PIN_12, Pull::Up);
    let mut input_5 = Input::new(p.PIN_13, Pull::Up);
    let mut input_6 = Input::new(p.PIN_14, Pull::Up);
    let mut input_7 = Input::new(p.PIN_15, Pull::Up);

    let miso = p.PIN_4;
    let mosi = p.PIN_3;
    let clk = p.PIN_2;

    let mut microsd_cs = Output::new(p.PIN_5, Level::High);
    let mut spi_channel = p.SPI0;
    let mut dma_spi_1 = p.DMA_CH0;
    let mut dma_spi_2 = p.DMA_CH1;

    // let _ =spawner.spawn(
    //     vol_input(
    //         potentiometer, 
    //         adc, 
    //         VOLUME_CHANNEL.sender(), 
    //         clock, 
    //         in_data, 
    //         latch,
    //         clear,
    //     )
    // ).unwrap();

    // let _ =spawner.spawn(
    //     oct_input(
    //         OCTAVE_CHANNEL.sender(),
    //         sw_oct_1, 
    //         sw_oct_2, 
    //         sw_oct_3
    //     )
    // );

    // let _ =spawner.spawn(
    //     lsr_input(
    //         LASER_CHANNEL.sender(),
    //         input_1, 
    //         input_2,
    //         input_3,
    //         input_4,
    //         input_5,
    //         input_6,
    //         input_7,
    //     )
    // );
    
    let _ =spawner.spawn(
        sd_card(
            miso,
            mosi,
            clk,
            microsd_cs,
            spi_channel,
            dma_spi_1,
            dma_spi_2,
        )
    );
    
    
    let mut volume: VolLvl = VolLvl::ONE;
    let mut octave: OctLvl = OctLvl::TWO;
    oct_1_led.set_low();
    oct_2_led.set_high();
    oct_3_led.set_low();
    let mut lsr: LsrLvl = LsrLvl::ONE;
    let mut var = 0;

    loop {
        let select = select3(VOLUME_CHANNEL.receive(), OCTAVE_CHANNEL.receive(),LASER_CHANNEL.receive()).await;

        match select {
            First3(volume_value) =>{
                match volume_value{
                    VolLvl::ONE => {

                    }
                    VolLvl::TWO => {

                    }
                    VolLvl::THREE => {

                    }
                    VolLvl::FOUR => {

                    }
                    VolLvl::FIVE => {

                    }
                    VolLvl::SIX => {

                    }
                    VolLvl::SEVEN => {

                    }
                    VolLvl::EIGHT => {

                    }
                    VolLvl::NINE => {

                    }
                    VolLvl::TEN => {

                    }
                }                
            }
            Second3(octave_value) =>{
                match octave_value{
                    OctLvl::ONE => {
                        oct_1_led.set_high();
                        oct_2_led.set_low();
                        oct_3_led.set_low();
                    }
                    OctLvl::TWO => {
                        oct_1_led.set_low();
                        oct_2_led.set_high();
                        oct_3_led.set_low();
                    }
                    OctLvl::THREE => {
                        oct_1_led.set_low();
                        oct_2_led.set_low();
                        oct_3_led.set_high();
                    }
                }
            }
            Third3(lsr_value) =>{
                match lsr_value{
                    LsrLvl::ONE =>{
                        info!("{} ONE!",var);
                        var += 1;
                    }
                    LsrLvl::TWO =>{
                        info!("{} TWO!",var);
                        var += 1;
                    }
                    LsrLvl::THREE =>{
                        info!("{} THREE!",var);
                        var += 1;
                    }
                    LsrLvl::FOUR =>{
                        info!("{} FOUR!",var);
                        var += 1;
                    }
                    LsrLvl::FIVE =>{
                        info!("{} FIVE!",var);
                        var += 1;
                    }
                    LsrLvl::SIX =>{
                        info!("{} SIX!",var);
                        var += 1;
                    }
                    LsrLvl::SEVEN =>{
                        info!("{} SEVEN!",var);
                        var += 1;
                    }
                }
            }
        }



    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}