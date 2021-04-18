use crate::bar::Bar;
use crate::candle::Candle;
use crate::fractal::Fractal;
use crate::ringbuffer::RingBuffer;

pub struct FractalDetector {
    window: RingBuffer<Candle>,
    next_index: u64,
    candles: Option<Vec<Bar>>,
}

impl FractalDetector {
    pub fn new() -> Self {
        Self {
            window: RingBuffer::new(3),
            next_index: 0,
            candles: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_candles() -> Self {
        Self {
            window: RingBuffer::new(3),
            next_index: 0,
            candles: Some(Vec::new()),
        }
    }

    #[allow(dead_code)]
    pub fn get_candles(&self) -> Option<&Vec<Bar>> {
        self.candles.as_ref()
    }

    fn notify(&mut self) {
        if let Some(container) = self.candles.as_mut() {
            if self.window.len() > 0 {
                let last = self.window.get(-1).unwrap();
                container.push(last.bar.clone());
            }
        }
    }

    // 当确定当前Bar与前Candle不存在合并关系的时候，该方法被调用
    fn add_candle(&mut self, bar: &Bar) {
        self.notify();
        let c = Candle::from_bar(self.next_index, bar);
        self.next_index += 1;
        self.window.push(c);
    }

    // 检查是否为顶底分型
    fn check_fractal(&self) -> Option<Fractal> {
        let k1 = self.window.get(-3).unwrap();
        let k2 = self.window.get(-2).unwrap();
        let k3 = self.window.get(-1).unwrap();

        Fractal::check_fractal(k1, k2, k3)
    }

    // 处理与当前bar的包含关系
    fn process_contain_relationship(&mut self, bar: &Bar) -> bool {
        // 队列中有至少两个经过包含处理的Candle
        debug_assert!(self.window.len() >= 2);
        let direction = {
            let k1 = self.window.get(-2).unwrap();
            let k2 = self.window.get(-1).unwrap();
            Candle::check_direction(k1, k2)
        };

        let current = self.window.get_mut(-1).unwrap();

        Candle::merge(direction, current, bar)
    }

    // 处理K线包含关系，更新内部缓冲区，检测分型
    pub fn on_new_bar(&mut self, bar: &Bar) -> Option<Fractal> {
        let len = self.window.len();
        debug_assert!(len <= 3);

        // 初始边界条件验证，前两个candle必须是非包含的
        match len {
            0 => {
                // 队列中没有K线
                self.add_candle(bar);
            }

            1 => {
                // 仅有一根K线
                // 起始开始的两K就存在包含关系，合理的处理方式是：
                // 1. 如果第一根K包含第二根K，直接忽略与第一根K存在包含的K线，直到遇到不包含的
                // 2. 如果第一根K包含在第二根K，忽略第一根K，从第二根K开始
                let last = self.window.get(-1).unwrap();
                let k1_include_k2 = last.bar.high >= bar.high && last.bar.low <= bar.low;
                let k2_include_k1 = last.bar.high <= bar.high && last.bar.low >= bar.low;
                if k1_include_k2 {
                    // 情况1，忽略当前Bar，直到遇到不包含的
                    return None;
                };

                if k2_include_k1 {
                    // 情况2，忽略K1,清空队列
                    self.window.clear();
                }
                // 当前Bar作为Candle放入队列
                self.add_candle(bar);
            }

            2 => {
                let merged = self.process_contain_relationship(bar);
                if !merged {
                    self.add_candle(bar);
                }
            }

            _ => {
                let merged = self.process_contain_relationship(bar);
                if !merged {
                    let result = self.check_fractal();
                    self.add_candle(bar);
                    return result;
                }
            }
        }
        None
    }
}

impl std::fmt::Debug for FractalDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FractalDetector")
            .field("window", &self.window)
            .field("next_index", &self.next_index)
            .finish()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::candle::Direction;
    use crate::fractal::FractalType;
    use crate::plot::*;
    use crate::test_util::tests::*;
    #[test]
    fn test_detector() {
        let b1 = Bar::new(1, 6.0, 8.0, 6.0, 8.0);
        let b2 = Bar::new(2, 9.0, 9.0, 7.0, 7.0);
        let b3 = Bar::new(3, 7.0, 7.0, 6.0, 6.0);
        let b4 = Bar::new(4, 6.0, 9.0, 6.0, 9.0);
        let b5 = Bar::new(5, 8.0, 11.0, 8.0, 11.0);

        let mut cq = FractalDetector::new();
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
        let k1 = f.k1;
        let k2 = f.k2;
        let k3 = f.k3;
        assert!(k1.bar.high == 8.0 && k1.bar.low == 6.0);
        assert!(k2.bar.high == 9.0 && k2.bar.low == 7.0);
        assert!(k3.bar.high == 7.0 && k3.bar.low == 6.0);
    }

    #[test]
    fn test_check_fractal() {
        let c1 = Candle::new(0, 1052779380000, 1.15642, 1.15642, 1.15627, 1.15627);
        let c2 = Candle::new(10, 1052779380000, 1.15645, 1.15645, 1.15634, 1.15634);
        let c3 = Candle::new(20, 1052779500000, 1.15638, 1.15638, 1.1562, 1.1562);
        let c4 = Candle::new(30, 1052780640000, 1.15604, 1.15604, 1.1559, 1.1559);
        let c5 = Candle::new(40, 1052780820000, 1.15602, 1.15602, 1.15576, 1.15576);
        let c6 = Candle::new(50, 1052780940000, 1.15624, 1.15624, 1.15599, 1.15599);

        let direction = Candle::check_direction(&c1, &c2);
        assert!(direction == Direction::Up);

        let f1 = Fractal::check_fractal(&c1, &c2, &c3);
        assert!(f1.is_some());
        assert!(f1.as_ref().unwrap().fractal_type() == FractalType::Top);
        assert!(f1.as_ref().unwrap().highest() == 1.15645);

        let f2 = Fractal::check_fractal(&c4, &c5, &c6);
        assert!(f2.is_some());
        assert!(f2.as_ref().unwrap().fractal_type() == FractalType::Bottom);
        assert!(f2.as_ref().unwrap().lowest() == 1.15576);
    }

    // eurusd 2021-3-16 3:41 - 3:58的数据来测试
    #[test]
    fn test_candle_merge() {
        let csv = include_str!("../tests/candle_test_eu20210316T0343-0359.csv");
        let bars = load_datetime_bar(&csv);
        let mut i = 1;
        for bar in &bars {
            println!(
                "let b{}=Bar::new({},{},{},{},{});",
                i, bar.time, bar.open, bar.high, bar.low, bar.close
            );
            i += 1;
        }
        let mut fd = FractalDetector::with_candles();
        for bar in &bars {
            let e = fd.on_new_bar(bar);
        }

        let candles = fd.candles.unwrap();
        let pens: Vec<Fractal> = Vec::new();
        let segments: Vec<Fractal> = Vec::new();
        for c in &candles {
            println!(
                "open= {} high={}, low={} close={}",
                c.open, c.high, c.low, c.close
            );
        }
        draw_bar_tradingview(&candles, &pens, &&segments);
    }
}
