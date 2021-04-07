use crate::fractal::Fractal;
use crate::fractal_util::{MergeAction, _is_pen, _is_valid_fractal, _merge_same_type};
use crate::pen::Pen;
use crate::pen_store::PenStore;
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
// 1.2.2 AB成笔,新建笔变量，转state3

// state 2
// +---2---+                        +---3---+           +--2-3--+       +-1-+
// | A | B |<-----C         =====>  | B | C |     or    | A |B/C|   or  |A/C|
// +---+---+                        +--2.1--+           +---+---+       +---+
// 前提：AB不成笔
// 2.1 BC成笔 ---- 去掉A，保留BC，新建笔变量，转state3
// 2.2 BC不成笔
// 2.2.1 BC同类，按同类合并规则处理
// 2.2.1.1 如果保留C，要检测AC是否成笔
// 2.2.1.1.1如果成笔，新建笔变量, 转state3
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
//  3.1 BC成笔   --- AB笔完成，emit笔完成事件，去掉A，剩下BC,更新笔变量，转state3
//  3.2 BC不成笔，
//  3.2.1 BC类型不同，保留C，转state4
//  3.2.2 BC类型相同，按同类合并规则处理BC
//  3.2.2.1如果保留C，更新笔端点，转state3
//  3.2.2.2如果保留B，抛弃C，转state3

// state 4
// +---+-4-+---+                    +---+-4-+---+       +---3---+
// | A | B | C |<-----D     =====>  | A | B |C/D|   or  | A |B/D|
// +---+---+---+                    +---+---+---+       +---+---+
// 前提 AB成笔且BC类型不同且BC不成笔
// 4.1 CD同类型-----按同类合并规则处理CD
// 4.1.1 如果保留D，要检测BD是否成笔
// 4.1.1.1如果BD成笔，AB笔完成，emit笔完成事件，去掉A，剩下BD,更新笔变量，转state3
// 4.1.1.2如果不成笔，转 state4
// 4.1.2 如果保留C，抛弃D，转state4
// 4.2 CD不同类-----去掉C，按同类合并规则处理BD
// 4.2.1 如果保留B,转state3
// 4.2.2 如果保留D，更新笔端点，转state3

// 上述算法解决的99%的笔问题，但是还有一种情况，无法完美处理
// 例子:
// X--A--B--C--D---E
// 以X-A为向上笔为例，
// X--A 向上成笔 B--C向下成笔 C-D向上不成笔，但是D高于B，E低于C, DE成笔
// 按照规则X-A笔在BC成笔的已经确认，最终B-E成向下笔，但是B不是这一笔的最高点
// 按照完美成笔，应该X-D成向上笔，D-E成向下笔，这样笔的端点是中间K的最高最低点，完美符合笔忽略中间波动的要求
// 这种情况实际上是要修正已经确认完成的笔，与当下确认笔有冲突的大原则有冲突
// 按照缠论从A0(1分钟)开始做推笔，线段才是最基本的构件
// 非完美的笔对线段没有影响

#[derive(Debug, Clone, Copy)]
pub enum PenEvent<'a> {
    Complete(&'a Pen),
    New(&'a Pen),
    UpdateTo(&'a Pen, &'a Fractal),
}

// TODO:考虑一种特殊情况就是顶分型高点相等或者底分型低点相等
pub struct FractalQueue<'a> {
    window: RingBuffer<Fractal>,
    current_pen: Option<Pen>,
    store: Option<&'a PenStore>,
}

impl<'a> FractalQueue<'a> {
    pub fn new() -> Self {
        Self {
            window: RingBuffer::new(3),
            current_pen: None,
            store: None,
        }
    }

    pub fn set_store(&mut self, store: &'a PenStore) {
        self.store = Some(store);
    }

    fn emit(&self, event: PenEvent) {
        if let Some(store) = self.store {
            store.on_event(event);
        }
    }

    fn emit_pen_complete_event(&self, pen: &Pen) {
        self.emit(PenEvent::Complete(pen));
    }

    fn emit_pen_new_event(&self, pen: &Pen) {
        self.emit(PenEvent::New(pen));
    }

    fn emit_pen_update_event(&self, pen: &Pen, f: &Fractal) {
        self.emit(PenEvent::UpdateTo(pen, f));
    }

    fn pen_new(&self, from: &Fractal, to: &Fractal) -> Pen {
        let pen = Pen::new(from.clone(), to.clone());
        self.emit_pen_new_event(&pen);
        pen
    }

    fn pen_complete(&self, pen: &Pen) {
        self.emit_pen_complete_event(pen);
    }

    fn ab_pen_complete_bc_pen_new(&mut self) {
        debug_assert!(self.window.len() == 3);
        let pen = self.current_pen.as_mut().unwrap();
        pen.commit();
        let pen = self.current_pen.as_ref().unwrap();
        self.pen_complete(pen);
        self.window.pop_front();
        self.ab_new_pen();
    }

    fn ab_new_pen(&mut self) {
        debug_assert!(self.window.len() == 2);
        let from = self.window.get(0).unwrap();
        let to = self.window.get(1).unwrap();
        debug_assert!(_is_pen(from, to));
        let pen = self.pen_new(from, to);
        self.current_pen = Some(pen);
    }

    fn _is_pen(&self, start_index: usize) -> bool {
        debug_assert!(self.window.len() >= 2 + start_index);
        _is_pen(
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

    fn ab_pen_update(&mut self) {
        debug_assert!(self.window.len() == 2);
        let new_to = self.window.get(1).unwrap();
        self.current_pen.as_mut().unwrap().update_to(new_to.clone());
        debug_assert!(self.ab_is_pen());
        let pen = self.current_pen.as_ref().unwrap();
        self.emit_pen_update_event(pen, &new_to);
    }

    fn state0(&mut self, f: Fractal) {
        debug_assert!(self.window.len() == 0 && self.current_pen.is_none());
        self.window.push(f)
    }

    fn state1(&mut self, f: Fractal) {
        debug_assert!(self.window.len() == 1 && self.current_pen.is_none());
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
                self.ab_new_pen();
            }
        }
    }

    fn state2(&mut self, f: Fractal) {
        debug_assert!(!self.ab_is_pen());
        debug_assert!(self.current_pen.is_none());
        debug_assert!(self.window.len() == 2);

        let b = self.window.get(-1).unwrap();
        let bc_is_pen = _is_pen(b, &f);
        if bc_is_pen {
            // 2.1
            self.window.push(f);
            self.window.pop_front();
            self.ab_new_pen();
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
                        self.ab_new_pen();
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
    }

    fn state3(&mut self, f: Fractal) {
        debug_assert!(self.ab_is_pen());
        debug_assert!(self.current_pen.is_some());
        debug_assert!(self.window.len() == 2);

        if !self.ab_is_pen() {
            println!("a :{:?} \n b: {:?}", self.window.get(0), self.window.get(1));

            // for debug
            if let Some(p) = self.current_pen.as_ref() {
                let from = p.from();
                let to = p.to();
                println!("from: {:?} \n to:{:?}", from, to);
            }
        }

        let b = self.window.get(-1).unwrap();
        let bc_is_pen = _is_pen(b, &f);
        if bc_is_pen {
            // 3.1
            self.window.push(f);
            self.ab_pen_complete_bc_pen_new();
        } else {
            if b.is_same_type(&f) {
                let action = _merge_same_type(b, &f);
                if action == MergeAction::Replace {
                    // 3.2.2.1
                    self.window.pop_back();
                    self.window.push(f);
                    self.ab_pen_update();
                }
            } else {
                // 3.2.1
                self.window.push(f);
            }
        }
    }

    fn state4(&mut self, f: Fractal) {
        debug_assert!(self.ab_is_pen());
        debug_assert!(!self
            .window
            .get(-2)
            .unwrap()
            .is_same_type(self.window.get(-1).unwrap()));
        debug_assert!(!_is_pen(
            self.window.get(-2).unwrap(),
            self.window.get(-1).unwrap()
        ));
        debug_assert!(self.current_pen.is_some());
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
                    self.ab_pen_complete_bc_pen_new();
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
                self.window.push(f);
                self.ab_pen_update();
            }
        }
    }

    pub fn on_new_fractal(&mut self, f: Fractal) {
        // step1: valid fractal
        if let Some(last) = self.window.get(-1) {
            if !_is_valid_fractal(last, &f) {
                return;
            }
        }

        // step2: process fractal
        let len = self.window.len();
        let is_pen = self.current_pen.is_some();
        //let is_pen = self.window.len() >= 2 && self.ab_is_pen();

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

        let new_is_pen = self.window.len() >= 2 && self.ab_is_pen();
        if (new_is_pen && !self.current_pen.is_some())
            || (!new_is_pen && self.current_pen.is_some())
        {
            println!(
                "len: {} is_pen: {}, is_some: {}\na: {:?}\n b: {:?}",
                self.window.len(),
                new_is_pen,
                self.current_pen.is_some(),
                self.window.get(0),
                self.window.get(1)
            );

            if self.current_pen.is_some() {
                println!("curren_pen: {:?}", self.current_pen);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::bar::Bar;
    use crate::candle::Candle;
    use crate::fractal::Fractal;
    use crate::fractal_analyzer::FractalAnalyzer;
    use crate::fractal_util::_is_pen;
    use crate::pen_analyzer::FractalQueue;
    use crate::pen_store::PenStore;
    use crate::plot::draw_bar;
    use crate::time::Time;
    use chrono::prelude::*;
    use csv;

    #[test]
    fn test_is_pen() {
        let k1 = Candle::new(1052779380000, 1.15642, 1.15627);
        let k2 = Candle::new(1052779380000, 1.15645, 1.15634);
        let k3 = Candle::new(1052779500000, 1.15638, 1.1562);
        let f1 = Fractal::new(1118, k1, k2, k3);
        let k4 = Candle::new(1052780640000, 1.15604, 1.1559);
        let k5 = Candle::new(1052780820000, 1.15602, 1.15576);
        let k6 = Candle::new(1052780940000, 1.15624, 1.15599);
        let f2 = Fractal::new(1132, k4, k5, k6);
        let has_enough_distance = f1.has_enough_distance(&f2);
        assert!(has_enough_distance);
        println!(
            "f1.type = {:?} f1.range = {:?}, f2.type = {:?}, f2.range = {:?}",
            f1.fractal_type(),
            f1.range(),
            f2.fractal_type(),
            f2.range()
        );
        let is_pen = _is_pen(&f1, &f2);
        assert!(is_pen);
    }

    #[test]
    fn test_pen_detector() {
        let store = PenStore::new();
        let (bars, fractals) = load_fractal();
        println!("total fractals:{}", fractals.len());

        let mut fq = FractalQueue::new();
        fq.set_store(&store);

        for f in fractals {
            fq.on_new_fractal(f.clone());
        }

        println!("Pen Count = {}", store.count());
        //draw_bar(&bars);
    }

    fn load_fractal() -> (Vec<Bar>, Vec<Fractal>) {
        let mut fractals: Vec<Fractal> = Vec::new();
        let bars = load_bar2();
        let mut cq = FractalAnalyzer::new();
        for bar in &bars {
            if let Some(f) = cq.on_new_bar(bar) {
                fractals.push(f);
            }
        }
        (bars, fractals)
    }

    #[allow(dead_code)]
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
        println!(
            "bar count={} datetime size={}",
            bars.len(),
            size_of::<DateTime<Utc>>()
        );
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
