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
struct Pen {
    from: Fractal,
    to: Fractal,
    ptype: PenType,
    status: PenStatus,
}

impl Pen {
    pub fn new(from: Fractal, to: Fractal, ptype: PenType) -> Self {
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
}

#[cfg(test)]
mod tests {
    //#[test]
}
