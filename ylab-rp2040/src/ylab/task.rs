use super::*;
use ylab_lib as yll;
use yll::ysns::{moi, yxz_bmi160, yxz_lsm6, ads1115, yco2};
use yll::yuii::btn;
use yll::yuio::led;
use mcu::gpio::{Input, Pull, Output};
use mcu::peripherals::{PIN_9, PIN_8, PIN_21, PIN_22, PIN_20};
use crate::ytfk::bsu::SINK;

#[embassy_executor::task]
pub async fn led_task(led: Output<'static>){
	led::task(led).await
}


#[embassy_executor::task]
pub async fn btn20_task(pin: Peri<'static, PIN_20>) {
    let pin = Input::new(pin, Pull::Up);
    btn::inner_task(pin).await;
}

#[embassy_executor::task]
pub async fn moi_task(
    pin_0: Peri<'static, PIN_21>,
    pin_1: Peri<'static, PIN_22>,
    pin_2: Peri<'static, PIN_8>,
    pin_3: Peri<'static, PIN_9>,
    id: u8)
    {
    let moi_0 = Input::new(pin_0, Pull::Up);
    let moi_1 = Input::new(pin_1, Pull::Up);
    let moi_2 = Input::new(pin_2, Pull::Up);
    let moi_3 = Input::new(pin_3, Pull::Up);
	moi::inner_task(moi_0, moi_1, moi_2, moi_3, 0, ytfk::bsu::SINK.sender()).await;
}

#[embassy_executor::task]
pub async fn lsm6_multi_task(i2c: SharedI2c1, hz: u64, id: u8, n: u8) {
	yxz_lsm6::inner_multi_task(i2c, n, hz, id, false, SINK.sender()).await;
}

#[embassy_executor::task]
pub async fn lsm6_task_0(i2c: SharedI2c0, hz: u64, id: u8) {
	yxz_lsm6::task(i2c, hz, id, SINK.sender()).await;
}

#[embassy_executor::task]
pub async fn ads_task_0(i2c: SharedI2c0, hz: u64, id: u8) {
	ads1115::inner_task(i2c, hz, id, SINK.sender()).await;
}

#[embassy_executor::task]
pub async fn bmi160_task(i2c: SharedI2c0, hz: u64, id: u8) {
	yxz_bmi160::inner_task(i2c, hz, id, SINK.sender()).await;
}

#[embassy_executor::task]
pub async fn ads_task_1(i2c: SharedI2c1,hz: u64, id: u8) {
	yll::ysns::ads1115::inner_task(i2c, hz, id, SINK.sender()).await;
}

#[embassy_executor::task]
pub async fn co2_task_0(i2c: SharedI2c0, id: u8) {
	yco2::task(i2c, 3, SINK.sender()).await;
}

#[embassy_executor::task]
pub async fn display_task_0(i2c: SharedI2c0) {
	yll::yuio::disp::task(i2c).await;
}
