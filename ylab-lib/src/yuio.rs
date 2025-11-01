pub use super::*;
pub mod led {
    // LED control
    use super::*;
    use embedded_hal::digital::{OutputPin, StatefulOutputPin};
    //use ylab_lib::{Duration, Timer};
    //use ylab_lib::Signal;
    pub enum State {
        Vibrate,
        Blink,
        Steady,
        Interrupt,
        Off,
    }
    pub static LED: Signal<RawMutex, State> = Signal::new();

    //#[embassy_executor::task]
    pub async fn task<T>(mut led: T)
    where
    	T: OutputPin + StatefulOutputPin,
    {
        //let mut led = Output::new(led_pin, Level::Low); <-- to main()
        loop {
            let next_signal = LED.wait().await;
            match next_signal {
                State::Vibrate => {
                    for _ in 1..10 {
                        led.set_high().unwrap();
                        Timer::after(Duration::from_millis(25)).await;
                        led.set_low().unwrap();
                        Timer::after(Duration::from_millis(25)).await;
                    }
                }
                State::Blink => {
                    led.set_low().unwrap();
                    Timer::after(Duration::from_millis(25)).await;
                    led.set_high().unwrap();
                    Timer::after(Duration::from_millis(50)).await;
                    led.set_low().unwrap()
                }
                State::Steady => led.set_high().unwrap(),
                State::Off => led.set_low().unwrap(),
                State::Interrupt => {
                    led.toggle().unwrap();
                    Timer::after(Duration::from_millis(5)).await;
                    led.toggle().unwrap();
                }
            }
        }
    }
}


pub mod disp {
    use super::*;
    use ybus::{SharedI2cDevice, SharedDeviceMutex};
    use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306Async};
    pub type OneLine = String<20>;
    pub type FourLines = [Option<OneLine>; 4];

    pub static TEXT: Signal<RawMutex, FourLines> = Signal::new();

    pub async fn task<M, B>(i2c: SharedI2cDevice<'_, M, B>)
    where
    	M: SharedDeviceMutex,
     	B: embedded_hal_async::i2c::I2c,
    {
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
