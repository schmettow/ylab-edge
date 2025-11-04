#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};
use defmt::*;

use embassy_executor::Executor;
#[allow(unused_imports)]
use mcu::adc::{Async, Blocking};
use mcu::multicore::{spawn_core1, Stack};

/// The following code initializes the second stack, plus
/// two heaps
static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();


use ylab_lib as yll;
use yll::ysns::moi;
use yll::yuii::btn as ybtn;
use yll::yuio::led as yled;

use ylab::*;
use task::{moi_task, btn20_task, led_task};
use ysns::adc as yadc;
use ytfk::bsu as ybsu;
use mcu::gpio::{Output, Level};


use yll::StaticCell;

#[derive(
    Debug, // used as fmt
    Clone,
    Copy, // because next_state
    PartialEq,
    Eq,
)] // testing equality
enum AppState {
    New,
    Ready,
    Record,
}

use mcu::adc;
use mcu::bind_interrupts;
bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

#[cortex_m_rt::entry]
fn init() -> ! {
    let p = mcu::init(Default::default());
    #[allow(static_mut_refs)]
    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner| {

            spawner.spawn(
         		moi_task(
	                p.PIN_21.into(),
	                p.PIN_22.into(),
	                p.PIN_8.into(),
	                p.PIN_9.into(),
					0
            ))
            .unwrap();

         	let adc0: adc::Adc<'_, Async> = adc::Adc::new(p.ADC, Irqs, adc::Config::default());
	        spawner
	            .spawn(yadc::task(adc0, p.PIN_26, p.PIN_27, p.PIN_28, 197, 1))
	            .unwrap();
        })
    });


    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        // task for controlling the led
        let led = Output::new(p.PIN_25, Level::Low);
        unwrap!(spawner.spawn(led_task(led)));
        // task for listening to button presses.
        unwrap!(spawner.spawn(btn20_task(p.PIN_20)));
        // task listening for data packeges to send up the line (reverse USB ;)
        unwrap!(spawner.spawn(ybsu::logger_task(p.USB, log::LevelFilter::Info)));
        unwrap!(spawner.spawn(ybsu::task()));
        // task to control sensors, storage and ui
        unwrap!(spawner.spawn(control_task()))
    });
}

/*`use mcu::gpio::Input;
use mcu::gpio::Pull;
use mcu::peripherals::{PIN_20, PIN_21, PIN_22, PIN_8, PIN_9};*/

/*#[embassy_executor::task]
async fn moi_task(
    pin_0: Peri<'static, PIN_21>,
    pin_1: Peri<'static, PIN_22>,
    pin_2: Peri<'static, PIN_8>,
    pin_3: Peri<'static, PIN_9>)
    {
    let moi_0 = Input::new(pin_0, Pull::Up);
    let moi_1 = Input::new(pin_1, Pull::Up);
    let moi_2 = Input::new(pin_2, Pull::Up);
    let moi_3 = Input::new(pin_3, Pull::Up);
	ylab_lib::ysns::moi::inner_task(moi_0, moi_1, moi_2, moi_3, 0, ylab::ytfk::bsu::SINK.sender()).await;
}

#[embassy_executor::task]
async fn ybtn_20(pin: Peri<'static, PIN_20>) {
    let pin = Input::new(pin, Pull::Up);
    yuii::btn::inner_task(pin).await;
}*/

#[embassy_executor::task]
async fn control_task() {
    let mut state = AppState::Record;
    moi::RECORD.store(true, ORD);
    yadc::RECORD.store(true, ORD);

    yled::LED.signal(yled::State::Steady);
    loop {
        let event = ybtn::BTN.wait().await;
        // Only when a new user event appears,
        // a state transition may occur.
        if let Some(next_state) = match (state, event) {
            (AppState::New, ybtn::Event::Short) => Some(AppState::Ready),
            (AppState::Ready, ybtn::Event::Short) => Some(AppState::Record),
            (AppState::Record, ybtn::Event::Short) => Some(AppState::Ready),
            (_, ybtn::Event::Long) => Some(AppState::New),
            (_, _) => None,
        } {
            // When a new event has been announced we do the transition.
            // This happens by sending the right messages to all our tasks.
            match next_state {
                AppState::New => {
                    // Reset all sensors and vibrate
                    yled::LED.signal(yled::State::Vibrate);
                    moi::RECORD.store(false, ORD);
                    yadc::RECORD.store(false, ORD);
                }
                AppState::Ready => {
                    // Pause all sensors and blink
                    yled::LED.signal(yled::State::Blink);
                    moi::RECORD.store(false, ORD);
                    yadc::RECORD.store(false, ORD);
                }
                AppState::Record => {
                    // Transmit sensor data and light up
                    yled::LED.signal(yled::State::Steady);
                    moi::RECORD.store(true, ORD);
                    yadc::RECORD.store(true, ORD);
                }
            }
            state = next_state;
        }
    }
}
