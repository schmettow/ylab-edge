#![no_std]
#![no_main]

//use ylab::yllsns::yds1299::descriptors::*;
use ylab::{Ticker, mcu};
use ylab_lib as yll;

use mcu::usart::{Config, Uart};
use ylab::ytfk::bsu as ybsu;
//use embassy_stm32::dma::NoDma;
use mcu::{bind_interrupts, peripherals, usart};

use mcu::mode::Async;
//use mcu::peripherals::{DMA1_CH3, DMA1_CH4, SPI2};
use mcu::spi::{Config as SpiConfig, Spi};

//use ads129x::{Ads129x, ConfigRegisters, Error};
use yds::Command;
use yll::ysns::yds1299 as yds;

// Für Logging / Defmt
use defmt::println;
use {defmt_rtt as _, panic_probe as _};
//use log::debug;

use embassy_embedded_hal;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use yll::{Mutex, NoopRawMutex, StaticCell};

static SPI_BUS: StaticCell<Mutex<NoopRawMutex, Spi<Async>>> = StaticCell::new();

bind_interrupts!(struct Irqs {
    //USART2 => usart::InterruptHandler<peripherals::USART2>;
    UART7 => usart::InterruptHandler<peripherals::UART7>;
});

#[embassy_executor::main]
async fn main(spawner: yll::Spawner) {
    let mut config = mcu::Config::default();
    {
        use mcu::rcc::*;
        // config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.hsi = None; // Since we're using HSE
        config.rcc.hse = Some(Hse {
            freq: mcu::time::mhz(20),
            mode: HseMode::Bypass,
        });
        config.rcc.csi = false;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSE,

            prediv: PllPreDiv::DIV2,

            mul: PllMul::MUL32,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8),
            divr: Some(PllDiv::DIV2),
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale3;
        //config.rcc.supply_config = SupplyConfig::DirectSMPS;
    }
    let p = embassy_stm32::init(config);
    //let p = embassy_stm32::init(Default::default());
    let mut config = Config::default();
    config.baudrate = 2_000_000;
    let usart = Uart::new(p.UART7, p.PF6, p.PF7, Irqs, p.DMA1_CH0, p.DMA1_CH1, config).unwrap();
    match spawner.spawn(ybsu::task(usart)) {
        Ok(_) => {
            println!("USART OK")
        }
        Err(e) => {
            println!("USART connection failed: {:?}", e)
        }
    }

    // SPI config
    let spi_cfg = SpiConfig::default();
    /*spi_cfg.frequency = 1_000_000;
    spi_cfg.phase = embassy_stm32::spi::Phase::CaptureOnFirstTransition;
    spi_cfg.polarity = embassy_stm32::spi::Polarity::IdleLow;*/

    // create the async Spi driver (this returns Spi<'d, Async>)
    let spi = Spi::new(
        p.SPI2, p.PB10, p.PC3, p.PC2, p.DMA1_CH4, p.DMA1_CH3, spi_cfg,
    );

    // wrap the spi into an embassy_sync::Mutex so it can be shared
    let spi_bus = yll::Mutex::new(spi);

    // make it 'static: initialize the StaticCell
    let spi_bus = SPI_BUS.init(spi_bus);

    // create a CS output pin (adjust constructor to your HAL's Output API)
    // embassy_stm32's Output::new signature may require OutputDrive; check your version.
    let cs =
        embassy_stm32::gpio::Output::new(p.PB9, mcu::gpio::Level::High, mcu::gpio::Speed::High);

    // CORRECT: construct a SpiDevice on top of the shared bus (not Device::new(spi,...))
    let spi_dev = SpiDevice::new(spi_bus, cs);
    println!("SPI device created");

    // now create the ADS driver with the SpiDevice
    let mut sensor = yds::Sensor::new(spi_dev, 0, 100);
    println!("Sensor device created");

    /*match sensor.init().await {
        Ok(_) => {
            println!("Sensor init OK");
        }
        Err(e) => {
            println!("Sensor init failed");
        }
    };*/

    if let Ok(_) = sensor.sensor.write_command_async(Command::RESET).await {
        println!("Sensor reset OK");
    }
    if let Ok(_) = sensor.sensor.write_command_async(Command::WAKEUP).await {
        println!("Sensor wakeup OK");
    }

    if let Ok(_) = sensor.sensor.write_command_async(Command::RDATAC).await {
        println!("Sensor continuous sampling OK");
    }
    if let Ok(_) = sensor.sensor.write_command_async(Command::START).await {
        println!("Sensor start OK");
    }

    if let Ok(_) = sensor.sensor.read_device_id_async().await {
        println!("Sensor ID OK");
    } else {
        println!("Sensor ID OK");
    }

    let mut ticker = Ticker::every(ylab::Duration::from_millis(4));
    let mut count = 0;
    loop {
        if let Ok(s) = sensor.sample().await {
            count += 1;
            if count % 10 == 0 {
                let y: yll::ydata::Ytf = s.clone().into();
                println!("{}: {:?}", count, y.read);
            };
            ybsu::SINK.send(s.into()).await;
        } else {
            println!("Reading failed")
        }
        ticker.next().await;
    }
}
