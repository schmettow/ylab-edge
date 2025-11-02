#![no_std]

pub use embassy_rp as mcu;
pub use mcu::gpio::AnyPin;
pub use mcu::Peri;
pub use ylab_lib as yll;
pub use yll::yuii;
pub use yll::yuio;
pub use yll::*;
pub use yll::ybus::*;

pub mod ysns; // Ylab sensors
pub mod ytfk;
//pub mod yuii; // YLab UI Input
//pub mod yuio; // YLab UI Output // YLab transfer formats & kodices
pub mod task;

pub use mcu::peripherals::I2C0;
pub use mcu::peripherals::I2C1;
use mcu::i2c::{Async, I2c};
use crate::ehal::shared_bus::asynch::i2c::I2cDevice;

type SharedBusMutexType = embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
pub type SharedBusMutex<I> = embassy_sync::mutex::Mutex<SharedBusMutexType, I>;
pub type SharedI2cBus<D> =
    SharedBusMutex<mcu::i2c::I2c<'static, D, Async>>;

pub type SharedI2c0 = I2cDevice<'static, SharedBusMutexType, I2c<'static, I2C0, mcu::i2c::Async>>;
pub type SharedI2c1 = I2cDevice<'static, SharedBusMutexType, I2c<'static, I2C1, mcu::i2c::Async>>;
