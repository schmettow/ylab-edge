#![no_std]

pub use core::sync::atomic::AtomicBool;
pub use core::sync::atomic::Ordering;
pub use heapless::{String, Vec};
pub use static_cell::StaticCell;
pub use core::fmt::Write;
pub use log;
pub use defmt::println;
pub use defmt::Format;
pub use embassy_embedded_hal as ehal;
pub use embassy_executor::Spawner;
pub use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as RawMutex;
pub use embassy_sync::blocking_mutex::raw::NoopRawMutex;
pub use embassy_sync::channel::Channel;
pub use embassy_sync::mutex::Mutex;
pub use embassy_sync::signal::Signal;
pub use embassy_time as time;

pub use time::{Delay, Duration, Instant, Ticker, Timer};
pub static ORD: Ordering = Ordering::SeqCst;

pub mod ybus;
pub mod ydata;
pub mod ysns;
pub mod yuii;
pub mod ytfk;
pub mod yuio;

//pub use ybus::SharedI2cBus;
//pub use ybus::SharedI2cDevice;
