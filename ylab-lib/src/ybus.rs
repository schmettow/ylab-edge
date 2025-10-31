pub use super::*;
//pub use embassy_sync::blocking_mutex::raw::NoopRawMutex as SharedBusMutex;
pub use embassy_sync::blocking_mutex::raw::RawMutex as SharedDeviceMutex;
pub type SharedBusMutex<I> = embassy_sync::mutex::Mutex<NoopRawMutex, I>;
pub type SharedI2cDevice<'a, M, BUS> =
    embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<'a, M, BUS>;

/*pub type SharedSpiDevice<'a, M, BUS> =
        ehal::shared_bus::asynch::spi::SpiDevice<'a, M, BUS, Input>;*/
