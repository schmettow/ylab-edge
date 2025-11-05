pub use super::ytfk::bsu as ybsu;
pub use super::*;

pub mod adc {
    pub use super::*;
    /// STM32
    pub use super::{Channel, Mutex, Ordering, mcu};
    use mcu::adc::Adc;
    ///
    const N: usize = 8;
    pub type Measure = u16;
    pub type Reading = [Measure; N];
    pub type Sample = ydata::Sample<Measure, N>;

    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static SAMPLE: AtomicBool = AtomicBool::new(true);

    //use mcu::adc::AnyAdcChannel;
    use mcu::peripherals::{ADC1, PA0, PA1, PA2, PA3, PA4, PA7, PB1, PC0};

    #[embassy_executor::task]
    pub async fn adcbank_1(
        mut adc: Adc<'static, ADC1>,
        mut pins: (
            Peri<'static, PA0>,
            Peri<'static, PA1>,
            Peri<'static, PA2>,
            Peri<'static, PA3>,
            Peri<'static, PA4>,
            Peri<'static, PC0>,
            Peri<'static, PA7>,
            Peri<'static, PB1>,
        ),
        //mut pins: [AnyAdcChannel<_>; 8],
        hz: u64,
        sensory: u8,
    ) {
        println!("Starting ADC task");
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        let mut _vrefint = adc.enable_vrefint();

        let mut sample: Sample;
        //adc.set_sample_time(SampleTime::CYCLES3);
        adc.set_resolution(mcu::adc::Resolution::BITS12);
        //println!("ADC set");
        loop {
            if SAMPLE.load(ORD) {
                /*let reading =
                pins.iter().map(|p| adc.blocking_read(&mut p)).collect();*/

                /*for pin in pins {
                    adc.blocking_read(&mut pin);
                }*/
                let reading = [
                    adc.blocking_read(&mut pins.0),
                    adc.blocking_read(&mut pins.1),
                    adc.blocking_read(&mut pins.2),
                    adc.blocking_read(&mut pins.3),
                    adc.blocking_read(&mut pins.4),
                    adc.blocking_read(&mut pins.5),
                    adc.blocking_read(&mut pins.6),
                    adc.blocking_read(&mut pins.7),
                ];
                sample = Sample {
                    sensory: sensory,
                    time: Instant::now(),
                    read: reading,
                };
                ybsu::SINK.send(sample.into()).await;
            };
            ticker.next().await;
        }
    }
}
