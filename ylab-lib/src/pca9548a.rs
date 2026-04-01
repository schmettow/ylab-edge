#![allow(dead_code)]
//! PCA9548A async I2C multiplexer proxy driver for Embassy/embedded-hal-async
//!
//! This module provides a proxy type for a single channel of a PCA9548A (or TCA9548A) I2C multiplexer,
//! allowing you to transparently use any async I2C device (including Embassy shared-bus devices) as the backend.
//!
//! # Features
//! - Generic over any `embedded_hal_async::i2c::I2c` implementation
//! - Channel selection is handled automatically before each I2C operation
//! - Suitable for use with Embassy shared-bus (`embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice`)
//! - Helper function to create a shared-bus device for a given channel
//!
//! # Example usage
//! ```ignore
//! use ylab_lib::pca9548a::{Pca9548aProxy, split_channel_shared};
//! use static_cell::StaticCell;
//! static MUX_PROXY: StaticCell<Pca9548aProxy<SharedI2cDevice>> = StaticCell::new();
//! let channel_dev = split_channel_shared(&MUX_PROXY, shared_i2c, 0x70, 3);
//! // Now use `channel_dev` as a normal I2C device (e.g. pass to sensor tasks)
//! ```

use embedded_hal_async::i2c::{ErrorType, I2c};
use static_cell::StaticCell;

/// PCA9548A I2C multiplexer proxy for a single channel.
///
/// This proxy wraps an underlying async I2C device and automatically selects the desired channel
/// before each I2C operation. It implements `embedded_hal_async::i2c::I2c` so it can be used
/// transparently with Embassy shared-bus and async sensor drivers.
///
/// - `I`: The underlying I2C bus/device type.
///
pub struct Pca9548aProxy<I> {
    backend: I,
    mux_addr: u8,
    channel: u8,
}

impl<I> Pca9548aProxy<I> {
    /// Create a new proxy for a given channel (0-7) on the PCA9548A at the given address.
    pub fn new(backend: I, mux_addr: u8, channel: u8) -> Self {
        assert!(channel < 8, "PCA9548A channel must be 0..=7");
        Self {
            backend,
            mux_addr,
            channel,
        }
    }

    /// Get the channel number for this proxy.
    pub fn channel(&self) -> u8 {
        self.channel
    }

    /// Get the I2C address of the mux.
    pub fn mux_addr(&self) -> u8 {
        self.mux_addr
    }

    /// Release the backend I2C device.
    pub fn release(self) -> I {
        self.backend
    }

    async fn select_channel(&mut self) -> Result<(), <I as ErrorType>::Error>
    where
        I: I2c,
    {
        let select = 1u8 << self.channel;
        self.backend.write(self.mux_addr, &[select]).await
    }
}

// Implement embedded_hal_async::i2c::ErrorType for the proxy.
impl<I> ErrorType for Pca9548aProxy<I>
where
    I: I2c,
{
    type Error = I::Error;
}

// Implement embedded_hal_async::i2c::I2c for the proxy.
// Each operation first selects the channel, then delegates to the backend.
impl<I> I2c for Pca9548aProxy<I>
where
    I: I2c,
{
    async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.select_channel().await?;
        self.backend.read(address, buffer).await
    }

    async fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.select_channel().await?;
        self.backend.write(address, bytes).await
    }

    async fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.select_channel().await?;
        self.backend.write_read(address, bytes, buffer).await
    }

    async fn transaction<'a>(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_async::i2c::Operation<'a>],
    ) -> Result<(), Self::Error> {
        self.select_channel().await?;
        self.backend.transaction(address, operations).await
    }
}

/// Helper to create a shared-bus device for a given PCA9548A channel.
///
/// - `mutex_cell`: A `StaticCell` for the mutex allocation (must be provided by the caller).
/// - `shared`: The shared-bus device for the upstream bus (e.g., `SharedI2cDevice`).
/// - `mux_addr`: The I2C address of the PCA9548A (usually 0x70..0x77).
/// - `channel`: The channel number (0..=7).
///
/// Returns a new `SharedI2cDevice` for the selected channel.
///
/// # Example
/// ```ignore
/// static MUX_MUTEX: StaticCell<Mutex<CriticalSectionRawMutex, Pca9548aProxy<SharedI2cDevice>>> = StaticCell::new();
/// let channel_dev = split_channel_shared(&MUX_MUTEX, shared_i2c, 0x70, 2);
/// ```
pub fn split_channel_shared<I, M>(
    mutex_cell: &'static StaticCell<
        embassy_sync::mutex::Mutex<
            M,
            Pca9548aProxy<embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<'static, M, I>>,
        >,
    >,
    shared: embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<'static, M, I>,
    mux_addr: u8,
    channel: u8,
) -> embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<
    'static,
    M,
    Pca9548aProxy<embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<'static, M, I>>,
>
where
    I: I2c + 'static,
    M: embassy_sync::blocking_mutex::raw::RawMutex + 'static,
{
    let proxy = Pca9548aProxy::new(shared, mux_addr, channel);
    let mutex = mutex_cell.init(embassy_sync::mutex::Mutex::new(proxy));
    embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice::new(mutex)
}
