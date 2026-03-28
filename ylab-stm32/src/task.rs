pub use super::*;
//use ylab_lib::ysns::YsenseErr;
pub use crate::SharedI2cDevice;
pub use crate::println;
pub use crate::ysns;
pub use crate::ytfk::bsu::SINK;
pub use embassy_stm32 as mcu;
pub use mcu::exti::ExtiInput;
pub use ylab_lib as yll;

#[embassy_executor::task]
pub async fn moi_task(
    pin_0: ExtiInput<'static>,
    pin_1: ExtiInput<'static>,
    pin_2: ExtiInput<'static>,
    pin_3: ExtiInput<'static>,
) {
    ysns::moi::inner_task(pin_0, pin_1, pin_2, pin_3, 0, SINK.sender()).await;
}

/*use cortex_m::singleton;
use mcu::Peripherals;
use mcu::adc::{Adc, AdcChannel, RingBufferedAdc, SampleTime};
#[embassy_executor::task]
pub async fn adc_task ( p:Peripherals,
                        id: u8,
                        hz: u64)
    {

}*/

#[embassy_executor::task]
pub async fn sen5_task(i2c: SharedI2cDevice, id: u8) {
    ysns::sen_five::task(i2c, Duration::from_secs(5), id, SINK.sender()).await;
}

#[embassy_executor::task]
pub async fn yirt_task(i2c: SharedI2cDevice, rate: ysns::yirt_max::SamplingRate, id: u8) {
    match ysns::yirt_max::task(i2c, rate, id, SINK.sender()).await {
        Ok(_) => println!("Max3 task ended okay."),
        Err(_) => println!("Max3 task failed"),
    };
}

#[embassy_executor::task]
pub async fn lsm6_multi_task(i2c: SharedI2cDevice, hz: u64, id: u8, n: u8) {
    ysns::yxz_lsm6::inner_multi_task(i2c, n, hz, id, false, SINK.sender()).await;
}

#[embassy_executor::task]
pub async fn lsm6_task(i2c: SharedI2cDevice, hz: u64, id: u8) {
    ysns::yxz_lsm6::task(i2c, hz, id, SINK.sender()).await;
}

#[embassy_executor::task]
pub async fn ads_task(i2c: SharedI2cDevice, hz: u64, id: u8) {
    ysns::ads1115::inner_task(i2c, hz, id, SINK.sender()).await;
}

#[embassy_executor::task]
pub async fn bmi160_task(i2c: SharedI2cDevice, hz: u64, id: u8) {
    ysns::yxz_bmi160::inner_task(i2c, hz, id, SINK.sender()).await;
}

#[embassy_executor::task]
pub async fn co2_task(i2c: SharedI2cDevice, hz: u64, id: u8) {
    match ysns::yco2::task(i2c, hz, id, SINK.sender()).await {
        Ok(_) => println!("Co2 task ended okay."),
        Err(e) => println!("Co2 task failed"),
    }
}

#[embassy_executor::task]
pub async fn display_task(i2c: SharedI2cDevice) {
    yll::yuio::disp::task(i2c).await;
}

/*#[macro_export]
macro_rules! init_usart {
    ($p: expr, $usart:ident, $tx:ident, $rx:ident, $dma_tx:ident, $dma_rx:ident, $config:expr, $baud:expr) => {
        config.baudrate = $baud;
        Uart::new(
            $p.$usart,
            $p.$tx,
            $p.$rx,
            Irqs,
            $p.$dma_tx,
            $p.$dma_rx,
            $config,
        )
    };
}

//pub(crate) use init_usart;

macro_rules! init_usart_default {
    () => {init_usart!(USART2, PA3, PA2, DMA1_CH6, DMA1_Ch5, 2_000_000)}
}

macro_rules! spawn_moi_task {
    ($pa:ident, $exti_pa:ident, $pb:ident, $exti_pb:ident, $pd0:ident, $exti_pd0:ident, $pd1:ident, $exti_pd1:ident, $val:expr, $sink:expr) => {
        {
            let pin_0 = ExtiInput::new(p.$pa, p.$exti_pa, ylab::Pull::Down);
            let pin_1 = ExtiInput::new(p.$pb, p.$exti_pb, ylab::Pull::Down);
            let pin_2 = ExtiInput::new(p.$pd0, p.$exti_pd0, ylab::Pull::Down);
            let pin_3 = ExtiInput::new(p.$pd1, p.$exti_pd1, ylab::Pull::Down);
            spawner.spawn(
                #[embassy_executor::task]
                async { moi::inner_task(pin_0, pin_1, pin_2, pin_3, $val, $sink).await; }
            ).unwrap();
        }
    };
}*/
