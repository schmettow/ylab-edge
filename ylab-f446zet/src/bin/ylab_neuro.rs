#![no_std]
#![no_main]

const BAUD: u32 = 2_000_000;

use ylab_lib as yll;
use ylab::mcu;

use ylab::ytfk::bsu;
use mcu::usart::{Config, Uart};
use mcu::{bind_interrupts, peripherals, usart};

use mcu::mode::Async;
//use mcu::peripherals::{DMA1_CH3, DMA1_CH4, SPI2};
use mcu::spi::{Config as SpiConfig, Spi};

//use ads129x::{Ads129x, ConfigRegisters, Error};
use yll::ysns::yds1299 as yds;
use yds::Command;

// Für Logging / Defmt
use defmt_rtt as _;
use panic_probe as _;
use log::debug;

//use embassy_embedded_hal::shared_bus::spi::SpiDevice;
use yll::{Mutex, NoopRawMutex, StaticCell};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;

static SPI_BUS: StaticCell<Mutex<NoopRawMutex, Spi<Async>>> = StaticCell::new();

bind_interrupts!(struct Irqs {
        USART2 => usart::InterruptHandler<peripherals::USART2>;
        USART3 => usart::InterruptHandler<peripherals::USART3>;
    });

#[embassy_executor::main]
async fn main(spawner: yll::Spawner) {
    defmt::info!("STM32F446ZE ADS1299 example init");

    // init peripherals
    let p = embassy_stm32::init(Default::default());
    let mut config = Config::default();
    config.baudrate = BAUD;
    let usart = p.USART2;
    let tx = p.PA3;
    let rx = p.PA2;
    //let usart_dma = p.DMA1_CH6;
    let usart = Uart::new(usart, tx, rx, Irqs, p.DMA1_CH6, p.DMA1_CH5, config);
    match usart {
        Ok(usart) => spawner.spawn(bsu::task(usart)).unwrap(),
        Err(_)  => {debug!("USART connection failed")},
    }

    // pins (be sure these match your wiring)
    let sck = p.PB10;
    let mosi = p.PC3;
    let miso = p.PC2;
    let cs_pin = p.PB9;

    // SPI config
    let spi_cfg = SpiConfig::default();
    /*spi_cfg.frequency = 1_000_000;
    spi_cfg.phase = embassy_stm32::spi::Phase::CaptureOnFirstTransition;
    spi_cfg.polarity = embassy_stm32::spi::Polarity::IdleLow;*/

    // --- IMPORTANT: pick DMA channels that are valid for SPI1 on F446ZE ---
    // Example channels — VERIFY these against `p` for your specific chip!
    // On many F4 parts SPI1 uses DMA2 streams/channels; check `embassy_stm32::peripherals`.
    let tx_dma = p.DMA1_CH4; // <-- verify on your device
    let rx_dma = p.DMA1_CH3; // <-- verify on your device

    // create the async Spi driver (this returns Spi<'d, Async>)
    let spi = Spi::new(p.SPI2, sck, mosi, miso, tx_dma, rx_dma, spi_cfg);

    // wrap the spi into an embassy_sync::Mutex so it can be shared
    let spi_bus = yll::Mutex::new(spi);

    // make it 'static: initialize the StaticCell
    let spi_bus = SPI_BUS.init(spi_bus);

    // create a CS output pin (adjust constructor to your HAL's Output API)
    // embassy_stm32's Output::new signature may require OutputDrive; check your version.
    let cs = embassy_stm32::gpio::Output::new(cs_pin, mcu::gpio::Level::High, mcu::gpio::Speed::High);

    // CORRECT: construct a SpiDevice on top of the shared bus (not Device::new(spi,...))
    let spi_dev = SpiDevice::new(spi_bus, cs);
    debug!("SPI device created");

    // now create the ADS driver with the SpiDevice
    let mut sensor = yds::Sensor::new(spi_dev, 0, 100);
    debug!("Sensor device created");

    /*match sensor.init().await {
        Ok(_) => {
            debug!("Sensor init OK");
        }
        Err(e) => {
            debug!("Sensor init failed");
        }
    };*/

    if let Ok(_) = sensor.sensor.write_command_async(Command::RESET).await {
        debug!("Sensor reset OK");
    }
    if let Ok(_) = sensor.sensor.write_command_async(Command::WAKEUP).await {
        debug!("Sensor wakeup OK");
    }
    if let Ok(_) = sensor.sensor.write_command_async(Command::RDATAC).await {
        debug!("Sensor continuous sampling OK");
    }
    if let Ok(_) = sensor.sensor.write_command_async(Command::START).await {
        debug!("Sensor start OK");
    }

    if let Ok(_) = sensor.sensor.read_device_id_async().await {
        debug!("Sensor ID OK");
    } else {
        debug!("Sensor ID OK");
    }

    let mut count = 0;
    loop {
        if let Ok(s) = sensor.sample().await {
            count += 1;
            if count % 1000 == 0 {
                let y: crate::bsu::Ytf = s.clone().into();
                debug!("{}: {:?}", count, y.read);
            };
            bsu::SINK.send(s.into()).await;
        } else {
            debug!("Reading failed")
        }
    }
}
