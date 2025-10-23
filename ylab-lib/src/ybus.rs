pub use super::*;
pub use embassy_sync::blocking_mutex::raw::NoopRawMutex;
pub type GenAsyncI2cBus<I> = embassy_sync::mutex::Mutex<NoopRawMutex, I>;
pub type AsyncI2cDevice<'a, M, BUS> =
    embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<'a, M, BUS>;
