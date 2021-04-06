// 线段
use crate::pen::Pen;
#[derive(Debug, Clone)]
pub struct Segment {
    pens: Vec<Pen>,
}

impl Segment {
    pub fn new() -> Self {
        Self { pens: Vec::new() }
    }
}
