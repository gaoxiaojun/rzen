use crate::candle::Candle;

#[derive(Clone)]
pub enum FractalType {
    Top,
    Bottom,
}
#[derive(Clone)]
pub struct Fractal {
    pub ftype: FractalType,
    id: u64,
    pub k1: Candle,
    pub k2: Candle,
    pub k3: Candle,
}

impl Fractal {
    pub fn new(ftype: FractalType, id: u64, k1: Candle, k2: Candle, k3: Candle) -> Self {
        Self {
            ftype,
            id,
            k1,
            k2,
            k3,
        }
    }

    pub fn distance(&self, other: &Fractal) -> u64 {
        if other.id > self.id {
            other.id - self.id
        } else {
            self.id - other.id
        }
    }
}
