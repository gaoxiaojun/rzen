use crate::{candle::Candle, time::Time};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractalType {
    Top,
    Bottom,
}
// 分型
#[derive(Debug, Clone)]
pub struct Fractal {
    k1: Candle,
    k2: Candle,
    k3: Candle,
    // cache
    ftype: FractalType,
}

// 计算分型之间K线的数量,K线是经过包含处理过的
fn distance(lhs: &Fractal, rhs: &Fractal) -> u64 {
    if rhs.k2.index > lhs.k2.index {
        rhs.k2.index - lhs.k2.index
    } else {
        lhs.k2.index - rhs.k2.index
    }
}

impl Fractal {
    pub fn new(k1: Candle, k2: Candle, k3: Candle) -> Self {
        debug_assert!(
            // 合并之后，分型的最高/最低是唯一的，所以没有等号
            ((k1.high < k2.high) && (k2.high > k3.high)) // Top
                || ((k1.low > k2.low) && (k2.low < k3.low)) // Bottom
        );

        let is_top = (k1.high < k2.high) && (k2.high > k3.high);
        let ftype = if is_top {
            FractalType::Top
        } else {
            FractalType::Bottom
        };

        Self { k1, k2, k3, ftype }
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
    }
    // 分型最高点
    pub fn highest(&self) -> f64 {
        if self.ftype == FractalType::Top {
            self.k2.high
        } else {
            f64::max(self.k1.high, self.k3.high)
        }
    }

    // 分型最低点
    pub fn lowest(&self) -> f64 {
        if self.ftype == FractalType::Bottom {
            self.k2.low
        } else {
            f64::min(self.k1.low, self.k3.low)
        }
    }

    pub fn is_contain(&self, other: &Fractal) -> bool {
        if self.highest() >= other.highest() && self.lowest() <= other.lowest() {
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
    use crate::{candle::Candle, fractal::Fractal};
    #[test]
    fn test_distance_and_eq() {
        let k1 = Candle::new(9, 2000000, 100.0, 30.0);
        let k2 = Candle::new(10, 2000001, 150.0, 120.0);
        let k3 = Candle::new(11, 2000002, 130.0, 60.0);

        let k4 = Candle::new(12, 3000000, 90.0, 60.0);
        let k5 = Candle::new(13, 3000001, 70.0, 30.0);
        let k6 = Candle::new(14, 3000002, 80.0, 50.0);
        let f1 = Fractal::new(k1, k2, k3);
        let f2 = Fractal::new(k4, k5, k6);

        let d1 = f1.distance(&f2);
        let d2 = f2.distance(&f1);

        assert_eq!(d1, d2);
        assert_eq!(d1, 2);

        // test eq
        assert_ne!(f1, f2);

        // test is_contain
        assert!(f1.is_contain(&f2));
    }
}
