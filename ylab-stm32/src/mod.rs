#![no_std]

use {defmt_rtt as _, panic_probe as _};
pub use embassy_stm32 as mcu;
pub use mcu::gpio::{AnyPin, Pull, Input, Output, Level};
pub use mcu::Peri;
pub use mcu::exti::ExtiInput;
pub use mcu::usart::{Config as UartConfig, InterruptHandler as UartInterruptHandler, Uart};
pub use mcu::{bind_interrupts, peripherals};
pub use ylab_lib as yll;
pub use yll::ydata as data;
pub use yll::Spawner;
pub use data::Ytf;
pub use yll::ybus::*;
pub use yll::yuii;
pub use yll::yuio;
//pub use yll::ysns as yllsns;

/*pub use mcu::peripherals::I2C1;
pub use mcu::peripherals::I2C2;*/
pub use mcu::i2c;
pub use i2c::I2c;
use mcu::mode::Async;
//pub use mcu::adc;
//use embassy_embedded_hal as ehal;
pub use ehal::shared_bus::asynch::i2c::I2cDevice;

type SharedBusMutexType = embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
pub type SharedBusMutex<I> = embassy_sync::mutex::Mutex<SharedBusMutexType, I>;
pub type SharedI2cBus =
    SharedBusMutex<MasterAsyncI2c>;

pub type MasterAsyncI2c = mcu::i2c::I2c<'static, mcu::mode::Async, mcu::i2c::Master>;
//pub type I2c1 = I2cDevice<'static, SharedBusMutexType, I2c<'static, I2C1, Async>>;
pub type SharedI2cDevice = I2cDevice<'static, SharedBusMutexType, I2c<'static, Async, mcu::i2c::Master>>;

//pub mod ysns; // analog sensors
pub mod ytfk;
pub mod task;
pub mod ysns;
//pub mod yuii; // YLab UI Input
//pub mod yuio; // YLab UI Output // YLab transfer formats & kodices
