#![no_std]
#![no_main]


use embassy_stm32 as mcu;
use ylab::*;
use ylab::ysns::adc as yadc;
use ylab::ytfk::bsu as ybsu;
use ylab::task;

//use ylab_lib as yll;
//use yll::ysns::moi;

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
    config.baudrate = 2_000_000;
    let usart = Uart::new(p.USART3, p.PD9, p.PD8, Irqs, p.DMA1_CH3, p.DMA1_CH1, config).unwrap();
    match spawner.spawn(ybsu::task(usart)) {
        Ok(_) => {println!("USART OK")},
        Err(e)  => {println!("USART connection failed: {:?}", e)},
    }
    match spawner.spawn(control_task()){
    	Ok(_) => println!("Control task OK"),
     	Err(e) => println!("Control task failed: {:?}", e),
    };
    // MOI
    let moi_0
        = ExtiInput::new(p.PA10,  p.EXTI10, ylab::Pull::Down,);
    let moi_1
        = ExtiInput::new(p.PB3, p.EXTI3, ylab::Pull::Down);
    let moi_3
        = ExtiInput::new(p.PD0,  p.EXTI0, ylab::Pull::Down,);
    let moi_4
        = ExtiInput::new(p.PD1, p.EXTI1, ylab::Pull::Down);

    match spawner.spawn(task::moi_task(moi_0, moi_1, moi_3, moi_4)) {
    	Ok(_) => println!("MOI task OK"),
   		Err(e) => println!("MOI task failed: {:?}", e),
    }

    //ADC
    //let mut delay = Delay;
    let adc1 = adc::Adc::new(p.ADC1);
    match spawner.spawn(yadc::adcbank_1(adc1,
                                (p.PA0, p.PA1, p.PA4, p.PB0, p.PC1, p.PC0, p.PC3, p.PC2),
                                3, 1)) {
                                	Ok(_) => println!("ADC task OK"),
                              		Err(e) => println!("ADC task failed: {:?}", e),
                                }
}


/*use mcu::gpio::Input;
use mcu::gpio::Pull;
use mcu::peripherals::{PD0, PD1, PD2, PD3};*/

/*#[embassy_executor::task]
async fn moi_task(
    pin_0: ExtiInput<'static>,
    pin_1: ExtiInput<'static>,
    pin_2: ExtiInput<'static>,
    pin_3: ExtiInput<'static>)
    {
	moi::inner_task(pin_0, pin_1, pin_2, pin_3, 0, ylab::ytfk::bsu::SINK.sender()).await;
}*/


#[embassy_executor::task]
async fn control_task() {
    let _state = AppState::Send;
    yadc::SAMPLE.store(true, ORD);
    //moi::SAMPLE.store(true, ORD);
}
