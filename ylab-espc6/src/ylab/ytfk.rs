pub use super::*;
pub use ylab_lib as yll;
pub use yll::ydata::Ytf;
pub use yll::{Channel, RawMutex};

pub mod bsu {
    pub use super::*;

    // Channel
    pub static SINK: Channel<RawMutex, Ytf, 8> = Channel::new();

    // USB
    use mcu::uart::Uart;
    //use embedded_io::Write;
    //use crate::mcu::Async;
    #[embassy_executor::task]
    pub async fn task(mut tx: Uart<'static, Async>) {
        //use core::fmt::Write;
        use embedded_io_async::Write as io;
        io::flush(&mut tx).await.unwrap();
        write!(&mut tx, "# YLab data channel").unwrap();
        io::flush(&mut tx).await.unwrap();
        loop {
            let sample: Ytf = SINK.receive().await;
            //let mut out: Vec<u8, 256> = Vec::new();
            //core::write!(&mut out, "{}\n", sample);
            write!(&mut tx, "{}", sample).unwrap();
            io::flush(&mut tx).await.unwrap();
            //esp_println::println!("{}", sample);
        }
    }

    #[embassy_executor::task]
    pub async fn task_println() -> ! {
        //println!("# YLab data channel");
        loop {
            let sample: Ytf = SINK.receive().await;
            println!("{}", sample);
        }
    }
}
