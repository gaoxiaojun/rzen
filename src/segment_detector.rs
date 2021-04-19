use std::collections::VecDeque;

use crate::{
    fractal::Fractal,
    pen_detector::PenEvent,
    ringbuffer::RingBuffer,
    sequence::{MergeDirection, Seq},
};

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
    New(Fractal, Fractal),
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
    fractals: VecDeque<Fractal>,
    segments: Vec<FractalVecIndex>,
    direction: Option<SegmentDirection>,
    last: Option<(FractalVecIndex, FractalVecIndex)>,

    // 假设的线段终结点
    current: FractalVecIndex,
    // 对应假设终结点的前高(低)点，用于特征分型第一元素的计算
    prev: FractalVecIndex,

    // 对应线段终结第一种情况，保存3个分型判断即可
    window1: RingBuffer<Seq>,

    // 对应线段终结第二种情况，
    window2: VecDeque<Seq>,
}

impl SegmentDetector {
    pub fn new() -> Self {
        Self {
            fractals: VecDeque::new(),
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

    fn reset_state(&mut self, p2: usize, p4: usize) {
        self.current = p4;
        self.prev = p2;
    }

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

    fn is_segment_termination(&mut self) {}
    // 判断第一个线段的时候，条件约束较严格
    fn is_first_segment(
        p1: &Fractal,
        p2: &Fractal,
        p3: &Fractal,
        p4: &Fractal,
    ) -> Option<SegmentDirection> {
        let direction_up = p1.price() < p2.price()
            && p2.price() > p3.price()
            && p3.price() > p1.price()
            && p4.price() > p3.price()
            && p4.price() > p2.price();
        let direction_down = p1.price() > p2.price()
            && p2.price() < p3.price()
            && p3.price() < p1.price()
            && p4.price() < p3.price()
            && p4.price() < p2.price();

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

    fn merge_seq_up(&self, start: usize, end: usize, dir: MergeDirection) -> Seq {
        debug_assert!(end - start >= 2);
        let mut fromIndex = start;
        let from = self.get(fromIndex as isize).unwrap();
        let to = self.get((fromIndex + 1) as isize).unwrap();
        let mut seq = Seq::new(from.time(), from.price(), to.time(), to.price());
        while fromIndex + 2 < end {
            fromIndex += 2;
            let new_from = self.get(fromIndex as isize).unwrap();
            let new_to = self.get((fromIndex + 1) as isize).unwrap();
            let new_seq = Seq::new(
                new_from.time(),
                new_from.price(),
                new_to.time(),
                new_to.price(),
            );
            let is_merged = seq.merge(&new_seq, dir);
            if !is_merged {
                break;
            }
        }
        seq
    }
    // [start, end) end不包含在里面
    fn merge_seq1(&self, start: usize, end: usize) -> Seq {
        debug_assert!(end - start >= 2);
        let direction = self.direction.unwrap();
        let dir = match direction {
            SegmentDirection::Down => MergeDirection::Down,
            SegmentDirection::Up => MergeDirection::Up,
        };

        let mut fromIndex = start;
        let from = self.get(fromIndex as isize).unwrap();
        let to = self.get((fromIndex + 1) as isize).unwrap();
        let mut seq = Seq::new(from.time(), from.price(), to.time(), to.price());
        while fromIndex + 2 < end {
            fromIndex += 2;
            let new_from = self.get(fromIndex as isize).unwrap();
            let new_to = self.get((fromIndex + 1) as isize).unwrap();
            let new_seq = Seq::new(
                new_from.time(),
                new_from.price(),
                new_to.time(),
                new_to.price(),
            );
            let is_merged = seq.merge(&new_seq, dir);
            if !is_merged {
                break;
            }
        }
        seq
    }

    fn on_lower_low(&mut self) {
        let new_end_point = self.fractals.len() - 1;
        self.reset_state(self.current, new_end_point);
        self.window1.clear();
        let seq = self.merge_seq1(self.current, new_end_point);
        self.window1.push(seq);
    }

    fn on_higher_high(&mut self) {
        let new_end_point = self.fractals.len() - 1;
        self.reset_state(self.current, new_end_point);
        self.window1.clear();
        let seq = self.merge_seq1(self.current, new_end_point);
        self.window1.push(seq);
    }

    pub fn on_pen_event(&mut self, pen_event: PenEvent) -> Option<SegmentEvent> {
        match pen_event {
            PenEvent::First(a, b) => {
                self.fractals.push_back(a);
                self.fractals.push_back(b);
                None
            }

            PenEvent::New(a) => {
                // PenEvent::New代表原有笔已经终结,但是该新笔后续还可能延伸
                // 线段检测算法只关注已经完成的笔
                let event = self.process();
                self.fractals.push_back(a);
                event
            }

            PenEvent::UpdateTo(a) => {
                self.fractals.pop_back();
                self.fractals.push_back(a);
                None
            }
        }
    }

    fn get(&self, index: isize) -> Option<&Fractal> {
        if index >= 0 {
            self.fractals.get(index as usize)
        } else {
            self.fractals
                .get((self.fractals.len() as isize + index) as usize)
        }
    }

    fn get_index(&self, index: isize) -> usize {
        debug_assert!(
            (index > 0 && index < self.fractals.len() as isize)
                || (index < 0
                    && ((self.fractals.len() as isize + index) as usize) < self.fractals.len())
        );
        if index >= 0 {
            index as usize
        } else {
            (self.fractals.len() as isize + index) as usize
        }
    }

    fn on_new_pen(&mut self) {}

    fn process_first_segment(&mut self) -> Option<SegmentEvent> {
        // 判断第一个线段
        // 判断方式通过4个分型的滑动窗口来判断
        let p1 = self.get(-4).unwrap();
        let p2 = self.get(-3).unwrap();
        let p3 = self.get(-2).unwrap();
        let p4 = self.get(-1).unwrap();

        self.direction = SegmentDetector::is_first_segment(p1, p2, p3, p4);

        if self.direction.is_some() {
            let len = self.fractals.len();
            self.reset_state(len - 3, len - 1);
            let start = self.get(-4).unwrap().clone();
            let end = self.get(-1).unwrap().clone();
            Some(SegmentEvent::New(start, end))
        } else {
            // 不需要弹出无用分型
            // self.fractals.pop_front();
            None
        }
    }

    fn process_normal_segment(&mut self) -> Option<SegmentEvent> {
        // 开始常规线段处理
        debug_assert!(self.direction.is_some());
        let direction = self.direction.unwrap();
        let last_pen = self.get(-1).unwrap();
        let length = self.fractals.len();

        match direction {
            SegmentDirection::Up => {
                if last_pen.price() > self.fractals[self.current].price() {
                    // 创新高，假设该点是线段终结点
                    self.on_higher_high();
                } else {
                    self.on_new_pen();
                }
            }
            SegmentDirection::Down => {
                if last_pen.price() < self.fractals[self.current].price() {
                    // 创新低，假设该点是线段终结点
                    self.on_lower_low();
                } else {
                    self.on_new_pen();
                }
            }
        }

        None
    }

    pub fn process(&mut self) -> Option<SegmentEvent> {
        // pens数组是先保存最新的笔，然后调用本方法，所以至少需要4个端点
        if self.fractals.len() < 5 {
            return None;
        }

        if self.direction.is_none() {
            self.process_first_segment()
        } else {
            self.process_normal_segment()
        }
    }
}
