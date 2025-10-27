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
