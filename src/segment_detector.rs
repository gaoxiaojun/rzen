use crate::fractal::Fractal;

// 三笔重叠判断算法
// 三笔4个端点必须：
// 情况A：向上线段   1<=min(2,3,4) && 4>=max(1,2,3)
// 情况B：向下线段   1>=max(2,3,4) && 4<=min(1,2,3)

#[derive(Debug, Clone)]
pub enum SegmentEvent {
    New(usize, usize),
    UpdateTo(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum SegmentDirection {
    Up,
    Down,
}

pub type FractalVecIndex = usize;

#[derive(Debug, Clone)]
pub struct SegmentDetector {
    segments: Vec<FractalVecIndex>,
    direction: Option<SegmentDirection>,
    last: Option<(FractalVecIndex, FractalVecIndex)>,
}

impl SegmentDetector {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            direction: None,
            last: None,
        }
    }

    pub fn on_pen_event(&mut self, pens: &Vec<Fractal>) -> Option<SegmentEvent> {
        // pens数组是先保存最新的笔，然后调用本方法，所以至少需要5个端点
        if pens.len() < 5 {
            return None;
        }

        // 为了防止今后修改上述逻辑
        let length = pens.len();
        let p1_index = length - 5;
        let p2_index = length - 4;
        let p3_index = length - 3;
        let p4_index = length - 2;

        if !self.direction.is_none() {
            let p1 = &pens[p1_index];
            let p2 = &pens[p2_index];
            let p3 = &pens[p3_index];
            let p4 = &pens[p4_index];

            let direction_up =
                p1.price() < p2.price() && p2.price() > p3.price() && p3.price() < p4.price(); // ?? p2.price() <= p4.price()
            let direction_down =
                p1.price() > p2.price() && p2.price() < p3.price() && p3.price() > p4.price(); // ?? p2.price() >= p4.price()

            self.direction = {
                match (direction_up, direction_down) {
                    (true, false) => Some(SegmentDirection::Up),
                    (false, true) => Some(SegmentDirection::Down),
                    (_, _) => None,
                }
            };

            if self.direction.is_some() {
                self.segments.push(p4_index);
                self.segments.push(p1_index);
                self.last = Some((p2_index, p3_index));
                return Some(SegmentEvent::New(p1_index, p4_index));
            } else {
                return None;
            }
        } else {
            None
        }
    }
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
                    //sd.on_pen_event(&pen_event);
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
