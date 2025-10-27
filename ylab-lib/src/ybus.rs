pub use super::*;
//pub use embassy_sync::blocking_mutex::raw::NoopRawMutex as SharedBusMutex;
pub use embassy_sync::blocking_mutex::raw::RawMutex as SharedDeviceMutex;
pub type SharedI2cDevice<'a, M, BUS> =
    embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<'a, M, BUS>;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
pub type SharedBusMutex<I> = embassy_sync::mutex::Mutex<NoopRawMutex, I>;
