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
    use mcu::peripherals::ADC1;
    use mcu::gpio::AnalogPin;
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

    //#[embassy_executor::task]
    pub async fn task<AP>(
        adc: ADC1<'static>,
        pin_0: AP,
        pin_1: AP,
        pin_2: AP,
        pin_3: AP,
        hz: u64,
        sensory: u8,
    ) where
    	AP: AnalogPin + AdcChannel,
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

            	/*let futures = [ch_0, ch_1, ch_2, ch_3]
	                .map(
	                   	async |ch| {
	                  		adc.read_oneshot(*ch).await
	                   	});
                let readings = embassy_futures::join::Join4::new(


                );
                 	.iter().map(
                  		|ch| async { adc.read_oneshot(*ch).await
						})).await;*/


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
}
