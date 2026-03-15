#![no_std]
#![no_main]

use task;
use ylab::*;
use ylab::mcu;
use ylab::ysns::adc as yadc;
use ylab::ytfk::bsu as ybsu;


#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state
    PartialEq, Eq, )] // testing equality
enum AppState {Send}

use mcu::adc;
use mcu::exti::ExtiInput;
use mcu::usart::{Config, Uart};
use mcu::i2c;
use mcu::{bind_interrupts, peripherals, usart};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
    USART3 => usart::InterruptHandler<peripherals::USART3>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
    I2C3_EV => i2c::EventInterruptHandler<peripherals::I2C3>;
    I2C3_ER => i2c::ErrorInterruptHandler<peripherals::I2C3>;
    //I2C4_EV => i2c::EventInterruptHandler<peripherals::I2C4>;
    //I2C4_ER => i2c::ErrorInterruptHandler<peripherals::I2C4>;
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

    spawner.spawn(task::moi_task(moi_0, moi_1, moi_3, moi_4)).unwrap();

    //ADC
    //let mut delay = Delay;
    let adc1 = adc::Adc::new(p.ADC1);
    spawner.spawn(yadc::adcbank_1(adc1,
                                (p.PA0, p.PA1, p.PA4, p.PB0, p.PC1, p.PC0, p.PC3, p.PC2),
                                50, 1)).unwrap();

    // I2C1
    let i2c1 = I2c::new(p.I2C1, p.PB8, p.PB9, Irqs, p.DMA1_CH6, p.DMA1_CH0, Default::default());
    static I2C_BUS_1: StaticCell<SharedI2cBus> = StaticCell::new();
    let i2c_bus_1 = I2C_BUS_1.init(Mutex::new(i2c1));
    let i2c11 = SharedI2cDevice::new(i2c_bus_1);
    let i2c12 = SharedI2cDevice::new(i2c_bus_1);
    // I2C 2
    /*let i2c2 = I2c::new(p.I2C2, p.PF1, p.PB11, Irqs, p.DMA1_CH7, p.DMA1_CH3, Default::default());
    static I2C_BUS_2: StaticCell<SharedI2cBus> = StaticCell::new();
    let i2c_bus_2 = I2C_BUS_2.init(Mutex::new(i2c2));*/

    // I2C 3
    let i2c3 = I2c::new(p.I2C3, p.PA8, p.PB4, Irqs, p.DMA1_CH4, p.DMA1_CH2, Default::default());
    static I2C_BUS_3: StaticCell<SharedI2cBus> = StaticCell::new();
    let i2c_bus_3 = I2C_BUS_3.init(Mutex::new(i2c3));
    let i2c31 = SharedI2cDevice::new(i2c_bus_3);
    let i2c32 = SharedI2cDevice::new(i2c_bus_3);
    /*// I2C 4
    let i2c4 = I2c::new(p.I2C4, p.PF14, p.PF15, Irqs, p.DMA1_CH2, p.DMA1_CH5, Default::default());
    static I2C_BUS_4: StaticCell<SharedI2cBus> = StaticCell::new();
    let i2c_bus_4 = I2C_BUS_4.init(Mutex::new(i2c4));*/


    spawner.spawn(task::co2_task(i2c11, 5, 4)).unwrap();
    spawner.spawn(task::yirt_task(i2c32, ylab_lib::ysns::yirt_max::SamplingRate::Sps50, 2)).unwrap();






}


#[embassy_executor::task]
async fn control_task() {
    let _state = AppState::Send;
    yadc::SAMPLE.store(true, ORD);
    //moi::SAMPLE.store(true, ORD);
}
