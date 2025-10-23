#![no_std]

pub use core::sync::atomic::AtomicBool;
pub use core::sync::atomic::Ordering;
pub use embassy_embedded_hal as hal;
pub use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as RawMutex;
pub use embassy_sync::channel::Channel;
pub use embassy_sync::mutex::Mutex;
pub use embassy_sync::signal::Signal;
pub use embassy_time as time;
pub use heapless::{String, Vec};
pub use static_cell::StaticCell;
pub use time::{Delay, Duration, Instant, Ticker, Timer};
pub static ORD: Ordering = Ordering::SeqCst;

pub mod ybus;
pub mod ydata;
