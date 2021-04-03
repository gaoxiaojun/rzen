use crate::{candle::Candle, time::Time};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractalType {
    Top,
    Bottom,
}

// 分型
#[derive(Debug, Clone)]
pub struct Fractal {
    ftype: FractalType,
    index: u64,
    k1: Candle,
    k2: Candle,
    k3: Candle,
}

// 计算分型之间K线的数量,K线是经过包含处理过的
fn distance(lhs: &Fractal, rhs: &Fractal) -> u64 {
    if rhs.index > lhs.index {
        rhs.index - lhs.index
    } else {
        lhs.index - rhs.index
    }
}

impl Fractal {
    pub fn new(ftype: FractalType, index: u64, k1: Candle, k2: Candle, k3: Candle) -> Self {
        Self {
            ftype,
            index,
            //high: f64::max(f64::max(k1.high, k2.high), k3.high),
            //low: f64::min(f64::min(k1.low, k2.low), k3.low),
            k1,
            k2,
            k3,
        }
    }

    pub fn get_k1(&self) -> &Candle {
        &self.k1
    }

    pub fn get_k2(&self) -> &Candle {
        &self.k2
    }

    pub fn get_k3(&self) -> &Candle {
        &self.k3
    }

    pub(crate) fn distance(&self, other: &Fractal) -> u64 {
        distance(self, other)
    }

    pub fn has_enough_distance(&self, other: &Fractal) -> bool {
        self.distance(other) >= 4
    }

    pub fn is_same_type(&self, other: &Fractal) -> bool {
        self.ftype == other.ftype
    }

    pub fn time(&self) -> Time {
        self.k2.time
    }

    pub fn fractal_type(&self) -> FractalType {
        self.ftype

        //if (self.k1.high < self.k2.high) && (self.k2.high > self.k3.high) {
        //    FractalType::Top
        //} else {
        //    FractalType::Bottom
        //}
    }

    pub fn highest(&self) -> f64 {
        //self.high
        f64::max(f64::max(self.k1.high, self.k2.high), self.k3.high)
    }

    pub fn lowest(&self) -> f64 {
        //self.low
        f64::min(f64::min(self.k1.low, self.k2.low), self.k3.low)
    }

    pub fn high(&self) -> f64 {
        self.highest()
    }

    pub fn low(&self) -> f64 {
        self.lowest()
    }

    pub fn has_same_price(&self, other: &Fractal) -> bool {
        debug_assert!(self.ftype == other.ftype);
        if self.ftype == FractalType::Top {
            if self.k2.high == other.k2.high {
                true
            } else {
                false
            }
        } else {
            if self.k2.low == other.k2.low {
                true
            } else {
                false
            }
        }
    }

    pub fn is_contain(&self, other: &Fractal) -> bool {
        if (self.high() >= other.high() && self.low() <= other.low())
            || (other.high() >= self.high() && other.low() <= self.low())
        {
            true
        } else {
            false
        }
    }
}

impl PartialEq for Fractal {
    fn eq(&self, other: &Self) -> bool {
        self.time() == other.time()
    }
}

#[cfg(test)]
mod tests {
    use crate::{candle::Candle, fractal::Fractal, fractal::FractalType};
    #[test]
    fn test_distance_and_eq() {
        let k1 = Candle::new(2000000, 100.0, 50.0);
        let k2 = Candle::new(2000001, 150.0, 120.0);
        let k3 = Candle::new(2000002, 130.0, 60.0);

        let k4 = Candle::new(3000000, 90.0, 60.0);
        let k5 = Candle::new(3000001, 70.0, 30.0);
        let k6 = Candle::new(3000002, 80.0, 50.0);
        let f1 = Fractal::new(FractalType::Top, 10, k1, k2, k3);
        let f2 = Fractal::new(FractalType::Bottom, 12, k4, k5, k6);

        let d1 = f1.distance(&f2);
        let d2 = f2.distance(&f1);

        assert_eq!(d1, d2);
        assert_eq!(d1, 2);

        // test eq
        assert_ne!(f1, f2);
    }
}
