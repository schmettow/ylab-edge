pub use super::{yll, mcu};
pub use yll::{Duration, Instant};
pub use yll::AtomicBool;
pub use yll::time::Ticker;
pub use crate::ytfk::bsu::SINK;
pub use yll::ysns::*;
pub use crate::println;

pub mod adc {
    /*use super::{yll, mcu, AtomicBool, Ticker, Duration, Instant, ORD, println};
    use embassy_sync::channel;
    use mcu::adc;
    pub use adc::AnyAdcChannel;
    use mcu::Peri;
    use crate::Vec;

    /// STM32
    //pub use super::{mcu, Channel, Mutex, Ordering};

    //use mcu::{Peri, peripherals::{ADC1, PA0, PA1, PA4, PB0, PC0, PC1, PC2, PC3}};
     //use mcu::peripherals::{ADC3, PF3, PF4, PF5, PF6, PF7, PF8, PF9, PF10};
    use mcu::adc::{Adc, SampleTime};
    ///
    pub const N: usize = 8;
    pub type Measure = u16;
    pub type Reading = [Measure; N];
    pub type Sample = yll::ydata::Sample<Measure, N>;


    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static SAMPLE: AtomicBool = AtomicBool::new(true);

    //type AdcPin: embedded_hal::adc::Channel<mcu::adc::Adc<'static>> + mcu::gpio::Pin;
    const DMA_BUF_LEN: usize = 120;*/


    /*pub async fn ring_task<I>(
        adc: Adc<'static, I>,
        pins: [Peri<'static, AnyAdcChannel<I>>; N],
        dma: Peri<'static, impl adc::RxDma<I>>,
        //dma_buf: &'static mut [u16],
        sensory: u8,)
    where
        I: adc::Instance + 'static,
    {
        let mut dma_buf = [0u16; DMA_BUF_LEN];
        let mut adc = adc.into_ring_buffered(
            dma,
            &mut dma_buf,
            /*[
                (&mut *pins[0], adc::SampleTime::CYCLES15),
                (&mut *pins[1], adc::SampleTime::CYCLES15),
            ].into_iter()*/
        );


        let mut measurements = [0u16; DMA_BUF_LEN / 2];
        loop {
            match adc.read(&mut measurements).await {
                Ok(_) => {
                    defmt::info!("adc1: {}", measurements);
                }
                Err(e) => {
                    defmt::warn!("Error: {:?}", e);
                }
            }
        }
                /*sample = Sample {
                    sensory: sensory,
                    time: Instant::now(),
                    read: reading,
                };
                crate::ysns::SINK.send(sample.into()).await;
                ticker.next().await;*/
        }*/




    // Task for ADC controller 1 with eight pins
    /*pub async fn inner_task<I>(
        mut adc: Adc<'static, I>,
        pins: [Peri<'static, AnyAdcChannel<I>>; N],
        //
        hz: u64,
        sensory: u8,
    ) where I: adc::Instance + 'static {
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        //let mut _vrefint = adc.enable_vrefint();
        let mut sample: Sample;
        adc.set_sample_time(SampleTime::CYCLES3);
        adc.set_resolution(mcu::adc::Resolution::BITS12);
        println!("ADC set");
        SAMPLE.store(true, ORD);
        if SAMPLE.load(ORD) {
            loop {
                let reading = pins.into_iter().map(|mut p| {
                    adc.blocking_read(&mut p.get_hw_channel())
                }).collect::<Vec<u16, N>>()
                  .into_array()
                  .unwrap();
                sample = Sample {
                    sensory: sensory,
                    time: Instant::now(),
                    read: reading,
                };
                crate::ysns::SINK.send(sample.into()).await;
                ticker.next().await;
            };
        }

    }*/


    /*#[embassy_executor::task]
    pub async fn task_1(
        // STM32
        mut adc: Adc<'static, ADC1>,
        mut pins: ( Peri<'static,PA0>, Peri<'static,PA1>,
                    Peri<'static,PA4>, Peri<'static,PB0>,
                    Peri<'static,PC1>, Peri<'static,PC0>,
                    Peri<'static,PC2>, Peri<'static,PC3>),
        //
        hz: u64,
        sensory: u8,
    ) {
        //println!("Starting ADC task");
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
                bsu::SINK.send(sample.into()).await;
            };
            ticker.next().await;
        }
    }*/
}
