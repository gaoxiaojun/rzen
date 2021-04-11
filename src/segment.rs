// 线段
use crate::pen::Pen;

// 每3笔做一次判断 (分型个数 -1 ) % 3 == 0
// 从第6笔开始判断（分型7），以分型4为分界点
//

// 一 寻找第一个线段

#[derive(Debug)]
struct Segment {
    pens: Vec<Pen>,
}

impl Segment {
    fn new() -> Self {
        Self { pens: Vec::new() }
    }
}
