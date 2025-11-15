pub use ylab_lib as yll;
pub use yll::{Channel, RawMutex};
pub use yll::ydata::Ytf;
pub use super::*;

pub mod bsu {
    pub use super::*;

    // Channel
    pub static SINK: Channel<RawMutex, Ytf, 8> = Channel::new();

    // USB
    use mcu::uart::Uart;
    use embedded_io::Write;
    //use crate::mcu::Async;
    #[embassy_executor::task]
    pub async fn task(mut tx: Uart<'static, Async>) {
   		//use core::fmt::Write;
        //embedded_io_async::Write::flush(&mut tx).await.unwrap();
        loop {
        	let sample: Ytf = SINK.receive().await;
            write!(&mut tx, "{}", sample).unwrap();
            embedded_io_async::Write::flush(&mut tx).await.unwrap();
        }
    }



}
