use crate::time::Time;
#[derive(Clone)]
pub struct Bar {
    pub time: Time,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl Bar {
    pub fn new(time: Time, open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            time,
            open,
            high,
            low,
            close,
        }
    }
}
