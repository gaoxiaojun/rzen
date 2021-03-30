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
    //high: f64,
    //low: f64,
    k1: Candle,
    k2: Candle,
    k3: Candle,
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

    fn distance(&self, other: &Fractal) -> u64 {
        if other.index > self.index {
            other.index - self.index
        } else {
            self.index - other.index
        }
    }

    pub fn has_enough_distance(&self, other: &Fractal) -> bool {
        self.distance(other) >= 4
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

    pub fn high(&self) -> f64 {
        //self.high
        f64::max(f64::max(self.k1.high, self.k2.high), self.k3.high)
    }

    pub fn low(&self) -> f64 {
        //self.low
        f64::min(f64::min(self.k1.low, self.k2.low), self.k3.low)
    }

    pub fn is_valid_fractal(&self, other: &Fractal) -> bool {
        _is_valid_fractal(self, other)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FractalValidEnum {
    Prev,
    Next,
    None,
}

// 处理前后分型共用K的情况
// 规则一：如果两个分型类型不同，前分型为有效分型，后分型无效
// 规则二：如果两个分型类型相同，以高低点决定那个分型有效
fn share_k_fractal_is_valid(f1: &Fractal, f2: &Fractal) -> FractalValidEnum {
    if (f1.distance(f2) < 3) {
        if f1.fractal_type() != f2.fractal_type() {
            return FractalValidEnum::Prev;
        }

        if f1.fractal_type() == FractalType::Top {
            if f2.high() < f1.high() {
                return FractalValidEnum::Prev;
            } else {
                return FractalValidEnum::Next;
            }
        } else {
            if f2.low() > f1.low() {
                return FractalValidEnum::Prev;
            } else {
                return FractalValidEnum::Next;
            }
        }
    }
    FractalValidEnum::None
}

// 处理前后分型包含的情况
// 规则一：前分型包含后分型，后分型为无效分型
// 规则二：后分型包含前分型，前分型为无效分型
fn process_fractal_contain(f1: &Fractal, f2: &Fractal) -> bool {
    let f1_high = f1.high();
    let f1_low = f1.low();
    let f2_high = f2.high();
    let f2_low = f2.low();

    if (f1_high >= f2_high && f1_low <= f2_low) || (f2_high >= f1_high && f2_low <= f1_low) {
        false
    } else {
        true
    }
}

fn _is_valid_fractal(f1: &Fractal, f2: &Fractal) -> bool {
    process_fractal_contain(f1, f2)
}

#[cfg(test)]
mod tests {
    use crate::{candle::Candle, fractal::Fractal, fractal::FractalType};
    #[test]
    fn test_distance() {
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
    }
}
