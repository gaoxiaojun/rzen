use crate::fractal::{Fractal, FractalType};
use crate::ringbuffer::RingBuffer;

// 一、寻找第一笔
// state 0
// +-0-+                            +-1-+
// |   |<-----A             =====>  | A |
// +---+                            +---+
// 保留A，转state1

// state 1
// +-1-+                            +-1-+               +--2-3--+
// | A |<-----B             ======> |A/B|        or     | A | B |
// +---+                            +---+               +---+---+
// 1.1 AB同类型，按同类合并规则处理AB，转state1
// 1.2 AB不同类型，保存B
// 1.2.1 AB不成笔转state2
// 1.2.2 AB成笔, emit First(A,B), has_pen=true，转state3

// state 2
// +---2---+                        +---3---+           +--2-3--+       +-1-+
// | A | B |<-----C         =====>  | B | C |     or    | A |B/C|   or  |A/C|
// +---+---+                        +--2.1--+           +---+---+       +---+
// 前提：AB不成笔
// 2.1 BC成笔 ---- 去掉A，保留BC，emit First(B,C), has_pen=true, 转state3
// 2.2 BC不成笔
// 2.2.1 BC同类，按同类合并规则处理
// 2.2.1.1 如果保留C，要检测AC是否成笔
// 2.2.1.1.1如果AC成笔，emit First(A,C),has_pen = true, 转state3
// 2.2.1.1.2如果不成笔，转state 2
// 2.2.1.2 如果保留B，抛弃C，转state2
// 2.2.2 BC不同类，按同类合并规则处理AC
// 2.2.2.1 如果保留A，则保留B，抛弃C，转state2
// 2.2.2.2 如果保留C，抛弃AB，转state1

// 二、已经有笔
// state 3
// +---3---+                        +---3---+           +---+-4-+---+
// | A | B |<-----C         =====>  | B | C |      or   | A | B | C |
// +---+---+                        +---+---+           +---+---+---+
// 前提 AB成笔
//  3.1 BC成笔   --- AB笔完成，emit New(C)，去掉A，剩下BC,转state3
//  3.2 BC不成笔，
//  3.2.1 BC类型不同，保留C，转state4
//  3.2.2 BC类型相同，按同类合并规则处理BC
//  3.2.2.1如果保留C，emit UpdateTo(C)，转state3
//  3.2.2.2如果保留B，抛弃C，转state3

// state 4
// +---+-4-+---+                    +---+-4-+---+       +---3---+
// | A | B | C |<-----D     =====>  | A | B |C/D|   or  | A |B/D|
// +---+---+---+                    +---+---+---+       +---+---+
// 前提 AB成笔且BC类型不同且BC不成笔
// 4.1 CD同类型-----按同类合并规则处理CD
// 4.1.1 如果保留D，要检测BD是否成笔
// 4.1.1.1如果BD成笔，AB笔完成，emit New(D)，去掉A，剩下BD，转state3
// 4.1.1.2如果不成笔，转 state4
// 4.1.2 如果保留C，抛弃D，转state4
// 4.2 CD不同类-----去掉C，按同类合并规则处理BD
// 4.2.1 如果保留B,转state3
// 4.2.2 如果保留D，emit UpdateTo(D)，转state3

// 上述算法解决的99%的笔问题，但是还有一种情况，无法完美处理
// 例子:
// A-B-C-D-E
// 以A-B为向上笔为例，
// A-B向上成笔 B-C向下成笔 C-D向上不成笔，但是D高于B，E低于C, DE成笔
// 按照规则A-B笔在BC成笔的已经确认，最终B-E成向下笔，但是B不是这一笔的最高点
// 按照完美成笔，应该A-D成向上笔，D-E成向下笔，这样笔的端点是中间K的最高最低点，完美符合笔忽略中间波动的要求
// 这种情况实际上是要修正已经确认完成的笔，与当下确认笔有冲突的大原则有冲突
// 按照缠论从A0(1分钟)开始做推笔，线段才是最基本的构件
// 非完美的笔对线段没有影响

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeAction {
    Keep,
    Replace,
}

pub fn _merge_same_type(f1: &Fractal, f2: &Fractal) -> MergeAction {
    debug_assert!(f1.fractal_type() == f2.fractal_type());
    if f1.fractal_type() == FractalType::Top {
        if f1.highest() > f2.highest() {
            MergeAction::Keep
        } else {
            MergeAction::Replace
        }
    } else {
        if f1.lowest() < f2.lowest() {
            MergeAction::Keep
        } else {
            MergeAction::Replace
        }
    }
}

pub fn _is_valid_fractal(f1: &Fractal, f2: &Fractal) -> bool {
    // 1.1 共享K线分析，后分型无效
    //if f1.distance(f2) < 3 && !f1.is_same_type(f2) {
    //    // 共享K线分型，
    //    return false;
    //}

    // 1.2 包含关系分析，无效
    if f1.is_contain(f2) {
        return false;
    }

    true
}

pub fn is_pen(f1: &Fractal, f2: &Fractal) -> bool {
    if f1.fractal_type() == FractalType::Top
        && f2.fractal_type() == FractalType::Bottom
        && f1.has_enough_distance(f2)
        && f2.lowest() < f1.lowest()
    {
        return true;
    }

    if f1.fractal_type() == FractalType::Bottom
        && f2.fractal_type() == FractalType::Top
        && f1.has_enough_distance(f2)
        && f2.highest() > f1.highest()
    {
        return true;
    }

    false
}

#[derive(Debug, Clone)]
pub enum PenEvent {
    First(Fractal, Fractal),
    New(Fractal),
    UpdateTo(Fractal),
}

// TODO:考虑一种特殊情况就是顶分型高点相等或者底分型低点相等
#[derive(Debug)]
pub struct PenDetector {
    window: RingBuffer<Fractal>,
    has_pen: bool,
}

impl PenDetector {
    pub fn new() -> Self {
        Self {
            window: RingBuffer::new(3),
            has_pen: false,
        }
    }

    fn _is_pen(&self, start_index: usize) -> bool {
        debug_assert!(self.window.len() >= 2 + start_index);
        is_pen(
            self.window.get(start_index as isize).unwrap(),
            self.window.get((start_index + 1) as isize).unwrap(),
        )
    }

    fn bc_is_pen(&self) -> bool {
        self._is_pen(1)
    }

    fn ab_is_pen(&self) -> bool {
        self._is_pen(0)
    }

    fn state0(&mut self, f: Fractal) -> Option<PenEvent> {
        debug_assert!(self.window.len() == 0 && !self.has_pen);
        self.window.push(f);
        None
    }

    fn state1(&mut self, f: Fractal) -> Option<PenEvent> {
        debug_assert!(self.window.len() == 1 && !self.has_pen);
        let last = self.window.get(-1).unwrap();
        if last.is_same_type(&f) {
            // 1.1
            let action = _merge_same_type(last, &f);
            if action == MergeAction::Replace {
                self.window.pop_back();
                self.window.push(f);
            }
        } else {
            // 1.2
            self.window.push(f);
            if self.ab_is_pen() {
                // 1.2.2
                self.has_pen = true;
                return Some(PenEvent::First(
                    self.window.get(0).unwrap().clone(),
                    self.window.get(1).unwrap().clone(),
                ));
            }
        }
        None
    }

    fn state2(&mut self, f: Fractal) -> Option<PenEvent> {
        debug_assert!(!self.ab_is_pen());
        debug_assert!(!self.has_pen);
        debug_assert!(self.window.len() == 2);

        let b = self.window.get(-1).unwrap();
        let bc_is_pen = is_pen(b, &f);
        if bc_is_pen {
            // 2.1
            self.window.push(f);
            self.window.pop_front();
            self.has_pen = true;
            return Some(PenEvent::First(
                self.window.get(0).unwrap().clone(),
                self.window.get(1).unwrap().clone(),
            ));
        } else {
            // 2.2
            if b.is_same_type(&f) {
                // 2.2.1
                let action = _merge_same_type(b, &f);
                if action == MergeAction::Replace {
                    // 2.2.1.1
                    self.window.pop_back(); // pop b
                    self.window.push(f);
                    // test ac is pen?
                    if self.ab_is_pen() {
                        // 2.2.1.1.1
                        self.has_pen = true;
                        return Some(PenEvent::First(
                            self.window.get(0).unwrap().clone(),
                            self.window.get(1).unwrap().clone(),
                        ));
                    }
                }
            } else {
                // 2.2.2
                let a = self.window.get(0).unwrap();
                let action = _merge_same_type(a, &f);
                if action == MergeAction::Replace {
                    // 2.2.2.2
                    self.window.clear();
                    self.window.push(f);
                }
            }
        }
        None
    }

    fn state3(&mut self, f: Fractal) -> Option<PenEvent> {
        debug_assert!(self.ab_is_pen());
        debug_assert!(self.has_pen);
        debug_assert!(self.window.len() == 2);

        let b = self.window.get(-1).unwrap();
        let bc_is_pen = is_pen(b, &f);
        if bc_is_pen {
            // 3.1
            let c = f.clone();
            self.window.pop_front();
            self.window.push(f);
            //self.ab_pen_complete_bc_pen_new();
            return Some(PenEvent::New(c));
        } else {
            if b.is_same_type(&f) {
                let action = _merge_same_type(b, &f);
                if action == MergeAction::Replace {
                    // 3.2.2.1
                    self.window.pop_back();
                    let c = f.clone();
                    self.window.push(f);
                    //self.ab_pen_update();
                    return Some(PenEvent::UpdateTo(c));
                }
            } else {
                // 3.2.1
                self.window.push(f);
            }
        }

        None
    }

    fn state4(&mut self, f: Fractal) -> Option<PenEvent> {
        debug_assert!(self.ab_is_pen());
        debug_assert!(!self
            .window
            .get(-2)
            .unwrap()
            .is_same_type(self.window.get(-1).unwrap()));
        debug_assert!(!is_pen(
            self.window.get(-2).unwrap(),
            self.window.get(-1).unwrap()
        ));
        debug_assert!(self.has_pen);
        debug_assert!(self.window.len() == 3);

        let c = self.window.get(-1).unwrap();
        if c.is_same_type(&f) {
            // 4.1
            let action = _merge_same_type(c, &f);
            if action == MergeAction::Replace {
                // 4.1.1
                self.window.pop_back();
                self.window.push(f);
                if self.bc_is_pen() {
                    // 4.1.1.1
                    self.window.pop_front();
                    return Some(PenEvent::New(self.window.get(-1).unwrap().clone()));
                }
            }
        } else {
            // 4.2
            self.window.pop_back();
            let b = self.window.get(-1).unwrap();
            let action = _merge_same_type(b, &f);
            if action == MergeAction::Replace {
                // 4.2.2
                self.window.pop_back();
                let c = f.clone();
                self.window.push(f);
                return Some(PenEvent::UpdateTo(c));
            }
        }

        None
    }

    pub fn on_new_fractal(&mut self, f: Fractal) -> Option<PenEvent> {
        // step1: valid fractal
        /*if let Some(last) = self.window.get(-1) {
            if !_is_valid_fractal(last, &f) {
                return None;
            }
        }*/

        // step2: process fractal
        let len = self.window.len();
        let is_pen = self.has_pen;

        match (is_pen, len) {
            (false, 0) => self.state0(f),
            (false, 1) => self.state1(f),
            (false, 2) => self.state2(f),
            (true, 2) => self.state3(f),
            (true, 3) => self.state4(f),
            (_, _) => {
                unreachable!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bar::Bar;
    use crate::candle::Candle;
    use crate::fractal::Fractal;
    use crate::fractal_detector::FractalDetector;
    use crate::pen_detector::PenDetector;
    use crate::plot::{draw_bar_tradingview, draw_bar_vue};
    use crate::time::Time;
    use chrono::prelude::*;
    use csv;
    #[test]
    fn test_is_pen() {
        let k1 = Candle::new(1117, 1052779380000, 1.15642, 1.15627);
        let k2 = Candle::new(1118, 1052779380000, 1.15645, 1.15634);
        let k3 = Candle::new(1119, 1052779500000, 1.15638, 1.1562);
        let f1 = Fractal::new(k1, k2, k3);
        let k4 = Candle::new(1131, 1052780640000, 1.15604, 1.1559);
        let k5 = Candle::new(1132, 1052780820000, 1.15602, 1.15576);
        let k6 = Candle::new(1133, 1052780940000, 1.15624, 1.15599);
        let f2 = Fractal::new(k4, k5, k6);
        let has_enough_distance = f1.has_enough_distance(&f2);
        assert!(has_enough_distance);
        println!(
            "f1.type = {:?} f1.range = {:?}, f2.type = {:?}, f2.range = {:?}",
            f1.fractal_type(),
            (f1.highest(), f1.lowest()),
            f2.fractal_type(),
            (f2.highest(), f1.lowest())
        );
        let is_pen = is_pen(&f1, &f2);
        assert!(is_pen);
    }
    #[test]
    fn test_pen_detector() {
        let (bars, fractals) = load_fractal();
        println!("total fractals:{}", fractals.len());

        let mut fq = PenDetector::new();

        let mut pen_count = 0;
        let mut pen_update = 0;
        let mut pens: Vec<Fractal> = Vec::new();
        for f in fractals {
            let event = fq.on_new_fractal(f.clone());
            if let Some(pen_event) = event {
                match pen_event {
                    PenEvent::First(a, b) => {
                        pens.push(a);
                        pens.push(b);
                        pen_count += 1;
                    }
                    PenEvent::New(a) => {
                        pens.push(a);
                        pen_count += 1;
                    }

                    PenEvent::UpdateTo(a) => {
                        pens.pop();
                        pens.push(a);
                        pen_update += 1;
                    }
                }
            }
        }

        println!("pen_count = {}, pen_update ={}", pen_count, pen_update);
        //draw_bar_vue(&bars);
        draw_bar_tradingview(&bars, &pens);
    }

    fn load_fractal() -> (Vec<Bar>, Vec<Fractal>) {
        let mut fractals: Vec<Fractal> = Vec::new();
        let bars = load_duka_bar();
        let mut cq = FractalDetector::new();
        for bar in &bars {
            if let Some(f) = cq.on_new_bar(bar) {
                fractals.push(f);
            }
        }
        (bars, fractals)
    }

    #[allow(dead_code)]
    fn load_duka_bar() -> Vec<Bar> {
        // duka download datetime timezone is GMT+8
        let mut bars: Vec<Bar> = Vec::new();
        let csv = include_str!("../tests/EURUSD-2021_04_01-2021_04_06.csv");
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv.as_bytes());

        let china_timezone = FixedOffset::east(8 * 3600);
        for record in reader.records() {
            let record = record.unwrap();
            let timestr: &str = AsRef::<str>::as_ref(&record[0]);
            let dt = NaiveDateTime::parse_from_str(timestr, "%Y-%m-%d %H:%M:%S").unwrap();
            let china_dt = dt + china_timezone;
            let datetime: DateTime<Utc> = DateTime::from_utc(china_dt, Utc);
            let time = datetime.timestamp_millis();
            let open = AsRef::<str>::as_ref(&record[1]).parse::<f64>().unwrap();
            let close = AsRef::<str>::as_ref(&record[2]).parse::<f64>().unwrap();
            let high = AsRef::<str>::as_ref(&record[3]).parse::<f64>().unwrap();
            let low = AsRef::<str>::as_ref(&record[4]).parse::<f64>().unwrap();
            let bar = Bar::new(time, open, high, low, close);
            bars.push(bar);
        }
        bars
    }

    #[allow(dead_code)]
    fn load_bar() -> Vec<Bar> {
        let mut bars: Vec<Bar> = Vec::new();
        let csv = include_str!("../tests/eurusd_10000.csv");
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv.as_bytes());

        for record in reader.records() {
            let record = record.unwrap();
            let time: Time = AsRef::<str>::as_ref(&record[0]).parse::<i64>().unwrap();
            let open = AsRef::<str>::as_ref(&record[1]).parse::<f64>().unwrap();
            let high = AsRef::<str>::as_ref(&record[2]).parse::<f64>().unwrap();
            let low = AsRef::<str>::as_ref(&record[3]).parse::<f64>().unwrap();
            let close = AsRef::<str>::as_ref(&record[4]).parse::<f64>().unwrap();
            let bar = Bar::new(time, open, high, low, close);
            bars.push(bar);
        }
        assert!(bars.len() == 10000);
        bars
    }
}
