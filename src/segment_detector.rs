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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationReson {
    // 对应线段破坏的第一种情况
    CASE1,
    // 对应线段破坏的第二种情况且同时确认第二线段也成立，即window2也构成了无gap的分型
    CASE21,
    // 对应线段破坏的第二种情况但是不确认第二线段也成立，即window2也构成了有gap的分型
    CASE22,
}

#[derive(Debug, Clone)]
pub enum SegmentEvent {
    New(Fractal, Fractal),
    New2(Fractal, Fractal, Fractal),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentDirection {
    Up,
    Down,
}

pub type FractalVecIndex = usize;

#[derive(Debug)]
pub struct SegmentDetector {
    fractals: VecDeque<Fractal>,
    direction: Option<SegmentDirection>,

    // 线段的起点
    start_point: FractalVecIndex,
    // 假设的线段终结点
    current: FractalVecIndex,
    // 对应假设终结点的前高(低)点，用于特征分型第一元素的计算
    prev: FractalVecIndex,

    // 对应线段终结第一种情况，保存3个分型判断即可
    window1: RingBuffer<Seq>,

    // 对应线段终结第二种情况，
    window2: RingBuffer<Seq>,
}

impl SegmentDetector {
    pub fn new() -> Self {
        Self {
            fractals: VecDeque::new(),
            direction: None,
            start_point: 0,
            current: 0,
            prev: 0,
            window1: RingBuffer::new(3),
            window2: RingBuffer::new(3),
        }
    }

    fn reset_state(&mut self, p2: usize, p4: usize) {
        debug_assert!(p4 - p2 >= 2);
        self.current = p4;
        self.prev = p2;
        self.window1.clear();
        self.window2.clear();
        let seq = self.merge_seq(p2, p4, self.merge_direction());
        self.window1.push(seq);
    }

    fn flip_direction(&mut self) {
        match self.direction.unwrap() {
            SegmentDirection::Up => self.direction = Some(SegmentDirection::Down),
            SegmentDirection::Down => self.direction = Some(SegmentDirection::Up),
        }
    }

    fn emit_new_event(&self, start: usize, end: usize) -> SegmentEvent {
        let f1 = self.fractals.get(start).unwrap();
        let f2 = self.fractals.get(end).unwrap();
        SegmentEvent::New(f1.clone(), f2.clone())
    }

    fn emit_new2_event(&self, start: usize, end: usize, next_end: usize) -> SegmentEvent {
        let f1 = self.fractals.get(start).unwrap();
        let f2 = self.fractals.get(end).unwrap();
        let f3 = self.fractals.get(next_end).unwrap();
        SegmentEvent::New2(f1.clone(), f2.clone(), f3.clone())
    }

    fn flip(&mut self, reason: Option<TerminationReson>) -> Option<SegmentEvent> {
        match reason {
            None => None,
            CASE1 => {
                let event = self.emit_new_event(self.start_point, self.current);
                self.start_point = self.current;
                self.reset_state(self.current + 1, self.fractals.len() - 1);
                self.flip_direction();
                Some(event)
            }
            CASE21 => {
                let c2 = self.window2.get(-2).unwrap();
                let c3 = self.window2.get(-1).unwrap();
                let current2 = c2.from_index();
                let event = self.emit_new2_event(self.start_point, self.current, current2);
                self.start_point = c2.from_index();
                self.reset_state(c2.to_index(), c3.to_index());
                Some(event)
            }
            CASE22 => {
                let event = self.emit_new_event(self.start_point, self.current);
                let c1 = self.window2.get(-3).unwrap();
                let c2 = self.window2.get(-2).unwrap();
                self.start_point = self.current;
                self.reset_state(c1.from_index(), c2.from_index());
                self.flip_direction();
                Some(event)
            }
        }
    }

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

    // 特征序列进行标准化
    // [start, end) end不包含在里面
    fn merge_seq(&self, start: usize, end: usize, dir: MergeDirection) -> Seq {
        let mut from_index = start;
        let from = self.get(from_index as isize).unwrap();
        let to = self.get((from_index + 1) as isize).unwrap();
        let mut seq = Seq::new(
            from_index,
            from.time(),
            from.price(),
            from_index + 1,
            to.time(),
            to.price(),
        );
        while from_index + 2 < end {
            from_index += 2;
            let new_from = self.get(from_index as isize).unwrap();
            let new_to = self.get((from_index + 1) as isize).unwrap();
            let new_seq = Seq::new(
                from_index,
                new_from.time(),
                new_from.price(),
                from_index + 1,
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

    fn add_seq_on_window1(&mut self, dir: MergeDirection) {
        debug_assert!(self.window1.len() > 0);
        let length = self.window1.len();
        let last = self.get(-1).unwrap();
        let prev = self.get(-2).unwrap();
        let seq = Seq::new(
            length - 2,
            prev.time(),
            prev.price(),
            length - 1,
            last.time(),
            last.price(),
        );
        let length = self.window1.len();
        if length > 1 {
            let s = self.window1.get_mut(-1).unwrap();
            let is_merged = s.merge(&seq, dir);
            if !is_merged {
                self.window1.push(seq);
            }
        }
    }

    fn add_seg_on_window2(&mut self, dir: MergeDirection) {
        let length = self.window1.len();
        let last = self.get(-1).unwrap();
        let prev = self.get(-2).unwrap();
        let seq = Seq::new(
            length - 2,
            prev.time(),
            prev.price(),
            length - 1,
            last.time(),
            last.price(),
        );
        let length = self.window2.len();
        if length > 0 {
            let s = self.window2.get_mut(-1).unwrap();
            let is_merged = s.merge(&seq, dir);
            if !is_merged {
                self.window2.push(seq);
            }
        } else {
            self.window2.push(seq);
        }
    }

    fn check_window1_fractal(&self) -> bool {
        debug_assert!(self.window1.len() == 3);
        let s1 = self.window1.get(-3).unwrap();
        let s2 = self.window1.get(-2).unwrap();
        let s3 = self.window1.get(-1).unwrap();
        let dir = self.direction.unwrap();
        match dir {
            SegmentDirection::Up => Seq::is_top_fractal(s1, s2, s3),
            SegmentDirection::Down => Seq::is_bottom_fractal(s1, s2, s3),
        }
    }

    fn check_window1_has_gap(&self) -> bool {
        debug_assert!(self.window1.len() >= 2);
        let s1 = self.window1.get(0).unwrap();
        let s2 = self.window1.get(1).unwrap();
        let dir = self.direction.unwrap();
        match dir {
            SegmentDirection::Up => s1.high() < s2.low(),
            SegmentDirection::Down => s1.low() > s2.high(),
        }
    }

    fn check_window2_has_gap(&self) -> bool {
        debug_assert!(self.window2.len() >= 2);
        let s1 = self.window2.get(0).unwrap();
        let s2 = self.window2.get(1).unwrap();
        let dir = self.direction.unwrap();
        match dir {
            SegmentDirection::Up => s1.low() > s2.high(),
            SegmentDirection::Down => s1.high() < s2.low(),
        }
    }

    fn check_termination_case_1(&self) -> bool {
        debug_assert!(self.check_window1_fractal());
        let s1 = self.window1.get(-3).unwrap();
        let s2 = self.window1.get(-2).unwrap();
        let dir = self.direction.unwrap();
        match dir {
            SegmentDirection::Up => s1.high() >= s2.low(),
            SegmentDirection::Down => s1.low() <= s2.high(),
        }
    }

    fn check_window2_fractal(&self) -> bool {
        debug_assert!(self.window2.len() >= 3);
        let s1 = self.window2.get(-3).unwrap();
        let s2 = self.window2.get(-2).unwrap();
        let s3 = self.window2.get(-1).unwrap();
        let dir = self.direction.unwrap();
        match dir {
            SegmentDirection::Up => Seq::is_bottom_fractal(s1, s2, s3),
            SegmentDirection::Down => Seq::is_top_fractal(s1, s2, s3),
        }
    }

    fn check_termination(&self) -> Option<TerminationReson> {
        let is_case1 = self.check_window1_fractal() && self.check_termination_case_1();
        if is_case1 {
            return Some(TerminationReson::CASE1);
        }

        let is_case2 = self.check_window1_has_gap() && self.check_window2_fractal();
        if is_case2 {
            if self.check_window2_has_gap() {
                return Some(TerminationReson::CASE21);
            } else {
                return Some(TerminationReson::CASE22);
            }
        }
        None
    }

    fn on_new_pen(&mut self) -> Option<SegmentEvent> {
        // 每当新的一笔确认，在假设点前后，填充情况一及情况二的序列(window1, window2)
        //debug_assert!(self.window1.len() == 1);
        debug_assert!(self.fractals.len() > self.current);
        debug_assert!(self.direction.is_some());

        // 具体过程如下：
        // 与线段当前方向相反的笔合并处理后放入window1
        // 与线段当前方向相同的笔合并处理后放入window2
        // 当window1的数量达到3，看是否是case1，如果是case1，形成顶分型，线段结束
        // 如果不是，。。。。
        // 当window2的数量达到3，如果是底分型，线段1结束，
        let segment_dir = self.direction.unwrap();
        let last = self.get(-1).unwrap();
        let prev = self.get(-2).unwrap();
        let is_same_direction_up =
            segment_dir == SegmentDirection::Up && last.price() > prev.price();
        let is_same_direction_down =
            segment_dir == SegmentDirection::Down && last.price() < prev.price();
        let is_same_direction = is_same_direction_up || is_same_direction_down;
        if is_same_direction {
            self.add_seg_on_window2(SegmentDetector::get_flip_merge_direction(segment_dir));
        } else {
            self.add_seq_on_window1(SegmentDetector::get_merge_direction(segment_dir));
        }

        let reason = self.check_termination();
        self.flip(reason)
    }

    fn find_first_segment(&mut self) -> Option<SegmentEvent> {
        // 查找第一个线段
        // 判断方式通过4个分型的滑动窗口来判断
        // 这里没有包含全部的情况，例如1-2-3-4-5-6等多个笔组成线段，
        // TODO: 按照缠论前3笔重叠构成线段，因此如果前三笔没有构成4点高于2点是不是也算线段？
        // 如果算，这里的第一笔检测算法就要更新，
        debug_assert!(self.direction.is_none());
        let p1 = self.get(-4).unwrap();
        let p2 = self.get(-3).unwrap();
        let p3 = self.get(-2).unwrap();
        let p4 = self.get(-1).unwrap();

        self.direction = SegmentDetector::is_first_segment(p1, p2, p3, p4);

        self.start_point = self.fractals.len() - 4;

        if self.direction.is_some() {
            let len = self.fractals.len();
            self.reset_state(len - 3, len - 1);
            let start = self.get(-4).unwrap().clone();
            let end = self.get(-1).unwrap().clone();
            Some(SegmentEvent::New(start, end))
        } else {
            self.fractals.pop_front();
            None
        }
    }

    fn process_normal_segment(&mut self) -> Option<SegmentEvent> {
        // 开始常规线段处理
        debug_assert!(self.direction.is_some());
        let direction = self.direction.unwrap();
        let last_point = self.get(-1).unwrap();

        let new_higher = direction == SegmentDirection::Up
            && last_point.price() > self.fractals[self.current].price();

        let new_lower = direction == SegmentDirection::Down
            && last_point.price() < self.fractals[self.current].price();

        let new_higher_or_lower = new_higher || new_lower;

        if new_higher_or_lower {
            // 创新高或者新低，假设该点是线段终结点
            let new_assume_end_point = self.fractals.len() - 1;
            self.reset_state(self.current, new_assume_end_point);
            None
        } else {
            self.on_new_pen()
        }
    }

    pub fn process(&mut self) -> Option<SegmentEvent> {
        // 调用本方法，所以至少需要4个分型端点
        if self.fractals.len() < 5 {
            return None;
        }

        if self.direction.is_none() {
            self.find_first_segment()
        } else {
            self.process_normal_segment()
        }
    }

    //
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

    // helper
    fn get(&self, index: isize) -> Option<&Fractal> {
        if index >= 0 {
            self.fractals.get(index as usize)
        } else {
            self.fractals
                .get((self.fractals.len() as isize + index) as usize)
        }
    }

    fn pop_segment(&mut self, end: usize) {
        debug_assert!(end < self.fractals.len());
        self.fractals.drain(0..end - 1);
    }

    fn merge_direction(&self) -> MergeDirection {
        debug_assert!(self.direction.is_some());
        let direction = self.direction.unwrap();
        SegmentDetector::get_merge_direction(direction)
    }

    fn get_merge_direction(direction: SegmentDirection) -> MergeDirection {
        match direction {
            SegmentDirection::Down => MergeDirection::Down,
            SegmentDirection::Up => MergeDirection::Up,
        }
    }

    fn get_flip_merge_direction(direction: SegmentDirection) -> MergeDirection {
        match direction {
            SegmentDirection::Down => MergeDirection::Up,
            SegmentDirection::Up => MergeDirection::Down,
        }
    }
}
