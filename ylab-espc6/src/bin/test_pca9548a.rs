#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

//use core::str::SplitAsciiWhitespace;

use alloc::boxed::Box;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use mcu::gpio;
use ylab;
use ylab::mcu;
use ylab::println;
use ylab::{Mutex, SharedI2cDevice};
use ylab_lib as yll;
use yll::Spawner;
//use ylab::info;

//use bt_hci::controller::ExternalController;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
//use esp_radio::ble::controller::BleConnector;
//use trouble_host::prelude::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

//const CONNECTIONS_MAX: usize = 1;
//const L2CAP_CHANNELS_MAX: usize = 1;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

pub use embedded_hal_async::i2c::{Error, ErrorType, I2c};

/// TCA9548A I2C Multiplexer Proxy
///
/// This proxy wraps an I2C bus and automatically switches the mux channel
/// before each I2C transaction. It allows multiple sensors on different
/// mux channels to be accessed through the same I2C interface.
///
/// # Example
/// ```ignore
/// let mux = Pca9548aProxy::new(i2c_device, 0x70, 0); // Channel 0
/// mux.write(0x48, &[0x01, 0x02]).await?;
/// ```

pub struct Pca9548aProxy<I> {
    bus: I,
    mux_addr: u8,
    channel: u8,
}

impl<I> Pca9548aProxy<I> {
    /// Create a new mux proxy for a specific channel
    ///
    /// # Arguments
    /// * `bus` - The underlying I2C bus (typically a SharedI2cDevice)
    /// * `mux_addr` - I2C address of the TCA9548A mux (default 0x70)
    /// * `channel` - Which mux channel to connect to (0-7)
    pub fn new(bus: I, mux_addr: u8, channel: u8) -> Self {
        assert!(channel < 8, "TCA9548A only has 8 channels (0-7)");
        Self {
            bus,
            mux_addr,
            channel,
        }
    }

    /// Get the channel number
    pub fn channel(&self) -> u8 {
        self.channel
    }

    /// Get the mux address
    pub fn mux_addr(&self) -> u8 {
        self.mux_addr
    }
}

// Implement ErrorType trait for the Proxy
impl<I: I2c> ErrorType for Pca9548aProxy<I> {
    type Error = I::Error;
}

// Implement Async I2C for the Proxy
impl<I: I2c> I2c for Pca9548aProxy<I> {
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        // 1. Switch Mux to the correct channel
        self.bus.write(self.mux_addr, &[1 << self.channel]).await?;
        // 2. Perform the actual read from the sensor
        self.bus.read(address, read).await
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        // 1. Switch Mux to the correct channel
        self.bus.write(self.mux_addr, &[1 << self.channel]).await?;
        // 2. Perform the actual write to the sensor
        self.bus.write(address, write).await
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        // 1. Switch Mux to the correct channel
        self.bus.write(self.mux_addr, &[1 << self.channel]).await?;
        // 2. Perform the actual write_read from the sensor
        self.bus.write_read(address, write, read).await
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_async::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        // 1. Switch Mux to the correct channel
        self.bus.write(self.mux_addr, &[1 << self.channel]).await?;
        // 2. Perform the actual transaction from the sensor
        self.bus.transaction(address, operations).await
    }
}

/// Helper function to create a SharedI2cDevice for a specific TCA9548A mux channel
///
/// This simplifies using mux-based sensors with existing task functions.
/// The returned device automatically switches to the specified channel.
///
/// # Example
/// ```ignore
/// let i2c_ch0 = split_mux_channel(i2c_bus, 0x70, 0);
/// spawner.spawn(ylab::task::co2_task(i2c_ch0, 3)).unwrap();
///
/// let i2c_ch1 = split_mux_channel(i2c_bus, 0x70, 1);
/// spawner.spawn(ylab::task::lsm6_task(i2c_ch1, 100, 0x6a)).unwrap();
/// ```
fn split_mux_channel(
    i2c_bus: &'static Mutex<
        CriticalSectionRawMutex,
        ylab::mcu::i2c::master::I2c<'static, ylab::mcu::Async>,
    >,
    mux_addr: u8,
    channel: u8,
) -> SharedI2cDevice<
    'static,
    CriticalSectionRawMutex,
    &'static Pca9548aProxy<
        SharedI2cDevice<
            'static,
            CriticalSectionRawMutex,
            &'static Mutex<CriticalSectionRawMutex, ylab::mcu::i2c::master::I2c<'static, ylab::mcu::Async>>,
        >,
    >,
> {
    // Create proxy that wraps a SharedI2cDevice of the main I2C bus
    let proxy = Box::leak(Box::new(Pca9548aProxy::new(
        SharedI2cDevice::new(i2c_bus),
        mux_addr,
        channel,
    )));

    // Return as SharedI2cDevice - proxy is 'static and can be used directly
    SharedI2cDevice::new(proxy)
}

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // generator version: 1.0.1

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    let timg0 = TimerGroup::new(p.TIMG0);
    let sw_interrupt = esp_hal::interrupt::software::SoftwareInterruptControl::new(p.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    //println!("# Embassy initialized!");

    // BLE
    /*let radio_init = esp_radio::init().expect("# Failed to initialize Wi-Fi/BLE controller");
    // find more examples https://github.com/embassy-rs/trouble/tree/main/examples/esp32
    let transport = BleConnector::new(&radio_init, p.BT, Default::default()).unwrap();
    let ble_controller = ExternalController::<_, 20>::new(transport);
    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let _stack = trouble_host::new(ble_controller, &mut resources);*/

    match spawner.spawn(ylab::ytfk::bsu::task_println()) {
        Ok(_) => {}
        Err(e) => {
            println!("# Failed to spawn bsu::task_println: {:?}", e);
        }
    };

    let config = gpio::InputConfig::default().with_pull(gpio::Pull::Up);
    let moi_0 = gpio::Input::new(p.GPIO9, config);
    let moi_1 = gpio::Input::new(p.GPIO13, config);
    let moi_2 = gpio::Input::new(p.GPIO14, config);
    let moi_3 = gpio::Input::new(p.GPIO12, config);
    spawner
        .spawn(ylab::task::moi_task(moi_0, moi_1, moi_2, moi_3))
        .unwrap();

    // I2C
    let i2c0 = mcu::i2c::master::I2c::new(p.I2C0, mcu::i2c::master::Config::default())
        .unwrap()
        .with_sda(p.GPIO10)
        .with_scl(p.GPIO11)
        .into_async();

    /*use mcu::gpio::lp_io::LowPowerOutputOpenDrain;
    use mcu::time::Rate;
    let sda = LowPowerOutputOpenDrain::new(p.GPIO6);
    let scl = LowPowerOutputOpenDrain::new(p.GPIO7);
    let lp_i2c = esp_hal::i2c::lp_i2c::LpI2c::new(
        p.LP_I2C0,
        scl,
        sda,
        Rate::from_khz(100_u32), // Standard mode is safer for SCD41
    );*/

    // Create shared I2C bus using Box::leak to avoid static type inference issues
    let i2c_bus_0 = Box::leak(Box::new(Mutex::<CriticalSectionRawMutex, _>::new(i2c0)));

    // TCA9548A Mux Setup - Address 0x70
    const TCA9548A_ADDR: u8 = 0x70;

    // Create mux proxies for each channel
    // Each proxy automatically switches to its channel before I2C operations

    // Channel 0: CO2 Sensor (SCD41)
    let i2c_ch0 = split_mux_channel(i2c_bus_0, TCA9548A_ADDR, 0);
    println!("# TCA9548A Channel 0 ready for CO2 sensor (SCD41)");
    spawner.spawn(ylab::task::co2_task(i2c_ch0, 3)).unwrap();

    // Channel 1: LSM6 Accelerometer/Gyro
    let i2c_ch1 = split_mux_channel(i2c_bus_0, TCA9548A_ADDR, 1);
    println!("# TCA9548A Channel 1 ready for LSM6 accel/gyro");
    spawner
        .spawn(ylab::task::lsm6_task(i2c_ch1, 100, 0x6a))
        .unwrap();

    // Channel 2: BME280 Environmental Sensor
    let _i2c_ch2 = split_mux_channel(i2c_bus_0, TCA9548A_ADDR, 2);
    println!("# TCA9548A Channel 2 ready for BME280 environmental");

    println!(
        "# TCA9548A multiplexer initialized on address {:#x}",
        TCA9548A_ADDR
    );
    println!("# CO2 task on channel 0");
    println!("# LSM6 task on channel 1");
    println!("# Channel 2 ready for additional sensors");

    // ADC
    #[allow(unused_variables)]
    let adc_controller = p.ADC1;
    /*match spawner.spawn(ylab::ysns::adc::task_gpio0_3(
        adc_controller,
        p.GPIO0,
        p.GPIO1,
        p.GPIO2,
        p.GPIO3,
        1,
        1,
    )) {
        Ok(_) => {}
        Err(e) => {
            println!("# Failed to spawn adc_task: {:?}", e);
        }
    }*/
}
