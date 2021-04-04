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
                "from: {:?}--{}--{:?}, \nold_to: {:?}--{}--{:?}\nnew_to: {:?}--{}--{:?}",
                self.from.fractal_type(),
                self.from.index(),
                self.from.range(),
                self.to.fractal_type(),
                self.to.index(),
                self.to.range(),
                to.fractal_type(),
                to.index(),
                to.range()
            );
            println!(
                "{:?}\n{:?}\n{:?}",
                self.from.get_k1(),
                self.from.get_k2(),
                self.from.get_k3()
            );
            println!(
                "{:?}\n{:?}\n{:?}",
                self.to.get_k1(),
                self.to.get_k2(),
                self.to.get_k3()
            );
            println!("{:?}\n{:?}\n{:?}", to.get_k1(), to.get_k2(), to.get_k3());
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
