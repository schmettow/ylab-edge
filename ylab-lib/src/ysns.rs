//pub use ylab_lib::{Duration, Instant, RawMutex, Signal, Timer};
pub use super::*;
pub use crate::ybus::{SharedI2cDevice, SharedDeviceMutex};
use crate::ytfk::YtfSender;
pub use defmt::println;
pub use defmt::Format;

#[derive(Debug, Clone, Format)]
pub enum YsenseErr {
    Init,
    Read,
    Task,
}

/*pub trait Ysense<const N: usize> {
    type Measure;
    async fn init(&mut self) -> Result<(), YsenseErr>;
    async fn read(&self) -> Result<[Measure; N], YsenseErr>;
}*/

/// Generic sensor structure
///
/// with a sensor struct T (e.g. Ads1299, Lsm6dsox)
#[allow(dead_code)]
pub struct Sensor<T, const N: usize, R>
where R: core::fmt::Debug
{
    sensor: T,
    sample_rate: R,
    pub id: u8,
}


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


    pub async fn inner_task<M, BUS>(i2c: SharedI2cDevice<'static, M, BUS>, hz: u64, sensory: u8, sink: YtfSender<'static>)
    where
    	M: SharedDeviceMutex,
    	BUS: embedded_hal_async::i2c::I2c,
    {
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
                sink.send(sample.into()).await;
            };
        }
    }
}



pub mod yxz_bmi160 {
    use super::*;
    use bmi160::{AccelerometerPowerMode, Bmi160, GyroscopePowerMode, SensorSelector, SlaveAddr};
    //use embassy_rp::i2c::Instance;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(true);

    const N: usize = 6;
    pub type Measure = f32;
    pub type Reading = [Measure; N];
    pub type Sample = ydata::Sample<Measure, N>;

    pub async fn inner_task<M, BUS>(i2c: SharedI2cDevice<'static, M, BUS>, hz: u64, sensory: u8, sink: YtfSender<'static>)
    where
    	M: SharedDeviceMutex,
    	BUS: embedded_hal_async::i2c::I2c,
    {
        let address = SlaveAddr::default();
        let mut sensor = Bmi160::new_with_i2c(i2c, address);

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
                let data = sensor
                    .data(SensorSelector::new().accel().gyro())
                    .await
                    .unwrap();
                let acc = data.accel.unwrap();
                let gyr = data.gyro.unwrap();
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
                sink.send(sample.into()).await;
            };
        }
    }

}



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
    pub static RECORD: AtomicBool = AtomicBool::new(true);

    pub async fn inner_task<M, BUS>(i2c: SharedI2cDevice<'static, M, BUS>, hz: u64, sensory: u8, sink: YtfSender<'static>)
    where
    	M: SharedDeviceMutex,
    	BUS: embedded_hal_async::i2c::I2c,
    {
        let address = TargetAddr::default();
        let mut ads = Ads1x1x::new_ads1115(i2c, address).await;
        ads.set_data_rate(DataRate::Sps860).await.unwrap();
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        READY.store(true, ORD);
        println!("ADS set");
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
                sink.send(sample.into()).await;
            };
            ticker.next().await;
        }
    }
}




pub mod yco2 {
    use super::*;
    //use mcu::peripherals::I2C1 as ThisI2C;
    use scd4x;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static SAMPLE: AtomicBool = AtomicBool::new(true);

    // Generic result
    const N: usize = 3;
    pub type Measure = f32;
    pub type Reading = [Measure; N];
    pub type Sample = ydata::Sample<Measure, N>;

    //#[embassy_executor::task]

    pub async fn task<M, B>(i2c: SharedI2cDevice<'_, M, B>, sensory: u8, sink: ytfk::YtfSender<'_>)
    where
    	M: SharedDeviceMutex,
     	B: embedded_hal_async::i2c::I2c,
    {
        let mut sensor = scd4x::Scd4xAsync::new(i2c, time::Delay); // <-- this makes it sybc or async
        sensor.wake_up().await; //<---- This fails
        match sensor.stop_periodic_measurement().await {
            Ok(_) => {println!("Stopped periodic measurements")},
            Err(_) => {println!("Stopping periodic measurements failed.")}
        }

        match sensor.reinit().await {
            Ok(_) => READY.store(true, ORD),
            Err(_) => println!("SCD41 reinit failed.")
        }

        let serial = sensor.serial_number().await.unwrap();
        println!("CO2: serial {:?}", serial);
        sensor.start_periodic_measurement().await.unwrap();
        println!("CO2: Started periodic measurement");

        let mut ticker = Ticker::every(Duration::from_secs(5));
        let mut sample: Sample;
        println!("SCD41 all set");

        loop {
            if SAMPLE.load(ORD) {
                ticker.next().await;
                match sensor.measurement().await {
                    Err(_) => println!("CO2: read failed."),
                    Ok(raw) => {
                    	println!("CO2: read OK");
                        let reading: Reading =
                            [raw.co2 as f32, raw.humidity as f32, raw.temperature as f32];
                        sample = Sample {
                            sensory: sensory,
                            time: Instant::now(),
                            read: reading,};
                        sink.send(sample.into()).await;
                    }
                };
            };
        }
    }
}



pub mod sen_five {
    use super::*;
    const N: usize = 8;
    type Measure = f32;
    pub type Reading = [Measure; N];
    pub type Sample = ydata::Sample<Measure, N>;

    // control channels
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static SAMPLE: AtomicBool = AtomicBool::new(true);

    //use embedded_hal::delay::DelayNs;
    use embedded_hal_async::i2c::I2c;
    use sen5x::Sen5x;
    use async_sen5x as sen5x;

    pub struct Sensor<I>
    where
        I: I2c
    {
        sensor: Sen5x<I>,
        pub id: u8,
        pub interval: Duration,
    }

    impl<I> Sensor<I>
    where
        I: I2c,
        //D: DelayNs,
    {
        pub fn new(i2c: I, id: u8, interval: Duration) -> Self {
            Self {
                sensor: Sen5x::new(i2c),
                id: id,
                interval: interval,
            }
        }

        pub fn set_interval(&mut self, interval: Duration) {
            self.interval = interval;
        }

        pub fn set_hz(&mut self, hz: u32) {
            self.interval = Duration::from_hz(hz.into());
            todo!()
        }

        pub async fn init(&mut self) -> Result<(), ()> {
            match self.sensor.reinit().await {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            }
        }

        pub async fn read(&mut self) -> Result<Reading, ()> {
            let reading = self.sensor.measurement().await;
            match reading {
                Ok(r) => Ok([
                    r.humidity,
                    r.nox_index,
                    r.pm1_0,
                    r.pm2_5,
                    r.pm4_0,
                    r.pm10_0,
                    r.temperature,
                    r.voc_index,
                ]),
                _ => Err(()),
            }
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

    //use mcu::peripherals::I2C1 as ThisI2C;

    //#[embassy_executor::task]
    pub async fn task<M, B>(i2c: SharedI2cDevice<'_, M, B>, interval: Duration, sensory: u8, sink: ytfk::YtfSender<'_>)
    where
    	M: SharedDeviceMutex,
     	B: embedded_hal_async::i2c::I2c,
    {
        let mut sensor = Sensor::new(i2c, sensory, interval);
        match sensor.init().await {
            Err(_) => {
                println!("Sensor setup failed");
                return;
            } // connection error => end task
            Ok(_) => {}
        }

        let mut ticker = Ticker::every(interval);
        READY.store(true, ORD);
        println!("Sen5 ready");

        loop {
            if SAMPLE.load(ORD) {
                match sensor.sample().await {
                    Ok(sample) => {
                        sink.send(sample.into()).await;
                    }
                    Err(_) => {}
                }
            };
            ticker.next().await;
        }
    }
}




pub mod yds1299 {
    use super::*;
    // Sensor
    use ads1299::descriptors::*;
    pub use ads1299::descriptors::Command;
    use ads1299::Ads129x;
    //use ads1299::AdsData;
    pub use ads1299::SensorVersion;
    // SPI Bus
    use embedded_hal_async::spi::SpiDevice;
    use log::debug;
    // control channels and shared bus
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(true);
    //static SPI_BUS_1: StaticCell<SpiBusMutex1> = StaticCell::new();

    // measures
    const N: usize = 4;
    pub type Measure = f32;
    pub type Reading = [Measure; N];
    pub type Sample = ydata::Sample<Measure, N>;

    #[derive(Debug)]
    pub enum AdsError {
        Command(Command),
        Init,
        WakeUp,
        Config,
        Read,
    }

    pub struct Sensor<SPI>
    where
        SPI: SpiDevice,
    {
        pub sensor: Ads129x<SPI, N>,
        pub id: u8,
        pub hz: usize,
    }

    impl<SPI> Sensor<SPI>
    where
        SPI: SpiDevice,
    {
        pub fn new(spi: SPI, id: u8, hz: usize) -> Self {
            Self {
                sensor: Ads129x::new(spi, SensorVersion::Chan4),
                id: id,
                hz: hz,
            }
        }

        pub fn set_hz(&mut self, hz: usize) {
            self.hz = hz;
        }

        pub async fn init(&mut self) -> Result<(), AdsError> {
            let com = Command::WAKEUP;
            match self.sensor.write_command_async(com).await {
                Ok(_) => {}
                Err(e) => {
                    debug!("{:?}", e);
                    return Err(AdsError::Command(com));
                }
            };

            let com = Command::START;
            match self.sensor.write_command_async(com).await {
                Ok(_) => {}
                Err(e) => {
                    debug!("{:?}", e);
                    return Err(AdsError::Command(com));
                }
            };

            let com = Command::RDATAC;
            match self.sensor.write_command_async(com).await {
                Ok(_) => {}
                Err(e) => {
                    debug!("{:?}", e);
                    return Err(AdsError::Command(com));
                }
            };

            let config = ads1299::ConfigRegisters {
                config1: Config1::default(),
                config2: Config2::default(),
                config3: Config3::default(),
                config4: Config4::default(),
                loff: Loff::default(),
                ch1set: Ch1Set::default(),
                ch2set: Ch2Set::default(),
                ch3set: Ch3Set::default(),
                ch4set: Ch4Set::default(),
                ch5set: Ch5Set::default(),
                ch6set: Ch6Set::default(),
                ch7set: Ch7Set::default(),
                ch8set: Ch8Set::default(),
                gpio: Gpio::default(),
            };

            match self.sensor.apply_configuration_async(&config).await {
                Ok(_) => {
                    debug!("Applying configuration OK");
                }
                Err(e) => {
                    debug!("Applying configuration failed: {:?}", e);
                    return Err(AdsError::Config);
                }
            }

            if let Ok(r) = self.read().await {
                debug!("First read: {:?}", r);
                return Ok(());
            } else {
                debug!("Reading failed");
                return Err(AdsError::Read);
            }
        }

        pub async fn read(&mut self) -> Result<Reading, AdsError> {
            //let reading: Reading = [self.sensor.read_data_1ch_async().await];
            let reading = self.sensor.read().await;
            match reading {
                Ok(ads_data) => Ok(ads_data.voltage()),
                Err(_) => {
                    debug!("Ads1299 read failed");
                    Err(AdsError::Read)
                }
            }
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
}




pub mod yxz_lsm6 {

    use super::*;
    //use accelerometer::Accelerometer;
    use Lsm6dsox as Lsm6;
    use lsm6dsox::*;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(true);
    const N: usize = 6;
    pub type Measure = f32;
    pub type Reading = [Measure; N];
    pub type Sample = ydata::Sample<Measure, N>;

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
            self.sensor
                .set_gyro_scale(GyroscopeScale::Dps250)
                .await
                .unwrap();
            println!("LSM all set");
            Ok(())
        }

        pub async fn read(&mut self) -> Result<Reading, YsenseErr> {
            println!("LSM read");
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

    pub async fn task<M, B>(i2c: SharedI2cDevice<'_, M, B>, hz: u64, sensory: u8, sink: ytfk::YtfSender<'_>)
    where
    	M: SharedDeviceMutex,
     	B: AsyncI2c,
    {
        let mut sensor = Sensor::new(i2c, sensory, hz);
        match sensor.init().await {
        	Ok(_) => println!("Lsm6: init OK"),
         	Err(e) => println!("Lsm6: read failed {:?}", e)
        }
        loop {
	        let mut ticker = Ticker::every(Duration::from_hz(hz));
	        sink.send(sensor.sample().await.unwrap().into()).await;
	        ticker.next().await;
        }
    }

    // MULTI Task

    use xca9548a::{SlaveAddr, Xca9548a};
    pub async fn inner_multi_task<M,B>(
        i2c: SharedI2cDevice<'_,M,B>,
        n: u8,
        hz: u64,
        sensory: u8,
        _just_spin: bool,
        sink: YtfSender<'_>,
    ) where
        M: SharedDeviceMutex,
        B: AsyncI2c,
    {
        //let i2c_tca = SharedI2cDevice::new(&i2c_bus);
        let tca = Xca9548a::new(i2c, SlaveAddr::default());
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
            	println!("Sensor {} active", s);
                sensor_active[s] = true;
            };
        }
        loop {
	        for (s, sensor) in sensors.as_mut().into_iter().enumerate() {
	            if s >= n as usize {
	                continue;
	            }
	            if let Ok(sam) = sensor.sample().await {
	            	println!("Lsm6_{}: read OK", s);
	                sink.send(sam.into()).await;
	            }
	        }
        }
    }
}

pub mod moi {
    use super::*;
    use embedded_hal_async::digital::Wait;
    use embedded_hal::digital::InputPin;
    //use mcu::gpio::{Input, Pull};
    //use mcu::peripherals::{PIN_21, PIN_22};

    pub type Measure = bool;
    pub type Reading<const N: usize> = [Measure; N];
    pub type Sample<const N: usize> = ydata::Sample<Measure, N>;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(true);


    //#[embassy_executor::task]
    pub async fn inner_task<P: Wait>(
        mut moi_0: P,
        mut moi_1: P,
        mut moi_2: P,
        mut moi_3: P,
        sensory: u8,
        sink: ytfk::YtfSender<'_>,
    ) where
        P: Wait + InputPin,
        {
        /*let mut moi_0 = Input::new(moi_0, Pull::Up);
        let mut moi_1 = Input::new(moi_1, Pull::Up);
        let mut moi_2 = Input::new(moi_2, Pull::Up);
        let mut moi_3 = Input::new(moi_3, Pull::Up);*/

        use embassy_futures::select::select;
        loop {
            if RECORD.load(ORD) {
                select(
                    select(moi_0.wait_for_any_edge(), moi_1.wait_for_any_edge()),
                    select(moi_2.wait_for_any_edge(), moi_3.wait_for_any_edge()),
                )
                .await;
                let reading = [
                    moi_0.is_high().unwrap_or(false),
                    moi_1.is_high().unwrap_or(false),
                    moi_2.is_high().unwrap_or(false),
                    moi_3.is_high().unwrap_or(false),
                ];
                let sample = Sample {
                    sensory: sensory,
                    time: Instant::now(),
                    read: reading,
                };
                sink.send(sample.into()).await;
            };
        }
    }
}

pub mod btn {
    use super::*;
    use embedded_hal_async::digital::Wait;
    pub enum Event {
        Press,
        Short,
        Long,
    }
    pub static BTN: Signal<RawMutex, Event> = Signal::new();

    //#[embassy_executor::task]
    pub async fn inner_task<P: Wait>(mut btn: P) {
        //let mut btn = Input::new(btn_pin, Pull::Up);
        let longpress = 1000;
        let debounce = 50;

        loop {
            btn.wait_for_low().await.unwrap();
            BTN.signal(Event::Press);
            let when_pressed = Instant::now().as_millis();
            Timer::after(Duration::from_millis(debounce)).await;
            btn.wait_for_high().await.unwrap();
            if Instant::now().as_millis() - when_pressed >= longpress {
                BTN.signal(Event::Long);
            } else {
                BTN.signal(Event::Short);
            };
            Timer::after(Duration::from_millis(longpress)).await;
        }
    }
}
