// 线段
use crate::pen::Pen;

#[derive(Debug)]
struct Segment {
    pens: Vec<Pen>,
}

impl Segment {
    fn new() -> Self {
        Self { pens: Vec::new() }
    }
}
