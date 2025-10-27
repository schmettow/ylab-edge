pub use super::*;

pub mod led {
    // LED control
    use super::*;
    use embassy_rp::gpio::{AnyPin, Level, Output};
    use ylab_lib::{Duration, Timer};
    //use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use ylab_lib::Signal;
    pub enum State {
        Vibrate,
        Blink,
        Steady,
        Interrupt,
        Off,
    }
    pub static LED: Signal<RawMutex, State> = Signal::new();

    #[embassy_executor::task]
    pub async fn task(led_pin: Peri<'static, AnyPin>) {
        let mut led = Output::new(led_pin, Level::Low);
        loop {
            let next_signal = LED.wait().await;
            match next_signal {
                State::Vibrate => {
                    for _ in 1..10 {
                        led.set_high();
                        Timer::after(Duration::from_millis(25)).await;
                        led.set_low();
                        Timer::after(Duration::from_millis(25)).await;
                    }
                }
                State::Blink => {
                    led.set_low();
                    Timer::after(Duration::from_millis(25)).await;
                    led.set_high();
                    Timer::after(Duration::from_millis(50)).await;
                    led.set_low()
                }
                State::Steady => led.set_high(),
                State::Off => led.set_low(),
                State::Interrupt => {
                    led.toggle();
                    Timer::after(Duration::from_millis(5)).await;
                    led.toggle();
                }
            }
        }
    }
}

pub mod disp {
    use super::*;
    use ylab_lib::ybus::SharedI2cDevice;
    //use mcu::i2c;
    //use mcu::peripherals::I2C1 as I2C;
    //use i2c::Async as Mode;
    use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306Async};
    // inter-thread communication

    pub type OneLine = String<20>;
    pub type FourLines = [Option<OneLine>; 4];

    pub static TEXT: Signal<RawMutex, FourLines> = Signal::new();

    #[embassy_executor::task]
    pub async fn task_0(i2c_bus: &'static SharedI2cBus<I2C0>) {
        inner_task(i2c_bus).await
    }

    #[embassy_executor::task]
    pub async fn task_1(i2c_bus: &'static SharedI2cBus<I2C1>) {
        inner_task(i2c_bus).await
    }

    // Text display
    //use core::fmt::Write;

    async fn inner_task<I>(i2c_bus: &'static SharedI2cBus<I>)
    where
        I: mcu::i2c::Instance,
    {
        let i2c = SharedI2cDevice::new(&i2c_bus);
        let interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_terminal_mode();
        //display.into_buffered_graphics_mode();
        match display.init().await {
            Err(_) => {}
            Ok(_) => {
                display.init().await.unwrap();
                let _ = display.write_str("Ydsp").await.unwrap();

                loop {
                    let mesg: FourLines = TEXT.wait().await;
                    let _ = display.clear();
                    //let mut str_conv = itoa::Buffer::new(); // conversion to string
                    for row in mesg {
                        match row {
                            Some(text) => {
                                let _ = display.write_str(text.as_str());
                            }
                            None => {
                                let _ = display.write_str("");
                            }
                        }
                    }
                }
            }
        }
    }
}
