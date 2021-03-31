use crate::fractal::{Fractal, FractalType};
use crate::pen::{Pen, PenStatus, PenType};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FractalSharedKAction {
    Prev,
    Next,
    Both,
}

// 处理前后分型共用K的情况
// 规则一：如果两个分型类型不同，前分型为有效分型，后分型无效
// 规则二：如果两个分型类型相同，以高低点决定那个分型有效，如果高低点相同，两个分型都保留
fn _share_k_fractal_is_valid(f1: &Fractal, f2: &Fractal) -> FractalSharedKAction {
    if f1.fractal_type() != f2.fractal_type() {
        return FractalSharedKAction::Prev;
    }

    if f1.fractal_type() == FractalType::Top {
        if f2.high() == f1.high() {
            return FractalSharedKAction::Both;
        }

        if f2.high() < f1.high() {
            return FractalSharedKAction::Prev;
        } else {
            return FractalSharedKAction::Next;
        }
    } else {
        if f2.low() == f1.low() {
            return FractalSharedKAction::Both;
        }
        if f2.low() > f1.low() {
            return FractalSharedKAction::Prev;
        } else {
            return FractalSharedKAction::Next;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FractalContainAction {
    Prev,
    Next,
    None,
}
// 处理前后分型包含的情况
// 规则一：前分型包含后分型，后分型为无效分型
// 规则二：后分型包含前分型，前分型为无效分型
fn _fractal_contain_is_valid(f1: &Fractal, f2: &Fractal) -> FractalContainAction {
    let f1_high = f1.high();
    let f1_low = f1.low();
    let f2_high = f2.high();
    let f2_low = f2.low();

    let f1_contain_f2 = f1_high >= f2_high && f1_low <= f2_low;
    let f2_contain_f1 = f2_high >= f1_high && f2_low <= f1_low;

    if f1_contain_f2 {
        return FractalContainAction::Prev;
    }

    if f2_contain_f1 {
        return FractalContainAction::Next;
    }

    FractalContainAction::None
}

#[derive(Debug)]
pub struct FractalQueue {
    queue: Vec<Fractal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TwoFractalTypeEnum {
    Top_Top,
    Top_Bottom,
    Bottom_Top,
    Bottom_Bottom,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TwoFractalPriceEnum {
    High_Low,
    Low_High,
}

impl FractalQueue {
    pub fn new(queue: Vec<Fractal>) -> Self {
        Self { queue }
    }

    fn pen(f1: &Fractal, f2: &Fractal) -> Option<Pen> {
        let distance_is_enouge = f1.distance(f2) >= 4;
        let last_type = f1.fractal_type();
        let current_type = f2.fractal_type();

        match (distance_is_enouge, last_type, current_type) {
            (true, FractalType::Top, FractalType::Bottom) => {
                if f1.high() > f2.high() {
                    Some(Pen::new(f1.clone(), f2.clone(), PenType::Down))
                } else {
                    None
                }
            }
            (true, FractalType::Bottom, FractalType::Top) => {
                if f1.high() < f2.high() {
                    Some(Pen::new(f1.clone(), f2.clone(), PenType::Up))
                } else {
                    None
                }
            }
            (_, FractalType::Top, FractalType::Top) => {}
            (_, FractalType::Bottom, FractalType::Bottom) => {}
            (_, _, _) => None,
        }
    }

    fn is_valid_fractal(&mut self, f: &Fractal) -> bool {
        // step 1. 检测正确的分型
        // 1.1 共用K线情况
        let f1 = self.queue.last().unwrap();
        if f1.distance(&f) <= 2 {
            let shared_k_actio = _share_k_fractal_is_valid(f1, &f);
            match shared_k_actio {
                FractalSharedKAction::Prev => false,
                FractalSharedKAction::Next => {
                    self.queue.pop();
                    true
                }
                FractalSharedKAction::Both => true,
            }
        }
        // 1.2 前后分型包含情况
        else {
            let f1 = self.queue.last().unwrap();
            let contain_action = _fractal_contain_is_valid(f1, &f);
            match contain_action {
                FractalContainAction::Prev => false,
                FractalContainAction::Next => {
                    self.queue.pop();
                    true
                }
                _ => true,
            }
        }
    }

    pub fn on_new_fractal(&mut self, f: Fractal) {
        if self.queue.is_empty() {
            self.queue.push(f);
            return;
        }

        // step1: 检测分型是否标准
        let is_valid = self.is_valid_fractal(&f);
        if !is_valid {
            return;
        }

        // step2: 检查前后分型关系
        /*let last = self.queue.last().unwrap();
        match (last.fractal_type(), f.fractal_type()) {
            (FractalType::Top, FractalType::Top) => {
                if f.high() == last.high() {
                    self.queue.push(f);
                } else {
                    if f.high() > last.high() {
                        self.queue.pop();
                        self.queue.push(f);
                    }
                }
            }
            (FractalType::Bottom, FractalType::Bottom) => {
                if f.low() == last.low() {
                    self.queue.push(f);
                } else {
                    if f.low() < last.low() {
                        self.queue.pop();
                        self.queue.push(f);
                    }
                }
            }
            _ => {}
        }*/

        // 第一步检测标准分型后，
        // 第二步就是16种状态穷举
    }
}
