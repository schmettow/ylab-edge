use crate::ytfk::bsu::SINK;
pub use crate::*;
pub use mcu::i2c::Instance as I2cInstance;
pub use ylab_lib::ybus::SharedI2cDevice;
pub use ylab_lib::ydata::Sample;
pub use yll::yuio::disp::TEXT as DISP;


#[derive(Debug)]
pub enum YsenseErr {
    Init,
    Read,
    Task,
}


pub mod adc {

    use super::*;
    use mcu::adc::{Adc, Async, Channel};
    use mcu::gpio::Pull;
    use mcu::peripherals::{PIN_26, PIN_27, PIN_28};

    pub type Reading = [u16; 3];
    pub struct Result {
        pub time: Instant,
        pub reading: Reading,
    }
    /* result channel */
    pub static RESULT: Signal<RawMutex, Result> = Signal::new();

    /* control channels */

    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);

    //type AdcPin: embedded_mcu::adc::Channel<embassy_rp::adc::Adc<'static>> + embassy_rp::gpio::Pin;

    #[embassy_executor::task]
    pub async fn task(
        mut adc: Adc<'static, Async>,
        adc_0: Peri<'static, PIN_26>,
        adc_1: Peri<'static, PIN_27>,
        adc_2: Peri<'static, PIN_28>,
        hz: u64,
        sensory: u8,
    ) {
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        let mut chan = [
            Channel::new_pin(adc_0, Pull::None),
            Channel::new_pin(adc_1, Pull::None),
            Channel::new_pin(adc_2, Pull::None),
        ];

        loop {
            ticker.next().await;
            if RECORD.load(ORD) {
                let reading = [
                    adc.read(&mut chan[0]).await.unwrap(),
                    adc.read(&mut chan[1]).await.unwrap(),
                    adc.read(&mut chan[2]).await.unwrap(),
                ];

                let sample = Sample {
                    sensory: sensory,
                    time: Instant::now(),
                    read: reading,
                };

                SINK.send(sample.into()).await;
            };
        }
    }
}
