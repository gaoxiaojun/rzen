use crate::fractal::{Fractal, FractalType};
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
}

impl FractalQueue {
    pub fn new() -> Self {
        Self {
            window: RingBuffer::new(2),
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
                self.window.push(f);
                let start = self.window.get(-2).unwrap();
                let end = self.window.get(-1).unwrap();
                match _check_pen_action(&start, &end) {
                    Some(PenAction::New(direction)) => {
                        Some(Pen::new(start.clone(), end.clone(), direction))
                    }
                    _ => None,
                }
            }

            _ => {
                let start = self.window.get(-1).unwrap();
                match _check_pen_action(&start, &f) {
                    Some(PenAction::New(direction)) => {
                        let pen = Some(Pen::new(start.clone(), f.clone(), direction));
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
