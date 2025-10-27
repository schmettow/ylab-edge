use crate::ytfk::bsu::SINK;
pub use crate::*;
pub use mcu::i2c::Instance as I2cInstance;
pub use ylab_lib::ybus::SharedI2cDevice;
pub use ylab_lib::ydata::Sample;
pub use yuio::disp::TEXT as DISP;


#[derive(Debug)]
pub enum YsenseErr {
    Init,
    Read,
    Task,
}


pub mod moi {
    use super::*;
    use mcu::gpio::{Input, Pull};
    use mcu::peripherals::{PIN_21, PIN_22};

    pub type Measure = bool;
    pub type Reading<const N: usize> = [Measure; N];
    pub type Sample<const N: usize> = ydata::Sample<Measure, N>;

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

/* ADS1115 Sensor I2C1 */
pub mod ads1115 {
    use super::*;
    use ads1x1x::{channel, DataRate16Bit as DataRate};
    use ads1x1x::{Ads1x1x, TargetAddr};

    // Data

    pub const N: usize = 4;
    pub type Measure = f32;
    pub type Reading = [Measure; N];
    pub type Sample = ydata::Sample<Measure, N>;
    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);

    #[embassy_executor::task]
    pub async fn task_0(i2c_bus: &'static SharedI2cBus<I2C0>, hz: u64, sensory: u8) {
        inner_task(i2c_bus, hz, sensory).await
    }

    #[embassy_executor::task]
    pub async fn task_1(i2c_bus: &'static SharedI2cBus<I2C1>, hz: u64, sensory: u8) {
        inner_task(i2c_bus, hz, sensory).await
    }

    async fn inner_task<I>(i2c_bus: &'static SharedI2cBus<I>, hz: u64, sensory: u8)
    where
        I: I2cInstance,
    {
        let address = TargetAddr::default();
        let i2c = SharedI2cDevice::new(&i2c_bus);
        let mut ads = Ads1x1x::new_ads1115(i2c, address).await;
        ads.set_data_rate(DataRate::Sps860).await.unwrap();
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        READY.store(true, ORD);
        loop {
            if RECORD.load(ORD) {
                let reading: Reading = [
                    ads.read(channel::SingleA0).await.unwrap().into(),
                    ads.read(channel::SingleA1).await.unwrap().into(),
                    ads.read(channel::SingleA2).await.unwrap().into(),
                    ads.read(channel::SingleA3).await.unwrap().into(),
                ];
                let sample = Sample {
                    sensory: sensory,
                    time: Instant::now(),
                    read: reading,
                };
                SINK.send(sample.into()).await;
                log::debug!("Yxz read");
            };
            ticker.next().await;
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
    pub type Sample = ydata::Sample<Measure, N>;

    #[embassy_executor::task]
    pub async fn task_0(i2c_bus: &'static SharedI2cBus<I2C0>, hz: u64, sensory: u8) {
        inner_task(i2c_bus, hz, sensory).await;
    }

    #[embassy_executor::task]
    pub async fn task_1(i2c_bus: &'static SharedI2cBus<I2C1>, hz: u64, sensory: u8) {
        inner_task(i2c_bus, hz, sensory).await;
    }

    pub struct Sensor<I2C>
    where
        I2C: embedded_hal_async::i2c::I2c,
    {
        pub sensor: Lsm6<I2C>,
        pub id: u8,
        pub hz: u64,
    }

    impl<I> Sensor<I>
    where
        I: embedded_hal_async::i2c::I2c,
    {
        pub fn new(i2c: I, id: u8, hz: u64) -> Self {
            Self {
                sensor: Lsm6::new(i2c, SlaveAddress::Low).unwrap(),
                id: id,
                hz: hz,
            }
        }

        pub async fn set_hz(&mut self, hz: u64) {
            self.hz = hz;
        }

        pub async fn init(&mut self) -> Result<(), YsenseErr> {
            self.sensor.setup(Delay).await.unwrap();
            self.sensor
                .set_accel_sample_rate(DataRate::Freq1660Hz)
                .await
                .unwrap();
            self.sensor
                .set_accel_scale(AccelerometerScale::Accel2g)
                .await
                .unwrap();
            self.sensor
                .set_gyro_sample_rate(DataRate::Freq1660Hz)
                .await
                .unwrap();
            self.sensor
                .set_gyro_scale(GyroscopeScale::Dps250)
                .await
                .unwrap();
            log::debug!("Yxz set");
            Ok(())
        }

        pub async fn read(&mut self) -> Result<Reading, YsenseErr> {
            log::debug!("Yxz get");
            let accel = self.sensor.accel_norm().await.unwrap();
            let gyro = self.sensor.angular_rate().await.unwrap();
            let reading = [
                accel.x.as_meters_per_second_per_second() as f32,
                accel.y.as_meters_per_second_per_second() as f32,
                accel.z.as_meters_per_second_per_second() as f32,
                gyro.x.as_hertz() as f32,
                gyro.y.as_hertz() as f32,
                gyro.z.as_hertz() as f32,
            ];
            Ok(reading)
        }

        pub async fn sample(&mut self) -> Result<Sample, ()> {
            let reading = self.read().await;
            match reading {
                Ok(reading) => Ok(Sample {
                    sensory: self.id,
                    time: Instant::now(),
                    read: reading,
                }),
                Err(_) => Err(()),
            }
        }
    }

    async fn inner_task<I>(i2c_bus: &'static SharedI2cBus<I>, hz: u64, sensory: u8)
    where
        I: I2cInstance,
    {
        let i2c = SharedI2cDevice::new(&i2c_bus);
        let mut sensor = Sensor::new(i2c, sensory, hz);
        sensor.init().await.unwrap();
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        SINK.send(sensor.sample().await.unwrap().into()).await;
        log::debug!("Yxz read");
        ticker.next().await;
    }

    /// Multi-task
    ///
    #[embassy_executor::task]
    pub async fn multi_task_0(
        i2c_bus: &'static SharedI2cBus<I2C0>,
        n: u8,
        hz: u64,
        just_spin: bool,
        sensory: u8,
    ) {
        inner_multi_task(i2c_bus, n, hz, sensory, just_spin).await
    }

    #[embassy_executor::task]
    pub async fn multi_task_1(
        i2c_bus: &'static SharedI2cBus<I2C1>,
        n: u8,
        hz: u64,
        just_spin: bool,
        sensory: u8,
    ) {
        inner_multi_task(i2c_bus, n, hz, sensory, just_spin).await
    }

    use xca9548a::{SlaveAddr, Xca9548a};
    async fn inner_multi_task<I>(
        i2c_bus: &'static SharedI2cBus<I>,
        n: u8,
        hz: u64,
        sensory: u8,
        _just_spin: bool,
    ) where
        I: I2cInstance,
    {
        let i2c_tca = SharedI2cDevice::new(&i2c_bus);
        let tca = Xca9548a::new(i2c_tca, SlaveAddr::default());
        let hub = tca.split();

        let sen_0 = Sensor::new(hub.i2c0, sensory, hz);
        let sen_1 = Sensor::new(hub.i2c1, sensory + 1, hz);
        let sen_2 = Sensor::new(hub.i2c2, sensory + 2, hz);
        let sen_3 = Sensor::new(hub.i2c3, sensory + 3, hz);
        let sen_4 = Sensor::new(hub.i2c4, sensory + 4, hz);
        let sen_5 = Sensor::new(hub.i2c5, sensory + 5, hz);
        //let sen_6 = Lsm6::new(hub.i2c6, SlaveAddress::Low, time::Delay);
        //let sen_7 = Lsm6::new(hub.i2c7, SlaveAddress::Low, time::Delay);
        let mut sensors = [sen_0, sen_1, sen_2, sen_3, sen_4, sen_5]; // sen_6, sen_7];
        let mut sensor_active = [false, false, false, false, false, false];
        for (s, sens) in sensors.as_mut().into_iter().enumerate() {
            if s >= n as usize {
                continue;
            }
            if let Ok(_) = sens.init().await {
                sensor_active[s] = true;
            };
        }
        for (s, sensor) in sensors.as_mut().into_iter().enumerate() {
            if s >= n as usize {
                continue;
            }
            if let Ok(s) = sensor.sample().await {
                SINK.send(s.into()).await;
            }
        }
    }
}
/// ## BMI Acceleration Sensor

pub mod yxz_bmi160 {
    use super::*;
    #[allow(unused)]
    use bmi160::{AccelerometerPowerMode, Bmi160, GyroscopePowerMode, SensorSelector, SlaveAddr};
    //use embassy_rp::i2c::Instance;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(true);

    const N: usize = 6;
    pub type Measure = f32;
    pub type Reading = [Measure; N];
    pub type Sample = ydata::Sample<Measure, N>;

    #[embassy_executor::task]
    pub async fn task_0(i2c_bus: &'static SharedI2cBus<I2C0>, hz: u64, sensory: u8) {
        inner_task::<I2C0>(&i2c_bus, hz, sensory).await
    }

    #[embassy_executor::task]
    pub async fn task_1(i2c_bus: &'static SharedI2cBus<I2C1>, hz: u64, sensory: u8) {
        inner_task::<I2C1>(&i2c_bus, hz, sensory).await
    }

    pub async fn inner_task<I>(i2c_bus: &'static SharedI2cBus<I>, hz: u64, sensory: u8)
    where
        I: I2cInstance,
    {
        //DISP.signal([None, None, None, Some("BMI160 task".try_into().unwrap())]);
        let i2c = SharedI2cDevice::new(&i2c_bus);
        let address = SlaveAddr::default();
        let mut sensor = Bmi160::new_with_i2c(i2c, address);
        //DISP.signal([None, Some("BMI160 |==| I2C".try_into().unwrap()), None, None]);
        sensor
            .set_accel_power_mode(AccelerometerPowerMode::Normal)
            .await
            .unwrap();
        sensor
            .set_gyro_power_mode(GyroscopePowerMode::Normal)
            .await
            .unwrap();
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        READY.store(true, ORD);
        loop {
            ticker.next().await;
            if RECORD.load(ORD) {
                DISP.signal([None, None, None, Some("BMI160 ...".try_into().unwrap())]);
                let data = sensor
                    .data(SensorSelector::new().accel().gyro())
                    .await
                    .unwrap();
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
    #[allow(unused)]
    use tlv493d as tlv;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(true);

    const N: usize = 4;
    pub type Measure = i16;
    pub type Reading = [Measure; N];
    pub type Sample = ydata::Sample<Measure, N>;

    #[embassy_executor::task]
    pub async fn task_0(i2c_bus: &'static SharedI2cBus<I2C0>, hz: u64, sensory: u8) {
        inner_task(i2c_bus, hz, sensory).await;
    }

    #[embassy_executor::task]
    pub async fn task_1(i2c_bus: &'static SharedI2cBus<I2C1>, hz: u64, sensory: u8) {
        inner_task(i2c_bus, hz, sensory).await;
    }

    async fn inner_task<I>(i2c_bus: &'static SharedI2cBus<I>, hz: u64, sensory: u8)
    where
        I: I2cInstance,
    {
        let i2c = SharedI2cDevice::new(&i2c_bus);
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

/*pub mod yirt_max {
    use super::*;
    use max3010x::*;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);
    pub type Reading = [u32; 8];
    pub type Measure = SensorResult<Reading>;

    #[embassy_executor::task]
    pub async fn task_0(i2c_bus: &'static SharedI2cBus<I2C0>, hz: u64, sensory: u8) {
        inner_task(i2c_bus, hz, sensory).await
    }

    async fn inner_task<I>(i2c_bus: &'static SharedI2cBus<I>, hz: u64, sensory: u8)
    where
        I: I2cInstance,
    {
        let i2c = SharedI2cDevice::new(&i2c_bus);
        let sensor = Max3010x::new_max30102(i2c);
        sensor.wake_up().await.unwrap();
        sensor.into_multi_led().await;
        sensor.set_pulse_amplitude(Led::All, 255).await.unwrap();
        sensor.set_sample_averaging(sample_averaging)
        sensor.set_led_time_slots([
            TimeSlot::Led1,
            TimeSlot::Led2,
            TimeSlot::Led1,
            TimeSlot::Disabled
        ]).await.unwrap();
        sensor.enable_fifo_rollover().await.unwrap();

        let mut data = [0; 2];
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        loop {
            let samples_read = sensor.read_fifo(&mut data).unwrap();
            ticker.next().await
        }
    }
}*/

/* pub mod yirt {
    // MLX90614
    /* Sensor Generics */
    use super::*;
    use embassy_time::{Duration, Instant, Ticker};
    use mlx9061x::{Mlx9061x, SlaveAddr};

    // Generic result
    pub type Reading = [f32; 2];
    pub type Measure = SensorResult<Reading>;

    // I2C
    use mcu::i2c;
    use mcu::peripherals::I2C0;

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
}*/

pub mod yco2 {
    use super::*;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);

    // Generic result
    const N: usize = 3;
    pub type Measure = f32;
    pub type Reading = [Measure; N];
    pub type Sample = ydata::Sample<Measure, N>;

    #[embassy_executor::task]
    pub async fn task_0(i2c_bus: &'static SharedI2cBus<I2C0>, sensory: u8) {
        inner_task(i2c_bus, sensory).await;
    }

    #[embassy_executor::task]
    pub async fn task_1(i2c_bus: &'static SharedI2cBus<I2C1>, sensory: u8) {
        inner_task(i2c_bus, sensory).await;
    }

    pub async fn inner_task<I>(i2c_bus: &'static SharedI2cBus<I>, sensory: u8)
    where
        I: I2cInstance,
    {
        //DISP.signal([None, None, None, Some("CO2 start".try_into().unwrap())]);
        let i2c = SharedI2cDevice::new(&i2c_bus);
        let mut sensor = scd4x::Scd4xAsync::new(i2c, time::Delay);
        //sensor.wake_up(); <---- This fails
        sensor.stop_periodic_measurement().await.unwrap();
        match sensor.reinit().await {
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
                match sensor.measure_single_shot_non_blocking().await {
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
                        match sensor.measurement().await {
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
