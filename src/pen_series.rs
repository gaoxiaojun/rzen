use crate::fractal::{Fractal, FractalType};
use crate::pen::{Pen, PenStatus, PenType};

fn pen_detect(currentPen: Option<&Pen>, currentFractal: Option<&Fractal>, newFractal: Fractal) {}

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
struct PenDetector {
    last_fractal: Option<Fractal>,
    current_pen: Option<Pen>,
}

impl PenDetector {
    fn new() -> Self {
        Self {
            last_fractal: None,
            current_pen: None,
        }
    }

    fn is_valid_fractal(&mut self, f: Fractal) {}
    pub fn on_new_fractal(&mut self, f: Fractal) -> Option<Pen> {
        None
    }
}

// 梳理下逻辑
// 1. 先确定是否是合适的分型
//    对比current和new，
//      1.1 如果current是None，current = new， 分型有效
//      1.2 共用K分型比较
//      1.3 包含分型比较，
//    