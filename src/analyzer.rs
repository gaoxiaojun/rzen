use crate::fractal::Fractal;
use crate::fractal_detector::FractalDetector;
use crate::pen_detector::PenDetector;
use crate::{bar::Bar, pen_detector::PenEvent};

#[derive(Debug)]
pub struct Analyzer {
    fd: FractalDetector,
    pd: PenDetector,
    pub pens: Vec<Fractal>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            fd: FractalDetector::new(),
            pd: PenDetector::new(),
            pens: Vec::new(),
        }
    }

    pub fn on_new_bar(&mut self, bar: &Bar) {
        let fractal = self.fd.on_new_bar(bar);
        if let Some(f) = fractal {
            let pe = self.pd.on_new_fractal(f);
            if let Some(pen_event) = pe {
                match pen_event {
                    PenEvent::First(a, b) => {
                        self.pens.push(a);
                        self.pens.push(b);
                    }
                    PenEvent::New(a) => {
                        self.pens.push(a);
                    }

                    PenEvent::UpdateTo(a) => {
                        self.pens.pop();
                        self.pens.push(a);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bar::Bar;
    use chrono::{DateTime, NaiveDateTime, Utc};
    use csv;

    #[test]
    fn test_analyzer() {
        let bars = load_bar2();
        let mut analyzer = Analyzer::new();
        for bar in &bars {
            analyzer.on_new_bar(bar);
        }
        let count = analyzer.pens.len();
        println!("count = {}", count);
        assert!(count == 1899);
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
