#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

//use core::str::SplitAsciiWhitespace;

use mcu::gpio;
use ylab;
use ylab::mcu;
use ylab::println;
use ylab::{Mutex, SharedI2cDevice, StaticCell};
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
    let moi_1 = gpio::Input::new(p.GPIO10, config);
    let moi_2 = gpio::Input::new(p.GPIO11, config);
    let moi_3 = gpio::Input::new(p.GPIO12, config);
    spawner
        .spawn(ylab::task::moi_task(moi_0, moi_1, moi_2, moi_3))
        .unwrap();

    // I2C
    let i2c = mcu::i2c::master::I2c::new(p.I2C0, mcu::i2c::master::Config::default())
        .unwrap()
        .with_sda(p.GPIO6)
        .with_scl(p.GPIO7)
        .into_async();

    static I2C_BUS: StaticCell<ylab::SharedI2cBus> = ylab::StaticCell::new();
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));
    match spawner.spawn(ylab::task::co2_task(SharedI2cDevice::new(i2c_bus), 3)) {
        Ok(_) => {}
        Err(e) => {
            println!("# Failed to spawn co2 task: {:?}", e);
        }
    }

    // ADC
    let adc_controller = p.ADC1;
    match spawner.spawn(ylab::ysns::adc::task_gpio0_3(
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
    }
}
