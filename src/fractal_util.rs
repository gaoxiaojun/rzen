use crate::fractal::{Fractal, FractalType};

/*
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FractalSharedKAction {
    Prev,
    Next,
    Both,
}

// 处理前后分型共用K的情况
// 规则一：如果两个分型类型不同，前分型为有效分型，后分型无效
// 规则二：如果两个分型类型相同，以高低点决定那个分型有效，如果高低点相同，两个分型都保留
pub fn _share_k_fractal_is_valid(f1: &Fractal, f2: &Fractal) -> FractalSharedKAction {
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
*/

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

pub fn _is_pen(f1: &Fractal, f2: &Fractal) -> bool {
    if f1.fractal_type() == FractalType::Top
        && f2.fractal_type() == FractalType::Bottom
        && f1.has_enough_distance(f2)
        && f1.low() > f2.high()
    {
        return true;
    }

    if f1.fractal_type() == FractalType::Bottom
        && f2.fractal_type() == FractalType::Top
        && f1.has_enough_distance(f2)
        && f1.high() < f2.low()
    {
        return true;
    }

    false
}

pub fn _is_valid_fractal(f1: &Fractal, f2: &Fractal) -> bool {
    // 1.1 共享K线分析，后分型无效
    if f1.distance(f2) < 3 && !f1.is_same_type(f2) {
        // 共享K线分型，
        return false;
    }

    // 1.2 包含关系分析，无效
    let f1_contain_f2 = f1.high() >= f2.high() && f1.low() <= f2.low();
    if f1_contain_f2 {
        return false;
    }

    true
}
