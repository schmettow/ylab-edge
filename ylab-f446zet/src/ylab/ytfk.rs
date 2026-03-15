pub use super::*;

pub mod bsu {
    pub use super::*;

    // Channel
    pub static SINK: Channel<RawMutex, Ytf, 8> = Channel::new();

    // USB
    use mcu::usart::Uart;
    #[embassy_executor::task]
    pub async fn task(mut usart: Uart<'static, mcu::mode::Async>) {
        loop {
            let sample: Ytf = SINK.receive().await;
            let mut msg: Vec<u8, 256> = Vec::new();
            match core::write!(&mut msg, "{}\n", sample) {
                Ok(_) => {usart.write(&msg).await.unwrap()},
                Err(_) => {},
            }
            //usart.write(&msg).await.unwrap()
        }
    }
}
