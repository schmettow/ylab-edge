#![no_std]
#![no_main]

use ylab::*;
use ylab::mcu;
use ylab::ysns::adc as yadc;
use ylab_lib::ysns::moi;


#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state
    PartialEq, Eq, )] // testing equality
enum AppState {Send}


use mcu::adc;
use mcu::exti::ExtiInput;
use ylab::ytfk::bsu;
use mcu::usart::{Config, Uart};
use mcu::i2c;
use mcu::{bind_interrupts, peripherals, usart};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
    USART3 => usart::InterruptHandler<peripherals::USART3>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

use embassy_executor::Spawner;
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = mcu::init(Default::default());

    let mut config = Config::default();
    config.baudrate = 2_000_000;
    let usart = p.USART2;
    let tx = p.PA3;
    let rx = p.PA2;
    //let usart_dma = p.DMA1_CH6;
    let usart = Uart::new(usart, tx, rx, Irqs, p.DMA1_CH6, p.DMA1_CH5, config);
    match usart {
        Ok(usart) => spawner.spawn(bsu::task(usart)).unwrap(),
        Err(_)  => {log::debug!("USART connection failed")},
    }
    spawner.spawn(control_task()).unwrap();


    // MOI
    let moi_0
        = ExtiInput::new(p.PA10,  p.EXTI10, ylab::Pull::Down,);
    let moi_1
        = ExtiInput::new(p.PB3, p.EXTI3, ylab::Pull::Down);
    let moi_3
        = ExtiInput::new(p.PD0,  p.EXTI0, ylab::Pull::Down,);
    let moi_4
        = ExtiInput::new(p.PD1, p.EXTI1, ylab::Pull::Down);
    //spawner.spawn(ysns::moi::task(moi_0, moi_1, 0)).unwrap();
    spawner.spawn(moi_task(moi_0, moi_1, moi_3, moi_4)).unwrap();

    //ADC
    //let mut delay = Delay;
    let adc1 = adc::Adc::new(p.ADC1);
    spawner.spawn(yadc::adcbank_1(adc1,
                                (p.PA0, p.PA1, p.PA4, p.PB0, p.PC1, p.PC0, p.PC3, p.PC2),
                                197, 1)).unwrap();


    let i2c1 = I2c::new(p.I2C1, p.PB8, p.PB9, Irqs, p.DMA1_CH7, p.DMA1_CH0, Default::default());
    static I2C_BUS_1: StaticCell<SharedI2cBus> = StaticCell::new();
    let i2c_bus_1 = I2C_BUS_1.init(Mutex::new(i2c1));
    let i2c11 = SharedI2cDevice::new(i2c_bus_1);
    spawner.spawn(co2_task(i2c11)).unwrap();
}

/// ## Control task
///
/// bare minimum for Pro

#[embassy_executor::task]
async fn co2_task(i2c: SharedI2cDevice) {
	ylab_lib::ysns::yco2::task(i2c,  2, ytfk::bsu::SINK.sender()).await;
}

#[embassy_executor::task]
async fn moi_task(
    pin_0: ExtiInput<'static>,
    pin_1: ExtiInput<'static>,
    pin_2: ExtiInput<'static>,
    pin_3: ExtiInput<'static>)
    {
	moi::inner_task(pin_0, pin_1, pin_2, pin_3, 0, ylab::ytfk::bsu::SINK.sender()).await;
}


#[embassy_executor::task]
async fn control_task() {
    let _state = AppState::Send;

    loop {
        Timer::after_millis(5).await;
        if yadc::READY.load(ORD) {
            yadc::SAMPLE.store(true, ORD);
            println!("ADC sampling started");
            break
        }
    }

    /*loop {
        Timer::after_millis(5).await;
        if yco2::READY.load(ORD) {
            yco2::SAMPLE.store(true, ORD);
            println!("CO2 sampling started");
            break
        }
    }


    yco2::SAMPLE.store(true, ORD);*/
}

/*pub use core::sync::atomic::Ordering;
//use ydsp::{FourLines, OneLine};
#[embassy_executor::task]
async fn control_task() {
    let _state = AppState::Send;
    yadc::SAMPLE.store(true, Ordering::Relaxed);
    let title: OneLine = "YLab".try_into().unwrap();
    let disp_text: FourLines = [ Some(title) ,None, None, None];
    ydsp::TEXT.signal(disp_text);
}*/
