use std::collections::VecDeque;

use crate::fractal::{Fractal, FractalType};

#[derive(Debug, Clone)]
pub enum PenEvent {
    First(Fractal, Fractal),
    New(Fractal),
    UpdateTo(Fractal),
}

#[derive(Debug)]
pub struct PenDetector {
    window: VecDeque<Fractal>,
    has_pen: bool,
    prev_state: u32,
}

impl PenDetector {
    pub fn new() -> Self {
        Self {
            window: VecDeque::with_capacity(20),
            has_pen: false,
            prev_state: 0,
        }
    }

    pub fn get(&self, index: isize) -> Option<&Fractal> {
        if index >= 0 {
            self.window.get(index as usize)
        } else {
            self.window
                .get((self.window.len() as isize + index) as usize)
        }
    }

    fn _is_pen(&self, start_index: usize) -> bool {
        debug_assert!(self.window.len() >= 2 + start_index);
        is_pen_without_f2_contain_rule(
            self.window.get(start_index).unwrap(),
            self.window.get(start_index + 1).unwrap(),
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
        self.window.push_back(f);
        self.prev_state = 0;
        None
    }

    fn state1(&mut self, f: Fractal) -> Option<PenEvent> {
        debug_assert!(self.window.len() == 1 && !self.has_pen);
        self.prev_state = 1;
        let last = self.get(-1).unwrap();
        if last.is_same_type(&f) {
            // 1.1
            let action = merge_fractal(last, &f);
            if action == MergeAction::Replace {
                self.window.pop_back();
                self.window.push_back(f);
            }
        } else {
            // 1.2
            self.window.push_back(f);
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

        self.prev_state = 2;
        let b = self.get(-1).unwrap();
        let bc_is_pen = is_pen_without_f2_contain_rule(b, &f);
        if bc_is_pen {
            // 2.1
            self.window.push_back(f);
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
                let action = merge_fractal(b, &f);
                if action == MergeAction::Replace {
                    // 2.2.1.1
                    self.window.pop_back(); // pop b
                    self.window.push_back(f);
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
                let action = merge_fractal(a, &f);
                if action == MergeAction::Replace {
                    // 2.2.2.2
                    self.window.clear();
                    self.window.push_back(f);
                }
            }
        }
        None
    }

    fn state3(&mut self, f: Fractal) -> Option<PenEvent> {
        if !self.ab_is_pen() {
            println!(" prev_state = {}", self.prev_state);
        }
        debug_assert!(self.ab_is_pen());
        debug_assert!(self.has_pen);
        debug_assert!(self.window.len() == 2);

        self.prev_state = 3;
        let b = self.get(-1).unwrap();
        let bc_is_pen = is_pen_without_f2_contain_rule(b, &f);
        if bc_is_pen {
            // 3.1
            let c = f.clone();
            self.window.pop_front();
            self.window.push_back(f);
            //self.ab_pen_complete_bc_pen_new();
            return Some(PenEvent::New(c));
        } else {
            if b.is_same_type(&f) {
                let action = merge_fractal(b, &f);
                if action == MergeAction::Replace {
                    // 3.2.2.1
                    self.window.pop_back();
                    let c = f.clone();
                    self.window.push_back(f);
                    //self.ab_pen_update();
                    return Some(PenEvent::UpdateTo(c));
                }
            } else {
                // 3.2.1
                self.window.push_back(f);
            }
        }

        None
    }

    fn state4(&mut self, f: Fractal) -> Option<PenEvent> {
        debug_assert!(self.ab_is_pen());
        debug_assert!(!self.get(-2).unwrap().is_same_type(self.get(-1).unwrap()));
        debug_assert!(!is_pen_without_f2_contain_rule(
            self.get(-2).unwrap(),
            self.get(-1).unwrap()
        ));
        debug_assert!(self.has_pen);
        debug_assert!(self.window.len() == 3);

        self.prev_state = 4;
        let c = self.get(-1).unwrap();
        if c.is_same_type(&f) {
            // 4.1
            let action = merge_fractal(c, &f);
            if action == MergeAction::Replace {
                // 4.1.1
                self.window.pop_back();
                self.window.push_back(f);
                if self.bc_is_pen() {
                    // 4.1.1.1
                    self.window.pop_front();
                    return Some(PenEvent::New(self.get(-1).unwrap().clone()));
                }
            }
        } else {
            // 4.2
            self.window.pop_back();
            let b = self.get(-1).unwrap();
            let action = merge_fractal(b, &f);
            if action == MergeAction::Replace {
                // 4.2.2
                self.window.pop_back();
                let c = f.clone();
                self.window.push_back(f);
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

// util
pub fn is_pen_without_f2_contain_rule(f1: &Fractal, f2: &Fractal) -> bool {
    if f1.fractal_type() == FractalType::Top
        && f2.fractal_type() == FractalType::Bottom
        && f1.has_enough_distance(f2)
        && f2.lowest() < f1.lowest()
        && !f1.is_contain(f2)
    {
        return true;
    }

    if f1.fractal_type() == FractalType::Bottom
        && f2.fractal_type() == FractalType::Top
        && f1.has_enough_distance(f2)
        && f2.highest() > f1.highest()
        && !f1.is_contain(f2)
    {
        return true;
    }

    false
}

pub fn is_pen_with_f2_contain_rule(f1: &Fractal, f2: &Fractal) -> bool {
    if f1.fractal_type() == FractalType::Top
        && f2.fractal_type() == FractalType::Bottom
        && f1.has_enough_distance(f2)
        && f2.lowest() < f1.lowest()
        && !f1.is_contain(f2)
        && !f2.is_contain(f1)
    {
        return true;
    }

    if f1.fractal_type() == FractalType::Bottom
        && f2.fractal_type() == FractalType::Top
        && f1.has_enough_distance(f2)
        && f2.highest() > f1.highest()
        && !f1.is_contain(f2)
        && !f2.is_contain(f1)
    {
        return true;
    }

    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeAction {
    Keep,
    Replace,
    Both,
}

pub fn merge_fractal(f1: &Fractal, f2: &Fractal) -> MergeAction {
    debug_assert!(f1.fractal_type() == f2.fractal_type());
    if f1.fractal_type() == FractalType::Top {
        if f1.highest() > f2.highest() {
            MergeAction::Keep
        } else if f1.highest() == f2.highest() {
            MergeAction::Both
        } else {
            MergeAction::Replace
        }
    } else {
        if f1.lowest() < f2.lowest() {
            MergeAction::Keep
        } else if f1.lowest() == f2.lowest() {
            MergeAction::Both
        } else {
            MergeAction::Replace
        }
    }
}
