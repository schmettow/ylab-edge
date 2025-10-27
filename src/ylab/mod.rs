#![no_std]

pub use embassy_rp as mcu;
pub use mcu::gpio::AnyPin;
pub use mcu::Peri;
pub use ylab_lib::yuii;
pub use ylab_lib::*;
pub use ylab_lib::ybus::*;

pub mod ysns; // Ylab sensors
pub mod ytfk;
//pub mod yuii; // YLab UI Input
pub mod yuio; // YLab UI Output // YLab transfer formats & kodices

pub use mcu::peripherals::I2C0;
pub use mcu::peripherals::I2C1;

use crate::ehal::shared_bus::asynch::i2c::I2cDevice;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
pub type SharedBusMutex<I> = embassy_sync::mutex::Mutex<CriticalSectionRawMutex, I>;
pub type SharedI2cBus<D> =
    SharedBusMutex<mcu::i2c::I2c<'static, D, mcu::i2c::Async>>;
pub type I2c1 = I2cDevice<'static, CriticalSectionRawMutex, mcu::i2c::I2c<'static, I2C1, crate::mcu::i2c::Async>>;
