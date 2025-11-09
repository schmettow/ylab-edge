use super::*;
use ylab_lib as yll;
use yll::ysns::{moi, yxz_bmi160, yxz_lsm6, ads1115, yco2};
use yll::yuii::btn;
use yll::yuio::led;
use mcu::gpio::{Input, Pull, Output};
use crate::ytfk::bsu::SINK;

/*#[embassy_executor::task]
pub async fn led_task(led: Output<'static>){
	led::task(led).await
}


#[embassy_executor::task]
pub async fn btn20_task(pin: Peri<'static, PIN_20>) {
    let pin = Input::new(pin, Pull::Up);
    btn::inner_task(pin).await;
}*/

#[embassy_executor::task]
pub async fn moi_task(
    pin_0: Input<'static>,
    pin_1: Input<'static>,
    pin_2: Input<'static>,
    pin_3: Input<'static>)
    {
	moi::inner_task(pin_0, pin_1, pin_2, pin_3, 0, SINK.sender()).await;
}

/*#[embassy_executor::task]
pub async fn lsm6_multi_task(i2c: SharedI2c1, hz: u64, id: u8, n: u8) {
	yxz_lsm6::inner_multi_task(i2c, n, hz, id, false, SINK.sender()).await;
}*/

#[embassy_executor::task]
pub async fn lsm6_task(i2c: SharedI2c, hz: u64, id: u8) {
	yxz_lsm6::task(i2c, hz, id, SINK.sender()).await;
}

/*#[embassy_executor::task]
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
*/
