//pub use ylab_lib::{Duration, Instant, RawMutex, Signal, Timer};
pub use super::*;
pub use crate::ybus::{SharedI2cDevice, SharedDeviceMutex};

#[derive(Debug, Clone)]
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

pub struct Sensor<T, const N: usize> {
    _sensor: T,
    pub id: u8,
}

pub mod yxz_lsm6 {

    use crate::ytfk::YtfSender;

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

    /*#[embassy_executor::task]
    pub async fn task_0(i2c_bus: &'static AsyncI2cBus<I2C0>, hz: u64, sensory: u8) {
        inner_task(i2c_bus, hz, sensory).await;
    }

    #[embassy_executor::task]
    pub async fn task_1(i2c_bus: &'static AsyncI2cBus<I2C1>, hz: u64, sensory: u8) {
        inner_task(i2c_bus, hz, sensory).await;
    }*/

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

    pub async fn task<M, B>(i2c: SharedI2cDevice<'_, M, B>, hz: u64, sensory: u8, sink: ytfk::YtfSender<'_>)
    where
    	M: SharedDeviceMutex,
     	B: AsyncI2c,

    //async fn inner_task(i2c: &'static SharedI2cDevice, hz: u64, sensory: u8)
    //where
    //    I: I2cInstance,
    {
        //let i2c = SharedI2cDevice::new(&i2c_bus);
        let mut sensor = Sensor::new(i2c, sensory, hz);
        sensor.init().await.unwrap();
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        sink.send(sensor.sample().await.unwrap().into()).await;
        log::debug!("Yxz read");
        ticker.next().await;
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
                sensor_active[s] = true;
            };
        }
        for (s, sensor) in sensors.as_mut().into_iter().enumerate() {
            if s >= n as usize {
                continue;
            }
            if let Ok(s) = sensor.sample().await {
                sink.send(s.into()).await;
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
