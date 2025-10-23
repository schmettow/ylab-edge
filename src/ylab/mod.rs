#![no_std]

pub use embassy_rp as hal;
pub use hal::gpio::AnyPin;
pub use hal::Peri;
pub use ylab_lib::yuii;
pub use ylab_lib::*;

pub mod ysns; // Ylab sensors
pub mod ytfk;
//pub mod yuii; // YLab UI Input
pub mod yuio; // YLab UI Output // YLab transfer formats & kodices

pub use hal::peripherals::I2C0;
pub use hal::peripherals::I2C1;
pub type AsyncI2cBus<D> =
    ylab_lib::ybus::GenAsyncI2cBus<hal::i2c::I2c<'static, D, hal::i2c::Async>>;
