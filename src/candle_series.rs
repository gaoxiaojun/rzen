use crate::bar::Bar;
use crate::candle::Candle;
use crate::candle_util::{CandleWithIndex, _check_contain, _check_direction, _check_fractal};
use crate::fractal::Fractal;
use crate::ringbuffer::RingBuffer;

pub struct CandleQueue {
    window: RingBuffer<CandleWithIndex>,
    next_index: u64,
}

impl CandleQueue {
    pub fn new() -> Self {
        Self {
            window: RingBuffer::new(3),
            next_index: 0,
        }
    }

    fn add_candle(&mut self, candle: Candle) {
        let c = CandleWithIndex::new(self.next_index, candle);
        self.next_index += 1;
        self.window.push(c);
    }

    fn add_bar(&mut self, bar: &Bar) {
        let c = Candle::from_bar(bar);
        self.add_candle(c);
    }

    // 检查是否为顶底分型
    fn check_fractal(&self) -> Option<Fractal> {
        let k1 = self.window.get(-3).unwrap();
        let k2 = self.window.get(-2).unwrap();
        let k3 = self.window.get(-1).unwrap();

        _check_fractal(k1, k2, k3)
    }

    // 处理与当前bar的包含关系
    fn process_contain_relationship(&mut self, bar: &Bar) -> bool {
        // 队列中有至少两个经过包含处理的Candle
        debug_assert!(self.window.len() >= 2);
        let direction = {
            let k1 = self.window.get(-2).unwrap();
            let k2 = self.window.get(-1).unwrap();
            _check_direction(k1, k2)
        };

        let current = self.window.get_mut(-1).unwrap();

        _check_contain(direction, current, bar)
    }

    // 处理K线包含关系，更新内部缓冲区，检测分型
    pub fn on_new_bar(&mut self, bar: &Bar) -> Option<Fractal> {
        let len = self.window.len();
        debug_assert!(len <= 3);

        // 初始边界条件验证，前两个candle必须是非包含的
        match len {
            0 => {
                // 队列中没有K线
                self.add_bar(bar);
            }

            1 => {
                // 仅有一根K线
                // 起始开始的两K就存在包含关系，合理的处理方式是：
                // 1. 如果第一根K包含第二根K，直接忽略与第一根K存在包含的K线，直到遇到不包含的
                // 2. 如果第一根K包含在第二根K，忽略第一根K，从第二根K开始
                let last = self.window.get(-1).unwrap();
                let k1_include_k2 = last.candle.high >= bar.high && last.candle.low <= bar.low;
                let k2_include_k1 = last.candle.high <= bar.high && last.candle.low >= bar.low;
                if k1_include_k2 {
                    // 情况1，忽略当前Bar，直到遇到不包含的
                    return None;
                };

                if k2_include_k1 {
                    // 情况2，忽略K1,清空队列
                    self.window.clear();
                }
                // 当前Bar作为Candle放入队列
                self.add_bar(bar);
            }

            2 => {
                let merged = self.process_contain_relationship(bar);
                if !merged {
                    self.add_bar(bar);
                }
            }

            _ => {
                let merged = self.process_contain_relationship(bar);
                if !merged {
                    let result = self.check_fractal();
                    self.add_bar(bar);
                    return result;
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cq() {
        let b1 = Bar::new(1, 6.0, 8.0, 6.0, 8.0);
        let b2 = Bar::new(2, 9.0, 9.0, 7.0, 7.0);
        let b3 = Bar::new(3, 7.0, 7.0, 6.0, 6.0);
        let b4 = Bar::new(4, 6.0, 9.0, 6.0, 9.0);
        let b5 = Bar::new(5, 8.0, 11.0, 8.0, 11.0);

        let mut cq = CandleQueue::new();
        let f1 = cq.on_new_bar(&b1);
        let f2 = cq.on_new_bar(&b2);
        let f3 = cq.on_new_bar(&b3);
        let f4 = cq.on_new_bar(&b4);
        let f5 = cq.on_new_bar(&b5);
        assert!(f1.is_none());
        assert!(f2.is_none());
        assert!(f3.is_none());
        assert!(f4.is_none());
        assert!(f5.is_some());
        let f = f5.unwrap();
        let k1 = f.get_k1();
        let k2 = f.get_k2();
        let k3 = f.get_k3();
        assert!(k1.high == 8.0 && k1.low == 6.0);
        assert!(k2.high == 9.0 && k2.low == 7.0);
        assert!(k3.high == 7.0 && k3.low == 6.0);
    }
}
