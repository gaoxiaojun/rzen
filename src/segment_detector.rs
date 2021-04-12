use std::collections::VecDeque;

use crate::{fractal::Fractal, ringbuffer::RingBuffer, sequence::Seq};

// 三笔重叠判断算法
// 三笔4个端点必须：
// 情况A：向上线段   1<=min(2,3,4) && 4>=max(1,2,3)
// 情况B：向下线段   1>=max(2,3,4) && 4<=min(1,2,3)

// 每3笔做一次判断 (分型个数 -1 ) % 3 == 0
// 从第6笔开始判断（分型7），以分型4为分界点
//

// 何为重置内部状态
// 假设某个点为线段结束点，判断第一种情况或者第二种情况是否符合
// 重置内部状态就是先设定假设点，然后根据假设点设置内部状态，用于后续的第一、第二种情况判断
// 当出现新高新低时，假设失败，以新高新低为假设点，开始重新假设
// 动作
// 1.设置线段假设结束点和对应的前高
// 2.case1_window.push(假设结束点前的特征序列)
// 3.设置方向
// 4.结束

// 一、寻找第一个线段
// state 0
// 前提是分型数量小于4(4个分型代表有3笔)
// fractals.push(),如果 (fractals.len() -1) % 3 == 0,转状态1或者 fractals.len() >3 转状态1
// state 1
// 前提：有三笔(意味着着有四个端点)
// 1.1 如果成线段,转状态2
// 1.2 不成线段，pop_front,转状态0
//
// 二、已有线段，找线段的终结点
// state2
// 1. 标定假设点，重置内部状态
// 2. 假设点之后第一笔
// 2. 假设点之后第一笔推送给windows1作为Candle2，第三笔也推送给window1作为Candle3，等三笔齐全，判断是否符合第一种情况，这里有个要考虑包含，从Candle2开始
// 3.

// 流程2
// 等笔数量超过4，通过最后一笔和前高的比较来分析是否属于第一种情况

#[derive(Debug, Clone)]
pub enum SegmentEvent {
    New(usize, usize),
    UpdateTo(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum SegmentDirection {
    Up,
    Down,
}

pub type FractalVecIndex = usize;

#[derive(Debug)]
pub struct SegmentDetector {
    segments: Vec<FractalVecIndex>,
    direction: Option<SegmentDirection>,
    last: Option<(FractalVecIndex, FractalVecIndex)>,
    current: FractalVecIndex,
    prev: FractalVecIndex,
    window1: RingBuffer<Fractal>,
    window2: VecDeque<Fractal>,
}

impl SegmentDetector {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            direction: None,
            last: None,
            current: 0,
            prev: 0,
            window1: RingBuffer::new(3),
            window2: VecDeque::new(),
        }
    }

    fn is_top_fractal(s1: &Seq, s2: &Seq, s3: &Seq) -> bool {
        if s1.high() < s2.high() && s2.high() > s3.high() {
            true
        } else {
            false
        }
    }

    fn is_bottom_fractal(s1: &Seq, s2: &Seq, s3: &Seq) -> bool {
        if s1.low() > s2.low() && s2.low() > s3.low() {
            true
        } else {
            false
        }
    }

    fn reset_state(&mut self) {}
    fn is_segment(
        p1: &Fractal,
        p2: &Fractal,
        p3: &Fractal,
        p4: &Fractal,
    ) -> Option<SegmentDirection> {
        // TODO: 如果p2 和p4等高怎么办？

        let direction_up =
            p1.price() < p2.price() && p2.price() > p3.price() && p3.price() < p4.price(); // ?? p2.price() <= p4.price()
        let direction_down =
            p1.price() > p2.price() && p2.price() < p3.price() && p3.price() > p4.price(); // ?? p2.price() >= p4.price()

        let direction = {
            match (direction_up, direction_down) {
                (true, false) => Some(SegmentDirection::Up),
                (false, true) => Some(SegmentDirection::Down),
                (_, _) => None,
            }
        };
        direction
    }

    fn state0(&self) {}

    pub fn on_pen_event(&mut self, pens: &Vec<Fractal>) -> Option<SegmentEvent> {
        // pens数组是先保存最新的笔，然后调用本方法，所以至少需要5个端点
        if pens.len() < 5 {
            return None;
        }

        // 为了防止今后修改上述逻辑
        let length = pens.len();
        let p1_index = length - 5;
        let p2_index = length - 4;
        let p3_index = length - 3;
        let p4_index = length - 2;

        if !self.direction.is_none() {
            let p1 = &pens[p1_index];
            let p2 = &pens[p2_index];
            let p3 = &pens[p3_index];
            let p4 = &pens[p4_index];

            self.direction = SegmentDetector::is_segment(p1, p2, p3, p4);

            if self.direction.is_some() {
                self.segments.push(p4_index);
                self.segments.push(p1_index);
                self.last = Some((p2_index, p3_index));
                return Some(SegmentEvent::New(p1_index, p4_index));
            } else {
                return None;
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pen_detector::PenDetector;
    use crate::test_util::tests::*;
    use crate::{fractal_detector::FractalDetector, pen_detector::PenEvent};

    #[test]
    fn test_segment_detector() {
        let bars = load_eurusd_2021();
        let mut fd = FractalDetector::new();
        let mut pd = PenDetector::new();
        let mut sd = SegmentDetector::new();
        let mut fvec: Vec<Fractal> = Vec::new();
        for bar in &bars {
            let fractal = fd.on_new_bar(bar);
            if let Some(f) = fractal {
                let pe = pd.on_new_fractal(f);
                if let Some(pen_event) = pe {
                    match pen_event {
                        PenEvent::First(a, b) => {
                            fvec.push(a);
                            fvec.push(b);
                        }
                        PenEvent::New(a) => {
                            fvec.push(a);
                            // 线段检测算法只关注已经完成的笔
                            // PenEvent::New代表原有笔已经终结
                            sd.on_pen_event(&fvec);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
