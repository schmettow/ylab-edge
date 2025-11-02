#![no_std]
#![no_main]


use embassy_stm32 as mcu;
use ylab::*;
use ylab::ysns::adc as yadc;
use ylab::ytfk::bsu as ybsu;

use ylab_lib as yll;
use yll::ysns::moi;

#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state
    PartialEq, Eq, )] // testing equality
enum AppState {Send}

use mcu::adc;
use mcu::exti::ExtiInput;
/// USB
//use mcu::dma::NoDma;
use mcu::usart::{Config, Uart};
use mcu::{bind_interrupts, peripherals, usart};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
    USART3 => usart::InterruptHandler<peripherals::USART3>;
});

use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = mcu::init(Default::default());
    let mut config = Config::default();
    config.baudrate = 2000;
    let usart = p.USART2;
    let tx = p.PA3;
    let rx = p.PA2;
    //let usart_dma = p.DMA1_CH6;
    let usart = Uart::new(usart, tx, rx, Irqs, p.DMA1_CH6, p.DMA1_CH5, config);
    match usart {
        Ok(usart) => spawner.spawn(ybsu::task(usart)).unwrap(),
        Err(_)  => {println!("USART connection failed")},
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
}


/*use mcu::gpio::Input;
use mcu::gpio::Pull;
use mcu::peripherals::{PD0, PD1, PD2, PD3};*/

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
    yadc::SAMPLE.store(true, ORD);
    //moi::SAMPLE.store(true, ORD);
}
