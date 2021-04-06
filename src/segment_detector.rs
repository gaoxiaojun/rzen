use crate::fractal::Fractal;
use crate::pen_detector::PenEvent;

#[derive(Debug, Clone)]
pub enum SegmentEvent {
    First(Fractal, Fractal, Fractal, Fractal),
    New(),
}
#[derive(Debug, Clone)]
pub struct SegmentDetector {
    segments: Vec<Fractal>,
    has_segment: bool,
}

impl SegmentDetector {
    fn new() -> Self {
        Self {
            segments: Vec::new(),
            has_segment: false,
        }
    }

    pub fn on_pen_event(&mut self, event: &PenEvent) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bar::Bar;
    use crate::fractal_detector::FractalDetector;
    use crate::pen_detector::PenDetector;
    use chrono::prelude::*;
    #[test]
    fn test_segment_detector() {
        let bars = load_bar2();
        let mut fd = FractalDetector::new();
        let mut pd = PenDetector::new();
        let mut sd = SegmentDetector::new();
        for bar in &bars {
            let fractal = fd.on_new_bar(bar);
            if let Some(f) = fractal {
                let pe = pd.on_new_fractal(f);
                if let Some(pen_event) = pe {
                    sd.on_pen_event(&pen_event);
                }
            }
        }
    }

    fn load_bar2() -> Vec<Bar> {
        let mut bars: Vec<Bar> = Vec::new();
        let csv = include_str!("../tests/EURUSD-2010_09_01-2010_09_31.csv");
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv.as_bytes());

        for record in reader.records() {
            let record = record.unwrap();
            let timestr: &str = AsRef::<str>::as_ref(&record[0]);
            let dt = NaiveDateTime::parse_from_str(timestr, "%Y-%m-%d %H:%M:%S").unwrap();
            let datetime: DateTime<Utc> = DateTime::from_utc(dt, Utc);
            let time = datetime.timestamp_millis();
            let open = AsRef::<str>::as_ref(&record[1]).parse::<f64>().unwrap();
            let high = AsRef::<str>::as_ref(&record[2]).parse::<f64>().unwrap();
            let low = AsRef::<str>::as_ref(&record[3]).parse::<f64>().unwrap();
            let close = AsRef::<str>::as_ref(&record[4]).parse::<f64>().unwrap();
            let bar = Bar::new(time, open, high, low, close);
            bars.push(bar);
        }
        bars
    }
}
