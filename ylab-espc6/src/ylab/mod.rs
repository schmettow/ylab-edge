#![no_std]

pub use crate::mcu::Async;
pub use esp_hal as mcu;
pub use esp_println::println;
pub use mcu::gpio::AnyPin;
pub use mcu::i2c::master::I2c;
//pub use mcu::peripherals::Peripheral as Peri;
pub use defmt::{debug, info};
pub use ylab_lib as yll;
pub use yll::ybus::*;
pub use yll::yuii;
pub use yll::yuio;

pub mod ysns; // Ylab sensors
pub mod ytfk;
//pub mod yuii; // YLab UI Input
//pub mod yuio; // YLab UI Output // YLab transfer formats & kodices
pub mod task;

//pub use mcu::peripherals::I2C0;
use crate::ehal::shared_bus::asynch::i2c::I2cDevice;

type SharedBusMutexType = embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
pub type SharedBusMutex<I> = embassy_sync::mutex::Mutex<SharedBusMutexType, I>;
pub type SharedI2cBus = SharedBusMutex<I2c<'static, Async>>;

pub type SharedI2c = I2cDevice<'static, SharedBusMutexType, I2c<'static, Async>>;
/*pub use mcu::peripherals::I2C0;
pub type SharedI2c0<'a> =
    I2cDevice<'static, SharedBusMutexType, I2c<'static, I2C0<'a>, mcu::i2c::Async>>;*/
