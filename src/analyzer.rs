use crate::bar::Bar;
use crate::candle::Candle;
use crate::fractal::Fractal;
use crate::time::Time;

pub struct Analyzer {}

impl Analyzer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn on_new_fractal(&self, fractal: Fractal) {}

    pub fn on_new_pen() {}
}
