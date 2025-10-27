//pub use ylab_lib::{Duration, Instant, RawMutex, Signal, Timer};
pub use super::*;
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
