use crate::{candle::Candle, time::Time};

#[derive(Clone, Copy)]
pub enum FractalType {
    Top,
    Bottom,
}
#[derive(Clone)]
pub struct Fractal {
    ftype: FractalType,
    id: u64,
    //high: f64,
    //low: f64,
    k1: Candle,
    k2: Candle,
    k3: Candle,
}

impl Fractal {
    pub fn new(ftype: FractalType, id: u64, k1: Candle, k2: Candle, k3: Candle) -> Self {
        Self {
            ftype,
            id,
            //high: f64::max(f64::max(k1.high, k2.high), k3.high),
            //low: f64::min(f64::min(k1.low, k2.low), k3.low),
            k1,
            k2,
            k3,
        }
    }

    fn distance(&self, other: &Fractal) -> u64 {
        if other.id > self.id {
            other.id - self.id
        } else {
            self.id - other.id
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
        // case 1: 共用K
        if self.distance(other) < 3 {
            return false;
        }
        // case 2: 转折力度太小，有包含关系，这里前包含和后包含均不允许
        let lh = self.high();
        let ll = self.low();
        let rh = other.high();
        let rl = other.low();
        if (lh >= rh && ll <= rl) || (rh >= lh && rl <= ll) {
            false
        } else {
            true
        }
    }
}
