use crate::fractal::{Fractal, FractalType};

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
        self.to = to;
        self.status = PenStatus::Continue;
    }

    pub fn commit(&mut self) {
        self.status = PenStatus::Complete;
    }

    pub fn on_new_fractal(&mut self, f: Fractal) {
        // 前顶后低 and 距离足够 and 前高后低
    }
}

#[cfg(test)]
mod tests {
    //#[test]
}
