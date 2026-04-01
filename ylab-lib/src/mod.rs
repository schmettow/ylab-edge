#![no_std]

pub mod pca9548a;

pub use core::fmt::Write;
pub use core::sync::atomic::AtomicBool;
pub use core::sync::atomic::Ordering;
pub use defmt::Format;
pub use defmt::println;
pub use embassy_embedded_hal as ehal;
pub use embassy_executor::Spawner;
pub use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as RawMutex;
pub use embassy_sync::blocking_mutex::raw::NoopRawMutex;
pub use embassy_sync::channel::Channel;
pub use embassy_sync::mutex::Mutex;
pub use embassy_sync::signal::Signal;
pub use embassy_time as time;
pub use heapless::{String, Vec};
pub use log;
pub use static_cell::StaticCell;

pub use time::{Delay, Duration, Instant, Ticker, Timer};
pub static ORD: Ordering = Ordering::SeqCst;

type SharedBusMutexType = embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
pub type SharedBusMutex<I> = embassy_sync::mutex::Mutex<SharedBusMutexType, I>;

pub mod ybus;
pub mod ydata;
pub mod ysns;
pub mod ytfk;
pub mod yuii;
pub mod yuio;

//pub use ybus::SharedI2cBus;
//pub use ybus::SharedI2cDevice;
