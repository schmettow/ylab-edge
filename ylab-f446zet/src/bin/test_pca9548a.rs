#![no_std]
#![no_main]

//! Test program for STM32F446ZET: Demonstrates use of PCA9548A I2C multiplexer
//! with Embassy async I2C, shared-bus, and LSM6 sensor tasks.
//!
//! - Initializes I2C1 and I2C3 in async mode
//! - Sets up Embassy shared-bus for each
//! - Spawns LSM6 sensor task on:
//!     (a) plain shared bus device
//!     (b) PCA9548A channel device (using ylab_lib::ybus::split_mux_channel)

// Static channel for LSM6 sensor data
static LSM6_CHAN: Channel<CriticalSectionRawMutex, Ytf, 8> = Channel::new();
//
// This demonstrates that the sensor task is agnostic to the I2C backend.

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use static_cell::StaticCell;
use ylab::mcu;
use ylab_lib::pca9548a::{Pca9548aProxy, split_channel_shared};
use ylab_lib::ydata::Ytf;

use mcu::i2c::I2c;
use mcu::usart::{Config, Uart};
use mcu::{bind_interrupts, peripherals, usart};

bind_interrupts!(struct Irqs {
    USART3 => usart::InterruptHandler<peripherals::USART3>;
    I2C1_EV => mcu::i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => mcu::i2c::ErrorInterruptHandler<peripherals::I2C1>;
    I2C3_EV => mcu::i2c::EventInterruptHandler<peripherals::I2C3>;
    I2C3_ER => mcu::i2c::ErrorInterruptHandler<peripherals::I2C3>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = mcu::init(Default::default());

    // USART for debug output
    let mut config = Config::default();
    config.baudrate = 2_000_000;
    let _usart = Uart::new(p.USART3, p.PD9, p.PD8, Irqs, p.DMA1_CH3, p.DMA1_CH1, config).unwrap();
    defmt::println!("USART3 initialized");

    // I2C1 setup (for direct/shared LSM6)
    let i2c1 = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH0,
        Default::default(),
    );
    static I2C1_BUS: StaticCell<
        ylab::SharedBusMutex<ylab::I2c<'static, ylab::mcu::mode::Async, ylab::mcu::i2c::Master>>,
    > = StaticCell::new();
    let i2c1_bus = I2C1_BUS.init(Mutex::new(i2c1));
    let i2c1_dev = ylab::SharedI2cDevice::new(i2c1_bus);

    // I2C3 setup (for PCA9548A)
    let i2c3 = I2c::new(
        p.I2C3,
        p.PA8,
        p.PB4,
        Irqs,
        p.DMA1_CH4,
        p.DMA1_CH2,
        Default::default(),
    );
    static I2C3_BUS: StaticCell<ylab::SharedI2cBus> = StaticCell::new();
    let i2c3_bus = I2C3_BUS.init(Mutex::new(i2c3));
    let i2c3_dev = ylab::SharedI2cDevice::new(i2c3_bus);

    // PCA9548A address (A0/A1/A2 = 0)
    let mux_addr = 0x70;
    let mux_channel = 2; // Use channel 2 for this test

    // Create a shared-bus device for the mux channel using StaticCell for the mutex
    static MUX_MUTEX: StaticCell<
        Mutex<CriticalSectionRawMutex, Pca9548aProxy<ylab::SharedI2cDevice>>,
    > = StaticCell::new();
    let lsm6_mux_dev = split_channel_shared(&MUX_MUTEX, i2c3_dev, mux_addr, mux_channel);

    // Spawn LSM6 sensor task on I2C1 (direct/shared)
    spawner.spawn(lsm6_task_i2c1(i2c1_dev, 1, 50)).unwrap();

    // Spawn LSM6 sensor task on I2C3 via PCA9548A mux channel
    //spawner.spawn(lsm6_task_mux(lsm6_mux_dev, 2, 50)).unwrap();

    defmt::println!("Spawned LSM6 tasks: one direct, one via PCA9548A mux channel");
}

/// Concrete wrapper for the generic LSM6 task, so it can be spawned by Embassy.
/// This is needed because #[embassy_executor::task] does not support generics.
#[embassy_executor::task]
async fn lsm6_task_i2c1(i2c: ylab::SharedI2cDevice, id: u8, hz: u16) {
    defmt::println!("LSM6 task on I2C1");
    if let _ = ylab_lib::ysns::yxz_lsm6::task(i2c, hz as u64, id, LSM6_CHAN.sender()).await {
        defmt::println!("LSM6 task on I2C1 returned");
    }
}

#[embassy_executor::task]
async fn lsm6_task_mux(
    i2c: embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice<
        'static,
        CriticalSectionRawMutex,
        Pca9548aProxy<ylab::SharedI2cDevice>,
    >,
    id: u8,
    hz: u16,
) {
    defmt::println!("LSM6 task on I2C3 + Muxer");
    if let _ = ylab_lib::ysns::yxz_lsm6::task(i2c, hz as u64, id, LSM6_CHAN.sender()).await {
        defmt::println!("LSM6 task on Muxer on I2C3 returned");
    }
}
