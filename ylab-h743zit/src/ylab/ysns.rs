pub use super::ytfk::bsu as ybsu;
pub use super::*;

pub mod adc {
    /// STM32
    pub use super::{mcu, Channel, Mutex, Ordering};
    pub use super::*;
    use mcu::peripherals::{ADC1, PA0, PA1, PA4, PB0, PC0, PC1, PC2, PC3};
    //use mcu::peripherals::{ADC3, PF3, PF4, PF5, PF6, PF7, PF8, PF9, PF10};
    use mcu::adc::{Adc, SampleTime};
    ///
    const N: usize = 8;
    pub type Measure = u16;
    pub type Reading = [Measure; N];
    pub type Sample = ydata::Sample<Measure, N>;

    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static SAMPLE: AtomicBool = AtomicBool::new(true);

    //type AdcPin: embedded_hal::adc::Channel<mcu::adc::Adc<'static>> + mcu::gpio::Pin;

    /// Task for ADC controller 1 with eight pins
    ///

    #[embassy_executor::task]
    pub async fn adcbank_1(
        // STM32
        mut adc: Adc<'static, ADC1>,
        mut pins: ( Peri<'static,PA0>, Peri<'static,PA1>, Peri<'static,PA4>, Peri<'static,PB0>,
                    Peri<'static,PC1>, Peri<'static,PC0>, Peri<'static,PC3>, Peri<'static,PC2>),
        //
        hz: u64,
        sensory: u8,
    ) {
        //println!("Starting ADC task");
        //let state: Atomic<super::State> = Atomic::new(State::Offline);
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        let mut _vrefint = adc.enable_vrefint();

        let mut sample: Sample;
        adc.set_sample_time(SampleTime::CYCLES3);
        adc.set_resolution(mcu::adc::Resolution::BITS12);
        //println!("ADC set");
        loop {
            if SAMPLE.load(ORD) {
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
