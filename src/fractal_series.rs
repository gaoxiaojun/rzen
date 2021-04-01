use std::thread::current;

use crate::fractal::{Fractal, FractalType};
use crate::fractal_util::{MergeAction, _is_pen, _merge_same_type};
use crate::pen::{Pen, PenType};
use crate::ringbuffer::RingBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TwoFractalTypeEnum {
    TopTop,
    TopBottom,
    BottomTop,
    BottomBottom,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TwoFractalPriceEnum {
    HighLow,
    LowHigh,
}

pub(crate) fn get_two_fractal_type(f1: &Fractal, f2: &Fractal) -> TwoFractalTypeEnum {
    match (f1.fractal_type(), f2.fractal_type()) {
        (FractalType::Top, FractalType::Top) => TwoFractalTypeEnum::TopTop,
        (FractalType::Top, FractalType::Bottom) => TwoFractalTypeEnum::TopBottom,
        (FractalType::Bottom, FractalType::Top) => TwoFractalTypeEnum::BottomTop,
        (FractalType::Bottom, FractalType::Bottom) => TwoFractalTypeEnum::BottomBottom,
    }
}

pub(crate) fn get_two_fractal_price(f1: &Fractal, f2: &Fractal) -> TwoFractalPriceEnum {
    if f1.high() >= f2.high() {
        TwoFractalPriceEnum::HighLow
    } else {
        TwoFractalPriceEnum::LowHigh
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PenAction {
    New(PenType),
    Continue,
}

pub(crate) fn _check_pen_action(f1: &Fractal, f2: &Fractal) -> Option<PenAction> {
    let two_type = get_two_fractal_type(f1, f2);
    let two_price = get_two_fractal_price(f1, f2);
    let two_distance = f1.has_enough_distance(f2);

    match (two_type, two_distance, two_price) {
        (TwoFractalTypeEnum::TopBottom, true, TwoFractalPriceEnum::HighLow) => {
            Some(PenAction::New(PenType::Down))
        }
        (TwoFractalTypeEnum::BottomTop, true, TwoFractalPriceEnum::LowHigh) => {
            Some(PenAction::New(PenType::Up))
        }
        (TwoFractalTypeEnum::TopTop, _, TwoFractalPriceEnum::LowHigh) => Some(PenAction::Continue),
        (TwoFractalTypeEnum::BottomBottom, _, TwoFractalPriceEnum::HighLow) => {
            Some(PenAction::Continue)
        }
        (_, _, _) => None,
    }
}

// 考虑一种特殊情况就是顶分型高点相等或者底分型低点相等
pub struct FractalQueue {
    window: RingBuffer<Fractal>,
    current_pen: Option<Pen>,
}

// case 1
// +---+
// | A |<-----B
// +---+
// AB同类型，合并AB，转case1
// AB不同类型
// 1.1 AB不成笔转case2
// 1.2 AB成笔转case3

// case 2
// +---+---+
// | A | B |<-----C
// +---+---+
// 前提：A B 不成笔
// 1.1 BC成笔 ---- 去掉A，保留BC，转case3
// 1.2 BC不成笔
// 1.2.1 BC同类，按同类规则处理决定保留B或者C, 转case2
// 1.2.2 BC不同类，

// case 3
// +---+---+
// | A | B |<-----C
// +---+---+
// 前提 1. AB成笔
//  BC成笔   --- AB笔完成，emit笔完成事件，去掉A，剩下BC,转case3
//  BC不成笔，保留C，转case4

// case 4
// +---+---+---+
// | A | B | C |<-----D
// +---+---+---+
// 前提 1. AB成笔 2. BC类型不同且BC不成笔
// case 2.1 CD同类型-----按同类规则处理决定保留C或者D,转case4
// case 2.2 CD不同类-----去掉C，BD按同类规则处理决定保留B或者D,转case3

impl FractalQueue {
    pub fn new() -> Self {
        Self {
            window: RingBuffer::new(3),
            current_pen: None,
        }
    }

    pub fn on_new_fractal(&mut self, f: Fractal) -> Option<Pen> {
        let len = self.window.len();

        match len {
            0 => {
                self.window.push(f);
                None
            }

            1 => {
                let last = self.window.get(-1).unwrap();
                if last.fractal_type() == f.fractal_type() {
                    let action = _merge_same_type(last, &f);
                    if action == MergeAction::Replace {
                        self.window.pop_back();
                        self.window.push(f);
                    }
                    return None;
                } else {
                    let is_pen = _is_pen(last, &f);
                    if is_pen {
                        self.current_pen = Some(Pen::new(last.clone(), f));
                    } else {
                    }
                }
                None
            }

            _ => {
                let start = self.window.get(-1).unwrap();
                match _check_pen_action(&start, &f) {
                    Some(PenAction::New(direction)) => {
                        let pen = Some(Pen::new(start.clone(), f.clone()));
                        self.window.push(f);
                        pen
                    }
                    Some(PenAction::Continue) => {
                        self.window.pop_front();
                        self.window.push(f);
                        None
                    }
                    _ => None,
                }
            }
        }
    }
}
