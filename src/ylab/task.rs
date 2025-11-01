use super::*;
use ylab_lib as yll;
use yll::ysns::moi;
use yll::yuii::btn;
use yll::yuio::led;
use mcu::gpio::{Input, Pull, Output};
use mcu::peripherals::{PIN_9, PIN_8, PIN_21, PIN_22, PIN_20};

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
    pin_3: Peri<'static, PIN_9>)
    {
    let moi_0 = Input::new(pin_0, Pull::Up);
    let moi_1 = Input::new(pin_1, Pull::Up);
    let moi_2 = Input::new(pin_2, Pull::Up);
    let moi_3 = Input::new(pin_3, Pull::Up);
	moi::inner_task(moi_0, moi_1, moi_2, moi_3, 0, ytfk::bsu::SINK.sender()).await;
}
