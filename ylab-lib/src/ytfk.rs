use super::*;
use core::fmt::Write;
pub use ydata::Ytf;

//pub type Ytf = Sample<[Option<f32>; 8]>; // standard transport format
#[allow(dead_code)]
type YtfLine = Vec<u8, 512>;
#[allow(dead_code)]
trait YtfSend {
    fn msg_csv(&self) -> Result<YtfLine, core::fmt::Error>;
    fn msg_bin(&self) -> Result<YtfLine, core::fmt::Error>;
}

impl YtfSend for Ytf {
    fn msg_csv(&self) -> Result<YtfLine, core::fmt::Error> {
        let mut msg: YtfLine = Vec::new();
        match core::write!(&mut msg, "{}", self) {
            Ok(_) => return Ok(msg),
            Err(e) => return Err(e),
        }
    }

    fn msg_bin(&self) -> Result<YtfLine, core::fmt::Error> {
        todo!()
    }
}

pub type YtfChannel = Channel<RawMutex, Ytf, 8>;
pub type YtfSender<'a> = embassy_sync::channel::Sender<'a, RawMutex, Ytf, 8>;
