use crate::ytfk::bsu::SINK;
pub use crate::*;
use hal::i2c;
use i2c::Async as Mode;
pub use yuio::disp::TEXT as DISP;

pub struct SensorResult<R> {
    pub time: Instant,
    pub reading: R,
}

pub mod moi {
    use super::*;
    use hal::gpio::{Input, Pull};
    use hal::peripherals::{PIN_21, PIN_22};

    pub type Measure = bool;
    pub type Reading<const N: usize> = [Measure; N];
    pub type Sample<const N: usize> = crate::Sample<Measure, N>;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(true);

    #[embassy_executor::task]
    pub async fn task(
        moi_0: Peri<'static, AnyPin>,
        moi_1: Peri<'static, AnyPin>,
        moi_2: Peri<'static, AnyPin>,
        moi_3: Peri<'static, AnyPin>,
        sensory: u8,
    ) {
        //pub async fn task(pins: [AnyPin; 4], trigger: [(bool, Option<bool>); 4], hz: u64, sensory: u8) {
        let mut moi_0 = Input::new(moi_0, Pull::Up);
        let mut moi_1 = Input::new(moi_1, Pull::Up);
        let mut moi_2 = Input::new(moi_2, Pull::Up);
        let mut moi_3 = Input::new(moi_3, Pull::Up);

        //let last_reading: Reading = [false, false, false, false,];
        //let mut reading: Reading<4>;
        use embassy_futures::select::select;
        loop {
            if RECORD.load(ORD) {
                select(
                    select(moi_0.wait_for_any_edge(), moi_1.wait_for_any_edge()),
                    select(moi_2.wait_for_any_edge(), moi_3.wait_for_any_edge()),
                )
                .await;
                //moi_3.wait_for_any_edge().await;
                let reading = [
                    moi_0.get_level().into(),
                    moi_1.get_level().into(),
                    moi_2.get_level().into(),
                    moi_3.get_level().into(),
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

    #[embassy_executor::task]
    pub async fn task_2(moi_0: Peri<'static, PIN_21>, moi_1: Peri<'static, PIN_22>, sensory: u8) {
        //pub async fn task(pins: [AnyPin; 4], trigger: [(bool, Option<bool>); 4], hz: u64, sensory: u8) {
        let mut sample: Sample<2>;
        let mut moi_0 = Input::new(moi_0, Pull::Up);
        let mut moi_1 = Input::new(moi_1, Pull::Up);

        let mut reading: Reading<2>;
        use embassy_futures::select::select;
        loop {
            if RECORD.load(ORD) {
                select(moi_0.wait_for_any_edge(), moi_1.wait_for_any_edge()).await;
                reading = [moi_0.get_level().into(), moi_1.get_level().into()];
                sample = Sample {
                    sensory: sensory,
                    time: Instant::now(),
                    read: reading,
                };
                SINK.send(sample.into()).await;
            };
        }
    }
}

pub mod adc {

    use super::*;
    use hal::adc::{Adc, Async, Channel};
    use hal::gpio::Pull;
    use hal::peripherals::{PIN_26, PIN_27, PIN_28};

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

    //type AdcPin: embedded_hal::adc::Channel<embassy_rp::adc::Adc<'static>> + embassy_rp::gpio::Pin;

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
        //let mut reading: Reading;
        //let mut result: SensorResult<Reading>;
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

/*
///# ADS1015 on I2C1
pub mod ads1015 {
    use super::*;
    /// ## Sensor Generics
    use embassy_time::{Duration, Ticker, Instant};

    /// ## I2C
    use embassy_rp::i2c::{self};
    ///
    /// Change this and Data Rate to switch I2C0/1
    use hal::peripherals::I2C1 as I2C;
    use ads1x1x::{channel, Ads1x1x, SlaveAddr};
    use ads1x1x::DataRate12Bit as DataRate;
    use nb::block;

    // ITC
    // Data
    pub struct SensorResult<R> {
        pub time: Instant,
        pub reading: R,
    }
    type Reading = [i16;4];
    type Measure = SensorResult<Reading>;
    pub static RESULT:Signal<Mutex, Measure> = Signal::new();

    /* control channels */
    pub use core::sync::atomic::Ordering;
    use core::sync::atomic::AtomicBool;
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);

    #[embassy_executor::task]
    pub async fn task(i2c: i2c::I2c<'static, I2C, i2c::Async>,
                      hz: u64) {
        let address = SlaveAddr::default();
        let mut ads
                = Ads1x1x::new_ads1015(i2c, address);
        // ads.set_data_rate(DataRate16Bit::Sps860).unwrap();
        ads.set_data_rate(DataRate::Sps3300).unwrap();
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        let mut reading: Reading;
        let mut result: SensorResult<Reading>;
        READY.store(true, ORD);
        loop {
            ticker.next().await;
            if RECORD.load(ORD){
                reading = [0; 4
                    /*block!(ads.read(&mut channel::SingleA0)).unwrap(),
                    block!(ads.read(&mut channel::SingleA1)).unwrap(),
                    block!(ads.read(&mut channel::SingleA2)).unwrap(),
                    block!(ads.read(&mut channel::SingleA3)).unwrap(),*/
                    ];
                result = SensorResult{time: Instant::now(),
                                      reading: reading};
                log::info!("{},2,{},{},{},{},,,,",
                    result.time.as_micros(),
                    result.reading[0],
                    result.reading[1],
                    result.reading[2],
                    result.reading[3],);
                    };
                }
            }
    }


/* ADS1115 Sensor I2C1 */
pub mod ads1115 {
    use super::*;
    /* Sensor Generics */
    use embassy_time::{Duration, Ticker, Instant};

    // I2C
    use hal::i2c::{self};
    use hal::peripherals::I2C1 as I2C;
    use ads1x1x::{channel, Ads1x1x, SlaveAddr};
    // ads1115 takes 16 bit
    use ads1x1x::DataRate16Bit as DataRate; // <-----
    use nb::block;

    // Data
    pub struct SensorResult<R> {
        pub time: Instant,
        pub reading: R,
    }
    type Reading = [i16;4];
    type Measure = SensorResult<Reading>;
    pub static RESULT:Signal<Mutex, Measure> = Signal::new();

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);

    #[embassy_executor::task]
    pub async fn task(i2c: i2c::I2c<'static, I2C, i2c::Async>,
                      hz: u64) {
        let address = SlaveAddr::default();
        let mut ads
                = Ads1x1x::new_ads1115(i2c, address);
        ads.set_data_rate(DataRate::Sps860).unwrap();
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        let mut reading: Reading;
        let mut result: SensorResult<Reading>;
        READY.store(true, ORD);
        loop {
            ticker.next().await;
            if RECORD.load(ORD){
                reading = [0; 4];
                    /*block!(ads.read(&mut channel::SingleA0)).unwrap(),
                    block!(ads.read(&mut channel::SingleA1)).unwrap(),
                    block!(ads.read(&mut channel::SingleA2)).unwrap(),
                    block!(ads.read(&mut channel::SingleA3)).unwrap(),];*/
                result = SensorResult{time: Instant::now(),
                                      reading: reading};
                log::info!("{},2,{},{},{},{},,,,",
                    result.time.as_micros(),
                    result.reading[0],
                    result.reading[1],
                    result.reading[2],
                    result.reading[3],);
                    };
                }
            }
    }

*/

pub mod yxz_lsm6_old {
    use super::*;
    use hal::peripherals::I2C0 as I2C;
    use i2c::Blocking as Mode;
    use lsm6ds33::Lsm6ds33 as Lsm6;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);

    // Generic result
    /*pub struct SensorResult<R> {
        pub time: Instant,
        pub reading: R,
    }*/
    pub type Reading = [f32; 3];
    /// <--- 4 channel is total accel for now
    pub type Measure = SensorResult<Reading>;

    #[embassy_executor::task]
    pub async fn task(i2c: i2c::I2c<'static, I2C, Mode>, hz: u64, sensory: u8) {
        DISP.signal([None, None, None, Some("Lsm6 task".try_into().unwrap())]);
        let sensor_res: Result<
            Lsm6<i2c::I2c<'_, I2C, Mode>>,
            (i2c::I2c<'_, I2C, Mode>, lsm6ds33::Error<i2c::Error>),
        > = Lsm6::new(i2c, 0x6Au8);
        let mut sensor = match sensor_res {
            Result::Ok(sensor) => sensor,
            Result::Err(_) => {
                DISP.signal([None, None, None, Some("Lsm6 =/= I2C".try_into().unwrap())]);
                panic!()
            }
        };
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        let mut reading: Reading;
        let mut result: SensorResult<Reading>;
        READY.store(true, ORD);
        DISP.signal([None, None, None, Some("Lsm6 ticking".try_into().unwrap())]);
        loop {
            //DISP.signal([None, None, None, Some("Lsm6 reading".try_into().unwrap())]);
            ticker.next().await;
            if RECORD.load(ORD) {
                reading = sensor.read_accelerometer().unwrap().into();
                result = Measure {
                    time: Instant::now(),
                    reading: reading,
                };
                log::info!(
                    "{},{},{},{},{},,,,",
                    sensory,
                    result.time.as_micros(),
                    result.reading[0],
                    result.reading[1],
                    result.reading[2],
                );
            };
        }
    }
}

pub mod yxz_lsm6 {

    use super::*;
    //use accelerometer::Accelerometer;
    use lsm6dsox::*;
    use Lsm6dsox as Lsm6;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(true);
    const N: usize = 6;
    pub type Measure = f32;
    pub type Reading = [Measure; N];
    pub type Sample = crate::Sample<Measure, N>;

    async fn inner_task<I>(i2c_bus: &'static AsyncI2cBus<I>, hz: u64, sensory: u8)
    where
        I: hal::i2c::Instance,
    {
        let i2c = AsyncI2cDevice::new(&i2c_bus);
        let mut sensor = Lsm6::new(i2c, SlaveAddress::Low).unwrap();
        sensor.setup(Delay).await.unwrap();
        sensor
            .set_accel_sample_rate(DataRate::Freq1660Hz)
            .await
            .unwrap();
        sensor
            .set_accel_scale(AccelerometerScale::Accel2g)
            .await
            .unwrap();
        sensor
            .set_gyro_sample_rate(DataRate::Freq1660Hz)
            .await
            .unwrap();
        sensor.set_gyro_scale(GyroscopeScale::Dps250).await.unwrap();
        log::debug!("Yxz set");
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        //let mut reading: Reading;
        //let mut result: SensorResult<Reading>;
        READY.store(true, ORD);

        loop {
            if RECORD.load(ORD) {
                log::debug!("Yxz get");
                let accel = sensor.accel_norm().await.unwrap();
                let gyro = sensor.angular_rate().await.unwrap();
                let reading = [
                    accel.x.as_meters_per_second_per_second() as f32,
                    accel.y.as_meters_per_second_per_second() as f32,
                    accel.z.as_meters_per_second_per_second() as f32,
                    gyro.x.as_hertz() as f32,
                    gyro.y.as_hertz() as f32,
                    gyro.z.as_hertz() as f32,
                ];

                let sample = Sample {
                    sensory: sensory,
                    time: Instant::now(),
                    read: reading,
                };
                SINK.send(sample.into()).await;
                log::debug!("Yxz read");
                ticker.next().await;
            };
        }
        //let mut sensor = Lsm6::new(i2c, SlaveAddress::Low, time::Delay);
    }

    #[embassy_executor::task]
    pub async fn task_0(i2c_bus: &'static AsyncI2cBus<I2C0>, hz: u64, sensory: u8) {
        inner_task(i2c_bus, hz, sensory).await;
    }

    /// Multi-task
    use xca9548a::{SlaveAddr, Xca9548a};
    async fn inner_multi_task<I>(
        i2c_bus: &'static AsyncI2cBus<I>,
        n: u8,
        hz: u64,
        sensory: u8,
        just_spin: bool,
    ) where
        I: hal::i2c::Instance,
    {
        let i2c_tca = AsyncI2cDevice::new(&i2c_bus);
        let tca = Xca9548a::new(i2c_tca, SlaveAddr::default());
        let hub = tca.split();

        let sen_0 = Lsm6::new(hub.i2c0, SlaveAddress::Low).unwrap();
        let sen_1 = Lsm6::new(hub.i2c1, SlaveAddress::Low).unwrap();
        let sen_2 = Lsm6::new(hub.i2c2, SlaveAddress::Low).unwrap();
        let sen_3 = Lsm6::new(hub.i2c3, SlaveAddress::Low).unwrap();
        let sen_4 = Lsm6::new(hub.i2c4, SlaveAddress::Low).unwrap();
        let sen_5 = Lsm6::new(hub.i2c5, SlaveAddress::Low).unwrap();
        //let sen_6 = Lsm6::new(hub.i2c6, SlaveAddress::Low, time::Delay);
        //let sen_7 = Lsm6::new(hub.i2c7, SlaveAddress::Low, time::Delay);
        let mut sensors = [sen_0, sen_1, sen_2, sen_3, sen_4, sen_5]; // sen_6, sen_7];
                                                                      //let mut sensory = [Some(sen_0), Some(sen_1), Some(sen_2), Some(sen_3), Some(sen_4), Some(sen_5), Some(sen_6), Some(sen_7)];
        let data_rate = DataRate::Freq416Hz;
        let mut sensor_active = [false, false, false, false, false, false];
        for (s, sens) in sensors.as_mut().into_iter().enumerate() {
            if s >= n as usize {
                continue;
            }
            if let (Ok(_), Ok(_)) = (
                sens.set_accel_sample_rate(data_rate).await,
                sens.set_gyro_sample_rate(data_rate).await,
            ) {
                sensor_active[s] = true;
            };
        }
        //DISP.signal([None, None, None, Some("LSM6x3".try_into().unwrap())]);
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        //let mut reading: Reading;
        //let mut result: Sample;
        READY.store(true, ORD);
        loop {
            if RECORD.load(ORD) {
                for (s, sensor) in sensors.as_mut().into_iter().enumerate() {
                    if s >= n as usize {
                        continue;
                    }
                    if let (Ok(accel), Ok(gyro)) =
                        (sensor.accel_norm().await, sensor.angular_rate().await)
                    {
                        let reading = [
                            accel.x.as_meters_per_second_per_second() as f32,
                            accel.y.as_meters_per_second_per_second() as f32,
                            accel.z.as_meters_per_second_per_second() as f32,
                            gyro.x.as_hertz() as f32,
                            gyro.y.as_hertz() as f32,
                            gyro.z.as_hertz() as f32,
                        ];
                        let sample = Sample {
                            sensory: (s as u8 + sensory),
                            time: Instant::now(),
                            read: reading,
                        };
                        SINK.send(sample.into()).await;
                    }
                }
            };
            if !just_spin {
                ticker.next().await;
            };
        }
    }

    #[embassy_executor::task]
    pub async fn multi_task_0(
        i2c_bus: &'static AsyncI2cBus<I2C0>,
        n: u8,
        hz: u64,
        just_spin: bool,
        sensory: u8,
    ) {
        inner_multi_task(i2c_bus, n, hz, sensory, just_spin).await
    }
}
/// ## BMI Acceleration Sensor

pub mod yxz_bmi160 {
    use super::*;
    #[allow(unused)]
    use bmi160::{AccelerometerPowerMode, Bmi160, GyroscopePowerMode, SensorSelector, SlaveAddr};

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(true);

    const N: usize = 6;
    pub type Measure = f32;
    pub type Reading = [Measure; N];
    /// <--- 4 channel is total accel for now
    pub type Sample = crate::Sample<Measure, N>;

    #[embassy_executor::task]
    pub async fn task(i2c: i2c::I2c<'static, I2C1, Mode>, hz: u64, sensory: u8) {
        //DISP.signal([None, None, None, Some("BMI160 task".try_into().unwrap())]);
        let address = SlaveAddr::default();
        let mut sensor = Bmi160::new_with_i2c(i2c, address);
        //DISP.signal([None, Some("BMI160 |==| I2C".try_into().unwrap()), None, None]);
        sensor
            .set_accel_power_mode(AccelerometerPowerMode::Normal)
            .unwrap();
        //DISP.signal([None, Some("BMI160 accel".try_into().unwrap()), None, None]);
        sensor
            .set_gyro_power_mode(GyroscopePowerMode::Normal)
            .unwrap();
        //DISP.signal([None, Some("BMI160 gyro".try_into().unwrap()), None, None]);
        //DISP.signal([None, None, None, Some("BMI160 set".try_into().unwrap())]);
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        //DISP.signal([None, None, None, Some("BMI160 ticks".try_into().unwrap())]);
        //let mut reading: Reading;
        //let mut result: Sample;
        READY.store(true, ORD);
        loop {
            ticker.next().await;
            if RECORD.load(ORD) {
                DISP.signal([None, None, None, Some("BMI160 ...".try_into().unwrap())]);
                let data = sensor.data(SensorSelector::new().accel().gyro()).unwrap();
                let acc = data.accel.unwrap();
                let gyr = data.gyro.unwrap();
                DISP.signal([None, None, None, Some("BMI160     ...".try_into().unwrap())]);
                let reading = [
                    acc.x.into(),
                    acc.y.into(),
                    acc.z.into(),
                    gyr.x.into(),
                    gyr.y.into(),
                    gyr.z.into(),
                ];
                let sample = Sample {
                    time: Instant::now(),
                    sensory: sensory,
                    read: reading.into(),
                };
                SINK.send(sample.into()).await;
            };
        }
    }
}

/// ## TLV Hall effect

pub mod yxz_tlv {
    use super::*;
    //use hal::peripherals::I2C0 as I2C;
    //use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
    #[allow(unused)]
    use tlv493d as tlv;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(true);

    const N: usize = 4;
    pub type Measure = i16;
    pub type Reading = [Measure; N];
    pub type Sample = crate::Sample<Measure, N>;

    #[embassy_executor::task]
    pub async fn task_0(i2c_bus: &'static AsyncI2cBus<I2C0>, hz: u64, sensory: u8) {
        inner_task(i2c_bus, hz, sensory).await;
    }

    async fn inner_task<I>(i2c_bus: &'static AsyncI2cBus<I>, hz: u64, sensory: u8)
    where
        I: hal::i2c::Instance,
    {
        let i2c = AsyncI2cDevice::new(&i2c_bus);
        //DISP.signal([None, None, None, Some("LVT task".try_into().unwrap())]);
        let address = 0x5E;
        let mut sensor = tlv::Tlv493d::new_async(i2c, address, tlv::Mode::Master)
            .await
            .unwrap();
        //DISP.signal([None, Some("BMI160 |==| I2C".try_into().unwrap()), None, None]);
        let _: Reading = sensor.read_raw_async().await.unwrap();
        sensor.configure(tlv::Mode::Fast, true).await.unwrap();
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        READY.store(true, ORD);
        loop {
            ticker.next().await;
            if RECORD.load(ORD) {
                let reading: Reading = sensor.read_raw_async().await.unwrap();
                let sample = Sample {
                    time: Instant::now(),
                    sensory: sensory,
                    read: reading.into(),
                };
                SINK.send(sample.into()).await;
            };
        }
    }
}

pub mod yirt_max {
    use super::*;
    use max3010x::{Led, Max3010x, SampleAveraging};

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);
    pub type Reading = [u32; 8];
    pub type Measure = SensorResult<Reading>;

    /*use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
    async fn inner_task<I>(i2c_bus: &'static AsyncI2cBus<I>, hz: u64, sensory: u8)
    where
        I: embassy_rp::i2c::Instance,
    {
        let i2c = I2cDevice::new(&i2c_bus);
        let mut sensor = Max3010x::new_max30102(i2c).into_multi_led().unwrap();
    }*/

    #[embassy_executor::task]
    pub async fn task(i2c: i2c::I2c<'static, I2C0, Mode>, hz: u64, sensory: u8) {
        // Sensor specific
        let mut sensor = Max3010x::new_max30102(i2c).into_multi_led().unwrap();
        sensor
            .set_sampling_rate(max3010x::SamplingRate::Sps3200)
            .unwrap();
        sensor.set_sample_averaging(SampleAveraging::Sa16).unwrap();
        sensor.set_pulse_amplitude(Led::All, 15).unwrap();
        sensor.enable_fifo_rollover().unwrap();
        sensor.wake_up().unwrap();

        let mut data: [u32; 1] = [0; 1];
        let _ = sensor.read_fifo(&mut data).unwrap();
        DISP.signal([
            None,
            None,
            None,
            Some("IRTmax can read".try_into().unwrap()),
        ]);
        // Ticker
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        //let mut result: SensorResult<Reading>;
        READY.store(true, ORD);
        loop {
            if RECORD.load(ORD) {
                let mut reading = [0; 1];
                let _ = sensor.read_fifo(&mut reading);

                let sample = Sample {
                    sensory: sensory,
                    time: Instant::now(),
                    read: reading,
                };
                SINK.send(sample.into()).await;
                /*log::info!("{},1,{},,,,,,,",
                Instant::now().as_micros(),
                reading[0]);*/
                ticker.next().await;
            };
        }
    }
}

pub mod yirt {
    // MLX90614
    /* Sensor Generics */
    use super::*;
    use embassy_time::{Duration, Instant, Ticker};
    use mlx9061x::{Mlx9061x, SlaveAddr};

    // Generic result
    pub type Reading = [f32; 2];
    pub type Measure = SensorResult<Reading>;

    // I2C
    use hal::i2c;
    use hal::peripherals::I2C0;

    /* control channels */
    use core::sync::atomic::AtomicBool;
    pub use core::sync::atomic::Ordering;
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);

    #[embassy_executor::task]
    pub async fn task(i2c: i2c::I2c<'static, I2C0, i2c::Blocking>, hz: u64, sensory: u8) {
        let address = SlaveAddr::default();
        let mut sensor = Mlx9061x::new_mlx90614(i2c, address, 5).unwrap();
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        //let mut reading: Reading;
        //let mut result: SensorResult<Reading>;
        READY.store(true, ORD);
        loop {
            ticker.next().await;
            if RECORD.load(ORD) {
                let obj_temp: f32 = sensor.object1_temperature().unwrap().into();
                let amb_temp: f32 = sensor.ambient_temperature().unwrap().into();
                let reading: Reading = [obj_temp.into(), amb_temp.into()];
                let sample = Sample {
                    sensory: sensory,
                    time: Instant::now(),
                    read: reading,
                };
                SINK.send(sample.into()).await;
                /*log::info!("{},T,{},{}",
                result.time.as_micros(),
                result.reading[0],
                result.reading[1]);*/
            };
        }
    }
}

pub mod yco2 {
    use super::*;
    use hal::peripherals::I2C0;
    use scd4x;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);

    // Generic result
    pub type Reading = [f32; 3];
    pub type Measure = SensorResult<Reading>;

    #[embassy_executor::task]
    pub async fn task(i2c: i2c::I2c<'static, I2C0, Mode>, sensory: u8) {
        //DISP.signal([None, None, None, Some("CO2 start".try_into().unwrap())]);
        let mut sensor = scd4x::Scd4x::new(i2c, time::Delay);
        //sensor.wake_up(); <---- This fails
        sensor.stop_periodic_measurement().unwrap();
        match sensor.reinit() {
            Ok(_) => {}
            Err(_) => {
                DISP.signal([None, None, None, Some("Reinit failed".try_into().unwrap())]);
                return;
            }
        }
        //DISP.signal([None, None, None, Some("CO2 init".try_into().unwrap())]);
        let mut ticker = Ticker::every(Duration::from_secs(5));
        //let mut result: SensorResult<Reading>;
        READY.store(true, ORD);
        //DISP.signal([None, None, None, Some("CO2 ticking".try_into().unwrap())]);
        loop {
            if RECORD.load(ORD) {
                match sensor.measure_single_shot_non_blocking() {
                    Err(_) => {
                        DISP.signal([
                            None,
                            None,
                            None,
                            Some("CO2 prep failed".try_into().unwrap()),
                        ]);
                    }
                    Ok(_) => {
                        ticker.next().await;
                        match sensor.measurement() {
                            Err(_) => {
                                DISP.signal([
                                    None,
                                    None,
                                    None,
                                    Some("CO2 read failed".try_into().unwrap()),
                                ]);
                            }
                            Ok(raw) => {
                                let reading: Reading =
                                    [raw.co2 as f32, raw.humidity as f32, raw.temperature as f32];
                                let sample = Sample {
                                    sensory: sensory,
                                    time: Instant::now(),
                                    read: reading,
                                };
                                SINK.send(sample.into()).await;
                            }
                        };
                    }
                };
            };
        }
    }
}
