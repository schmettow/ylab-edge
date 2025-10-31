#![no_std]
#![no_main]

/// CONFIGURATION
///
/// Adc Tcm
//static SPEED: u32 = 100_000;
const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;
//const N_PROBES: u8 = 6;
use {defmt_rtt as _, panic_probe as _};

use defmt::*;
use embassy_executor::Executor;
use embassy_rp::gpio::Output;
use mcu::adc::Async;
#[allow(unused_imports)]
use mcu::gpio::Pin;
use mcu::multicore::{spawn_core1, Stack};

/// The following code initializes the second stack, plus
/// two heaps
static mut CORE1_STACK: Stack<4096> = Stack::new();
//use log::LevelFilter;

static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

use ylab::ybus::SharedI2cDevice;
use ylab::ysns::adc as yadc;
use ylab::ysns::moi;
use ylab::ytfk::bsu as ybsu;
use ylab_lib::yuii::btn as ybtn;
use ylab_lib::yuio::led as yled;
use ylab::*;

use ylab_lib::{Mutex, StaticCell};

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
use mcu::i2c::{self, Config};
use mcu::peripherals::{I2C0, I2C1};
use ylab::mcu;
//use ylab::*;
use ylab_lib::ysns::yxz_lsm6 as lsm6;
bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

#[cortex_m_rt::entry]
fn init() -> ! {
    let p = mcu::init(Default::default());
    // Init I2C shared busses
    let config = Config::default();

    // I2C Bus 0
    static I2C_BUS_0: StaticCell<SharedI2cBus<I2C0>> = StaticCell::new();
    let i2c0 = i2c::I2c::new_async(p.I2C0, p.PIN_1, p.PIN_0, Irqs, config);
    let i2c_bus_0 = I2C_BUS_0.init(Mutex::new(i2c0));

    // IC2 Bus 1
    static I2C_BUS_1: StaticCell<SharedI2cBus<I2C1>> = StaticCell::new();
    let i2c1 = i2c::I2c::new_async(p.I2C1, p.PIN_3, p.PIN_2, Irqs, config);
    let i2c_bus_1 = I2C_BUS_1.init(Mutex::new(i2c1));
    let i2c11 = SharedI2cDevice::new(i2c_bus_1);

    // CORE 1
    #[allow(static_mut_refs)]
    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner| {
        	spawner.spawn(lsm6_multi_task(i2c11)).unwrap();
        	spawner // BMI160
             .spawn(ylab::ysns::yxz_bmi160::task_0(i2c_bus_0, 101 as u64, 2))
             .unwrap();

        })
    });

    // CORE 0
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
    	// MOI task
        spawner
            .spawn(moi::task(
                p.PIN_21.into(),
                p.PIN_22.into(),
                p.PIN_8.into(),
                p.PIN_9.into(),
                0,
            ))
            .unwrap();
        // ADC task
        let adc0: adc::Adc<'_, Async> = adc::Adc::new(p.ADC, Irqs, adc::Config::default());
        spawner
            .spawn(yadc::task(adc0, p.PIN_26, p.PIN_27, p.PIN_28, 101, 1))
            .unwrap();

        // task for controlling the led
        use mcu::gpio::{Output, Level};
        let led = Output::new(p.PIN_25, Level::Low);
        unwrap!(spawner.spawn(led_task(led)));
        // task for listening to button presses.
        unwrap!(spawner.spawn(ybtn_20(p.PIN_20.into())));
        // task listening for data packeges to send up the line (reverse USB ;)
        unwrap!(spawner.spawn(ybsu::logger_task(p.USB, LOG_LEVEL)));
        unwrap!(spawner.spawn(ybsu::task()));
        // task to control sensors, storage and ui
        unwrap!(spawner.spawn(control_task()));
    });
}

// LED task
#[embassy_executor::task]
async fn led_task(led: Output<'static>){
	ylab_lib::yuio::led::task(led).await
}

// LSM6 task
#[embassy_executor::task]
async fn lsm6_multi_task(i2c: SharedI2c1) {
	lsm6::inner_multi_task(i2c, 6, 100, 2, false, ytfk::bsu::SINK.sender()).await;
}

// Button task
//use embassy_rp::peripherals::PIN_20;
use crate::mcu::gpio::Input;
use crate::mcu::gpio::Pull;
use crate::mcu::peripherals::PIN_20;

#[embassy_executor::task]
async fn ybtn_20(pin: Peri<'static, PIN_20>) {
    let pin = Input::new(pin, Pull::Up);
    yuii::btn::inner_task(pin).await;
}

#[embassy_executor::task]
async fn control_task() {
    let mut state = AppState::Ready; // <<--------
    moi::RECORD.store(true, ORD);
    yadc::RECORD.store(true, ORD);
    lsm6::RECORD.store(true, ORD);

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
                    lsm6::RECORD.store(false, ORD);
                }
                AppState::Ready => {
                    // Pause all sensors and blink
                    yled::LED.signal(yled::State::Blink);
                    yadc::RECORD.store(false, ORD);
                    moi::RECORD.store(false, ORD);
                    lsm6::RECORD.store(false, ORD);
                }
                AppState::Record => {
                    // Transmit sensor data and light up
                    yled::LED.signal(yled::State::Steady);
                    yadc::RECORD.store(true, ORD);
                    moi::RECORD.store(true, ORD);
                    lsm6::RECORD.store(true, ORD);
                }
            }
            state = next_state;
        }
    }
}
