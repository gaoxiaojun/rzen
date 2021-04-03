use crate::{candle::Candle, time::Time};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractalType {
    Top,
    Bottom,
}

// 分型
#[derive(Debug, Clone)]
pub struct Fractal {
    index: u64,
    k1: Candle,
    k2: Candle,
    k3: Candle,
    // cache
    ftype: FractalType,
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
    pub fn new(index: u64, k1: Candle, k2: Candle, k3: Candle) -> Self {
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

        Self {
            index,
            k1,
            k2,
            k3,
            ftype,
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
    }

    // 分型区间概念
    // 顶分型是[k1,k3最低点, k2.high]
    // 底分型是[k2.low, k1,k3最高点]
    pub fn range(&self) -> (f64, f64) {
        if self.ftype == FractalType::Top {
            (f64::min(self.k1.low, self.k3.low), self.k2.high)
        } else {
            (self.k2.low, f64::max(self.k1.high, self.k3.high))
        }
    }

    // 分型高点概念
    // 顶分型 -> 最高点
    // 底分型 -> 分型第二元素的高点
    pub fn high(&self) -> f64 {
        if self.ftype == FractalType::Top {
            self.k2.high
        } else {
            self.k3.high
        }
    }

    // 分型低点概念
    // 顶分型 -> 最低点
    // 底分型 -> 分型第二元素的低点
    pub fn low(&self) -> f64 {
        if self.ftype == FractalType::Bottom {
            self.k2.low
        } else {
            self.k3.low
        }
    }

    pub fn is_contain(&self, other: &Fractal) -> bool {
        let lhs_highest = f64::max(f64::max(self.k1.high, self.k2.high), self.k3.high);
        let lhs_lowest = f64::min(f64::min(self.k1.low, self.k2.low), self.k3.low);

        let rhs_highest = f64::max(f64::max(other.k1.high, other.k2.high), other.k3.high);
        let rhs_lowest = f64::min(f64::min(other.k1.low, other.k2.low), other.k3.low);

        if lhs_highest >= rhs_highest && lhs_lowest <= rhs_lowest {
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
        let k1 = Candle::new(2000000, 100.0, 30.0);
        let k2 = Candle::new(2000001, 150.0, 120.0);
        let k3 = Candle::new(2000002, 130.0, 60.0);

        let k4 = Candle::new(3000000, 90.0, 60.0);
        let k5 = Candle::new(3000001, 70.0, 30.0);
        let k6 = Candle::new(3000002, 80.0, 50.0);
        let f1 = Fractal::new(10, k1, k2, k3);
        let f2 = Fractal::new(12, k4, k5, k6);

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
