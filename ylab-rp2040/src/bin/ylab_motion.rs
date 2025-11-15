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

use ylab_lib as yll;
use yll::ysns::moi;
use yll::yuii::btn as ybtn;
use yll::yuio::led as yled;
use ylab::task::{moi_task, btn20_task, led_task};
use ylab::ybus::SharedI2cDevice;
use ylab::ysns::adc as yadc;
use ylab::ytfk::bsu as ybsu;
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
    let i2c01 = SharedI2cDevice::new(i2c_bus_0);

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
        	spawner.spawn(task::lsm6_multi_task_1(i2c11, 3, 3, 2)).unwrap();
         	spawner .spawn(task::ads_task_0(i2c01, 5, 3)).unwrap();
        })
    });

    // CORE 0
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
    	// MOI task
        spawner
            .spawn(moi_task(
                p.PIN_21.into(),
                p.PIN_22.into(),
                p.PIN_8.into(),
                p.PIN_9.into(),
                0
            ))
            .unwrap();
        // ADC task
        let adc0: adc::Adc<'_, Async> = adc::Adc::new(p.ADC, Irqs, adc::Config::default());
        spawner
            .spawn(yadc::task(adc0, p.PIN_26, p.PIN_27, p.PIN_28, 0, 1))
            .unwrap();

        // task for controlling the led
        use mcu::gpio::{Output, Level};
        let led = Output::new(p.PIN_25, Level::Low);
        unwrap!(spawner.spawn(led_task(led)));
        // task for listening to button presses.
        unwrap!(spawner.spawn(btn20_task(p.PIN_20.into())));
        // task listening for data packeges to send up the line (reverse USB ;)
        unwrap!(spawner.spawn(ybsu::logger_task(p.USB, LOG_LEVEL)));
        unwrap!(spawner.spawn(ybsu::task()));
        // task to control sensors, storage and ui
        unwrap!(spawner.spawn(control_task()));
    });
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
