pub use super::*;
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Sample<M, const N: usize>
where
    M: Into<YtfType>,
{
    pub sensory: u8,
    pub time: Instant,
    pub read: [M; N],
}

pub const YTF_LEN: usize = 8;
pub type YtfType = f64;
pub type YtfRead = [Option<YtfType>; YTF_LEN];

pub struct Ytf {
    pub sensory: u8,
    pub time: Instant,
    pub read: YtfRead,
}

impl<M: Into<YtfType>, const N: usize> Into<Ytf> for Sample<M, N> {
    fn into(self) -> Ytf {
        let mut out: YtfRead = [None; YTF_LEN];
        for (i, r) in self.read.into_iter().enumerate() {
            out[i] = Some(r.into());
        }
        Ytf {
            sensory: self.sensory,
            time: self.time,
            read: out,
        }
    }
}

impl core::fmt::Display for Ytf {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{},{}", self.time.as_micros(), self.sensory).unwrap();
        for r in self.read {
            match r {
                Some(v) => {
                    write!(f, ",{:.3}", v).unwrap();
                }
                None => {
                    write!(f, ",").unwrap();
                }
            }
        }
        write!(f, "")
    }
}
