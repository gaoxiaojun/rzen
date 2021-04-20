use crate::bar::Bar;
use crate::fractal::Fractal;
use crate::fractal_detector::FractalDetector;
use crate::pen_detector::PenDetector;
use crate::segment_detector::SegmentDetector;

#[derive(Debug)]
pub struct Analyzer {
    fd: FractalDetector,
    pd: PenDetector,
    sd: SegmentDetector,
    fractals: Vec<Fractal>,
    pens: Vec<usize>,
    segments: Vec<usize>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            fd: FractalDetector::new(),
            pd: PenDetector::new(),
            sd: SegmentDetector::new(),
            fractals: Vec::new(),
            pens: Vec::new(),
            segments: Vec::new(),
        }
    }

    pub fn on_new_bar(&mut self, bar: &Bar) {
        let fractal = self.fd.on_new_bar(bar);
        if let Some(f) = fractal {
            let pe = self.pd.on_new_fractal(f);
            if let Some(pen_event) = pe {
                self.sd.on_pen_event(pen_event);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::tests::*;

    #[test]
    fn test_analyzer() {
        let bars = load_eurusd_2021();
        let mut analyzer = Analyzer::new();
        for bar in &bars {
            analyzer.on_new_bar(bar);
        }
        let count = analyzer.fractals.len();
        println!("count = {}", count);
    }
}
