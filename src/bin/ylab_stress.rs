#![no_std]
#![no_main]

/// CONFIGURATION
///
/// Adc Tcm
use {defmt_rtt as _, panic_probe as _};

/// The following code initializes the second stack, plus
/// two heaps
static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

use ylab_lib as yll;
use yll::{Mutex, StaticCell};
use yll::ysns::moi;
use yll::ysns::yco2;
use yll::yuii::btn as ybtn;
use yll::yuio::yled;

use ydsp::TEXT as DISP;
use ylab::*;
use ysns::adc as yadc;
use ytfk::bsu as ybsu;
use yuio::disp as ydsp;
//use yuio::led as yled;

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
bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

use defmt::*;
use embassy_executor::Executor;
#[allow(unused_imports)]
use mcu::adc::{Async, Blocking};
use mcu::multicore::{spawn_core1, Stack};

#[cortex_m_rt::entry]
fn init() -> ! {
    let p = mcu::init(Default::default());
    // Init I2C shared busses
    let config = Config::default();
    static I2C_BUS_0: StaticCell<SharedI2cBus<I2C0>> = StaticCell::new();
    let i2c0 = i2c::I2c::new_async(p.I2C0, p.PIN_1, p.PIN_0, Irqs, config);
    let i2c_bus_0 = I2C_BUS_0.init(Mutex::new(i2c0));
    #[allow(unused_variables)]
    let i2c01 = SharedI2cDevice::new(i2c_bus_0);

    static I2C_BUS_1: StaticCell<SharedI2cBus<I2C1>> = StaticCell::new();
    let i2c1 = i2c::I2c::new_async(p.I2C1, p.PIN_3, p.PIN_2, Irqs, config);
    #[allow(unused_variables)]
    let i2c_bus_1 = I2C_BUS_1.init(Mutex::new(i2c1));
    #[allow(static_mut_refs)]
    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner| {
            spawner // CO2 (scd4)
                .spawn(co2_task(i2c01))
                .unwrap();
            unwrap!(spawner.spawn(ylab::ysns::ads1115::task_1(i2c_bus_1, 4, 3)));
            unwrap!(spawner.spawn(yuio::disp::task_1(i2c_bus_1)));
        })
    });

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        // task for controlling the led
        unwrap!(spawner.spawn(yled::task(p.PIN_25.into())));
        // listening to button presses.
        unwrap!(spawner.spawn(ybtn_20(p.PIN_20.into())));
        //  listening for data packeges to send up the line (reverse USB ;)
        unwrap!(spawner.spawn(ybsu::logger_task(p.USB, log::LevelFilter::Info)));
        unwrap!(spawner.spawn(ybsu::task()));
        // control sensors, storage and ui
        unwrap!(spawner.spawn(control_task()));
        // collect moi events
        spawner
            .spawn(ylab::ysns::moi::task_2(p.PIN_21, p.PIN_22, 0))
            .unwrap();
        // ADC task
        let adc0: adc::Adc<'_, Async> = adc::Adc::new(p.ADC, Irqs, adc::Config::default());
        spawner
            .spawn(yadc::task(
                adc0,
                p.PIN_26.into(),
                p.PIN_27.into(),
                p.PIN_28.into(),
                201,
                1,
            ))
            .unwrap();
    });
}

use mcu::gpio::Input;
use mcu::gpio::Pull;
use mcu::peripherals::PIN_20;

#[embassy_executor::task]
async fn ybtn_20(pin: Peri<'static, PIN_20>) {
    let pin = Input::new(pin, Pull::Up);
    yuii::btn::inner_task(pin).await;
}

// LSM6 task
#[embassy_executor::task]
async fn co2_task(i2c: SharedI2c0) {
	yco2::task(i2c, 3, ybsu::SINK.sender()).await;
}

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
                    //yco2::RECORD.store(false, ORD);
                    DISP.signal([Some("New".try_into().unwrap()), None, None, None]);
                }
                AppState::Ready => {
                    // Pause all sensors and blink
                    yled::LED.signal(yled::State::Blink);
                    moi::RECORD.store(false, ORD);
                    yadc::RECORD.store(false, ORD);
                    //yco2::RECORD.store(false, ORD);
                    DISP.signal([Some("Ready".try_into().unwrap()), None, None, None]);
                }
                AppState::Record => {
                    // Transmit sensor data and light up
                    moi::RECORD.store(true, ORD);
                    yled::LED.signal(yled::State::Steady);
                    yadc::RECORD.store(true, ORD);
                    //yco2::RECORD.store(true, ORD);
                    DISP.signal([Some("Record".try_into().unwrap()), None, None, None]);
                }
            }
            state = next_state;
        }
    }
}
