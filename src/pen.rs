use crate::fractal::{Fractal, FractalType};
use crate::fractal_util::_is_pen;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PenType {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PenStatus {
    New,
    Complete,
    Continue,
}

#[derive(Debug)]
pub struct Pen {
    from: Fractal,
    to: Fractal,
    ptype: PenType,
    status: PenStatus,
}

impl Pen {
    pub fn new(from: Fractal, to: Fractal) -> Self {
        debug_assert!(from.fractal_type() != to.fractal_type());
        let ptype = if from.fractal_type() == FractalType::Top {
            PenType::Down
        } else {
            PenType::Up
        };

        Self {
            from,
            to,
            ptype,
            status: PenStatus::New,
        }
    }

    pub fn update_to(&mut self, to: Fractal) {
        if !_is_pen(&self.from, &to) {
            println!(
                "from: {:?}, \nold_to: {:?}\n new_to: {:?}",
                self.from, self.to, to
            );
        }
        debug_assert!(_is_pen(self.from(), &to));
        self.to = to;
        self.status = PenStatus::Continue;
    }

    pub fn commit(&mut self) {
        self.status = PenStatus::Complete;
    }

    pub fn from(&self) -> &Fractal {
        &self.from
    }

    pub fn to(&self) -> &Fractal {
        &self.to
    }
}

#[cfg(test)]
mod tests {
    //#[test]
}
