use core::f64;

use crate::bar::Bar;
use crate::time::Time;

// 经过包含处理的K线
#[derive(Debug, Clone)]
pub struct Candle {
    pub time: Time,
    pub high: f64,
    pub low: f64,
}

impl Candle {
    pub fn new(time: Time, high: f64, low: f64) -> Self {
        Self { time, high, low }
    }

    pub fn from_bar(bar: &Bar) -> Self {
        Self {
            time: bar.time,
            high: bar.high,
            low: bar.low,
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    #[test]
    fn test_candle() {
        let bar = Bar::new(1052692740000, 1.15166, 1.15176, 1.15156, 1.15176);
        let c = Candle::from_bar(&bar);
        assert!(c.time == bar.time);
        assert!(c.high == bar.high);
        assert!(c.low == bar.low);
    }
}
