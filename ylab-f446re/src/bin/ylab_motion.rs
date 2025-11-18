#![no_std]
#![no_main]


use ylab_stm32 as ylab;
use ylab::ytfk::bsu;
use ylab::{mcu, println, task,
				Mutex, StaticCell,
				Pull, ExtiInput,
				UartInterruptHandler, Uart, UartConfig,
				i2c, I2c, SharedI2cBus, SharedI2cDevice,
				bind_interrupts, peripherals,
				Spawner};
use task::{lsm6_multi_task, bmi160_task};

bind_interrupts!(struct Irqs {
    USART2 => UartInterruptHandler<peripherals::USART2>;
    USART3 => UartInterruptHandler<peripherals::USART3>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    I2C3_EV => i2c::EventInterruptHandler<peripherals::I2C3>;
    I2C3_ER => i2c::ErrorInterruptHandler<peripherals::I2C3>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = mcu::init(Default::default());
    let mut config = UartConfig::default();
    config.baudrate = 2_000_000;
    let usart = Uart::new(p.USART3, p.PC11, p.PC10, Irqs, p.DMA1_CH3, p.DMA1_CH1, config).unwrap();
    match spawner.spawn(bsu::task(usart)) {
        Ok(_) => {println!("USART OK")},
        Err(e)  => {println!("USART connection failed: {:?}", e)},
    }
    match spawner.spawn(control_task()){
    	Ok(_) => println!("Control task OK"),
     	Err(e) => println!("Control task failed: {:?}", e),
    };
    // MOI
    let moi_0
        = ExtiInput::new(p.PA10,  p.EXTI10, Pull::Down,);
    let moi_1
        = ExtiInput::new(p.PB3, p.EXTI3, Pull::Down);
    let moi_3
        = ExtiInput::new(p.PA0,  p.EXTI0, Pull::Down,);
    let moi_4
        = ExtiInput::new(p.PA1, p.EXTI1, Pull::Down);

    match spawner.spawn(task::moi_task(moi_0, moi_1, moi_3, moi_4)) {
    	Ok(_) => println!("MOI task OK"),
   		Err(e) => println!("MOI task failed: {:?}", e),
    }
    //ADC
    /*let adc1 = adc::Adc::new(p.ADC1);
    match spawner.spawn(yadc::adcbank_1(adc1,
                                (p.PA0, p.PA1, p.PA4, p.PB0, p.PC1, p.PC0, p.PC3, p.PC2),
                                2, 1)) {
                                	Ok(_) => println!("ADC task OK"),
                              		Err(e) => println!("ADC task failed: {:?}", e),
                                }*/

    let i2c1 = I2c::new(p.I2C1, p.PB8, p.PB9, Irqs, p.DMA1_CH7, p.DMA1_CH0, Default::default());
    static I2C_BUS_1: StaticCell<SharedI2cBus> = StaticCell::new();
    let i2c_bus_1 = I2C_BUS_1.init(Mutex::new(i2c1));
    let i2c11 = SharedI2cDevice::new(i2c_bus_1);
    let i2c12 = SharedI2cDevice::new(i2c_bus_1);
    let _i2c13 = SharedI2cDevice::new(i2c_bus_1);
    let _i2c14 = SharedI2cDevice::new(i2c_bus_1);


    match spawner.spawn(bmi160_task(i2c11, 5, 2)) {
   		Ok(_) => println!("LSM task OK"),
 		Err(e) => println!("LSM task failed: {:?}", e),
    }

    match spawner.spawn(lsm6_multi_task(i2c12, 5, 2, 2)) {
   		Ok(_) => println!("Lsm6: multi task OK"),
 		Err(e) => println!("Lsm6 multi task failed: {:?}", e),
    }

    /*match spawner.spawn(task::ads_task(i2c13, 7, 3)) {
   		Ok(_) => println!("ADS task OK"),
 		Err(e) => println!("ADS task failed: {:?}", e),
    }*/

    /*match spawner.spawn(task::bmi160_task(i2c14, 11, 4)) {
   		Ok(_) => println!("Bmi160 task OK"),
 		Err(e) => println!("Bmi168 task failed: {:?}", e),
    }*/




}

#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state
    PartialEq, Eq, )] // testing equality
enum AppState {Send}


#[embassy_executor::task]
async fn control_task() {
    let _state = AppState::Send;
    //yadc::SAMPLE.store(true, ORD);
    //moi::SAMPLE.store(true, ORD);
}
