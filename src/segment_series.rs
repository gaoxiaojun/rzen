use crate::pen::{Pen, PenStatus, PenType};
use crate::segment::Segment;

#[derive(Debug, Clone)]
pub struct PenQueue {
    pens: Vec<Pen>,
    current: Option<Segment>,
}

impl PenQueue {
    fn new() -> Self {
        Self {
            pens: Vec::new(),
            current: None,
        }
    }

    pub fn on_new_pen(&mut self, pen: &Pen) {}
}
