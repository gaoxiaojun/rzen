use crate::fractal::{Fractal, FractalType};
use crate::ringbuffer::RingBuffer;



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

// TODO: 后分型包含前分型的情况需要处理
pub fn is_pen(f1: &Fractal, f2: &Fractal) -> bool {
    if f1.fractal_type() == FractalType::Top
        && f2.fractal_type() == FractalType::Bottom
        && f1.has_enough_distance(f2)
        && f2.lowest() < f1.lowest()
        && !f1.is_contain(f2)
    //&& !f2.is_contain(f1)
    {
        return true;
    }

    if f1.fractal_type() == FractalType::Bottom
        && f2.fractal_type() == FractalType::Top
        && f1.has_enough_distance(f2)
        && f2.highest() > f1.highest()
        && !f1.is_contain(f2)
    //&& !f2.is_contain(f1)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PenDirection {
    Up,
    Down,
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
            //self.window.pop_back();
            let b = self.window.get(-2).unwrap();
            let action = _merge_same_type(b, &f);
            if action == MergeAction::Replace {
                // 4.2.2
                self.window.pop_back();
                self.window.pop_back();
                let c = f.clone();
                self.window.push(f);
                return Some(PenEvent::UpdateTo(c));
            }
        }

        None
    }

    pub fn on_new_fractal(&mut self, f: Fractal) -> Option<PenEvent> {
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
    use crate::plot::*;
    use crate::test_util::tests::*;
    #[test]
    fn test_is_pen() {
        let k1 = Candle::new(1117, 1052779380000, 1.15642, 1.15642, 1.15627, 1.15627);
        let k2 = Candle::new(1118, 1052779380000, 1.15645, 1.15645, 1.15634, 1.15634);
        let k3 = Candle::new(1119, 1052779500000, 1.15638, 1.15638, 1.1562, 1.1562);
        let f1 = Fractal::new(k1, k2, k3);
        let k4 = Candle::new(1131, 1052780640000, 1.15604, 1.15604, 1.1559, 1.1559);
        let k5 = Candle::new(1132, 1052780820000, 1.15602, 1.15602, 1.15576, 1.15576);
        let k6 = Candle::new(1133, 1052780940000, 1.15624, 1.15624, 1.15599, 1.15599);
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
    fn test_pen_detector_with_candle() {
        let (bars, candles, fractals) = load_fractal();
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
        let segments: Vec<Fractal> = Vec::new();
        draw_bar_tradingview(&candles, &pens, &&segments);
    }

    fn load_fractal() -> (Vec<Bar>, Vec<Bar>, Vec<Fractal>) {
        let mut fractals: Vec<Fractal> = Vec::new();
        let bars = load_eurusd_2021();

        //let mut all_candles: Vec<Bar> = Vec::new();
        //let observer = |bar: &Bar| all_candles.push(bar.clone());

        let mut cq = FractalDetector::with_candles();

        for bar in &bars {
            if let Some(f) = cq.on_new_bar(bar) {
                fractals.push(f);
            }
        }

        (bars, cq.get_candles().unwrap().clone(), fractals)
    }
}
