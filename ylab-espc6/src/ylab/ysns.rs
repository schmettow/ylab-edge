pub use super::*;
use crate::ytfk::bsu::SINK;
use ylab_lib::ydata::Sample;

#[derive(Debug)]
pub enum YsenseErr {
    Init,
    Read,
    Task,
}

pub mod adc {
    use super::*;
    use mcu::analog::adc::*;
    use mcu::gpio::AnalogPin;
    use mcu::peripherals::ADC1;
    pub type Reading = [u16; 4];
    pub struct Result {
        pub time: Instant,
        pub reading: Reading,
    }
    /* result channel */
    pub static RESULT: Signal<RawMutex, Result> = Signal::new();

    /* control channels */

    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(true);

    pub async fn task<AP0, AP1, AP2, AP3>(
        adc: ADC1<'static>,
        pin_0: AP0,
        pin_1: AP1,
        pin_2: AP2,
        pin_3: AP3,
        hz: u64,
        sensory: u8,
    ) where
        AP0: AnalogPin + AdcChannel,
        AP1: AnalogPin + AdcChannel,
        AP2: AnalogPin + AdcChannel,
        AP3: AnalogPin + AdcChannel,
    {
        let mut config = AdcConfig::new();
        let atten = Attenuation::_11dB;
        let mut ch_0 = config.enable_pin(pin_0, atten);
        let mut ch_1 = config.enable_pin(pin_1, atten);
        let mut ch_2 = config.enable_pin(pin_2, atten);
        let mut ch_3 = config.enable_pin(pin_3, atten);
        let mut adc = Adc::new(adc, config).into_async();
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        READY.store(true, ORD);
        if READY.load(ORD) {
            loop {
                ticker.next().await;
                if RECORD.load(ORD) {
                    let readings: Reading = [
                        adc.read_oneshot(&mut ch_0).await,
                        adc.read_oneshot(&mut ch_1).await,
                        adc.read_oneshot(&mut ch_2).await,
                        adc.read_oneshot(&mut ch_3).await,
                    ];

                    let sample = Sample {
                        sensory: sensory,
                        time: Instant::now(),
                        read: readings,
                    };

                    SINK.send(sample.into()).await;
                };
            }
        }
    }

    #[embassy_executor::task]
    pub async fn task_gpio0_3(
        adc: ADC1<'static>,
        pin_0: mcu::peripherals::GPIO0<'static>,
        pin_1: mcu::peripherals::GPIO1<'static>,
        pin_2: mcu::peripherals::GPIO2<'static>,
        pin_3: mcu::peripherals::GPIO3<'static>,
        hz: u64,
        sensory: u8,
    ) {
        task(adc, pin_0, pin_1, pin_2, pin_3, hz, sensory).await;
    }
}
