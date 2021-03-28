use crate::bar::Bar;
use crate::candle::Candle;
use crate::fractal::{Fractal, FractalType};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer, RingBufferExt, RingBufferWrite};
pub(crate) struct CandleWithId {
    pub id: u64, // id的最用是为了计算Candle之间的距离，严格笔要求分型之间有5根K，通过Candle1.id - Candle2.id就很容易检测是否满足条件，而无需保存整个Candle序列
    pub candle: Candle,
}

impl CandleWithId {
    pub(crate) fn new(id: u64, candle: Candle) -> Self {
        Self { id, candle }
    }
}

trait FractalHandler {
    fn on_new_fractal(&mut self, f: Fractal);
}

enum Direction {
    Up,
    Down,
}

pub struct CandleQueue<'a> {
    window: ConstGenericRingBuffer<CandleWithId, 3>,
    next_id: u64,
    handler: Box<dyn FnMut(Fractal) + 'a>,
}

impl<'a> CandleQueue<'a> {
    pub fn new(handler: impl FnMut(Fractal) + 'a) -> Self {
        Self {
            window: ConstGenericRingBuffer::new(),
            next_id: 0,
            handler: Box::new(handler),
        }
    }

    fn add_candle(&mut self, candle: Candle) {
        let cid = CandleWithId::new(self.next_id, candle);
        self.next_id += 1;
        self.window.push(cid);
    }

    fn add_bar(&mut self, bar: &Bar) {
        let c = Candle::from_bar(bar);
        self.add_candle(c);
    }

    fn check_fractal(&self) -> Option<Fractal> {
        let k1 = self.window.get(-3).unwrap();
        let k2 = self.window.get(-2).unwrap();
        let k3 = self.window.get(-1).unwrap();

        if (k1.candle.high < k2.candle.high) && (k2.candle.high > k3.candle.high) {
            debug_assert!(
                k1.candle.low <= k2.candle.low && k2.candle.low >= k3.candle.low,
                "顶分型的底不是最高的"
            );
            return Some(Fractal::new(
                FractalType::Top,
                k2.id,
                k1.candle.clone(),
                k2.candle.clone(),
                k3.candle.clone(),
            ));
        }

        if (k1.candle.low > k2.candle.low) && (k2.candle.low < k3.candle.low) {
            debug_assert!(
                (k1.candle.high >= k2.candle.high) && (k2.candle.high <= k3.candle.high),
                "底分型的顶不是最低的"
            );
            return Some(Fractal::new(
                FractalType::Bottom,
                k2.id,
                k1.candle.clone(),
                k2.candle.clone(),
                k3.candle.clone(),
            ));
        }

        None
    }

    fn on_new_fractal(&mut self, f: Fractal) {
        (self.handler)(f)
    }

    fn process_contain_relationship(&mut self, bar: &Bar) {
        // 队列中有两个及以上的经过包含处理的K线,处理与当前bar的包含关系
        let direction = {
            let k1 = self.window.get(-2).unwrap();
            let k2 = self.window.get(-1).unwrap();
            if k1.candle.high > k2.candle.high {
                Direction::Down
            } else {
                Direction::Up
            }
        };

        let k2 = self.window.get_mut(-1).unwrap();

        // 检测k2,bar的是否有包含关系
        if (k2.candle.high >= bar.high && k2.candle.low <= bar.low)
            || (k2.candle.high <= bar.high && k2.candle.low >= bar.low)
        {
            // 特殊的一字板与前一根K高低点相同情况的处理
            let high_eq_low = bar.high == bar.low; // 一字板

            match direction {
                Direction::Down => {
                    // 下包含，取低低
                    if high_eq_low && bar.low == k2.candle.low {
                        // 一字板特例，不处理，直接忽略当前的bar
                        return;
                    }

                    k2.candle.high = f64::min(bar.high, k2.candle.high);
                    k2.candle.low = f64::min(bar.low, k2.candle.low);
                    k2.candle.time = if k2.candle.low <= bar.low {
                        k2.candle.time
                    } else {
                        bar.time
                    }
                }
                Direction::Up => {
                    // 上包含，取高高
                    if high_eq_low && bar.high == k2.candle.high {
                        // 一字板特例，不处理，直接忽略当前的bar
                        return;
                    }

                    k2.candle.high = f64::max(bar.high, k2.candle.high);
                    k2.candle.low = f64::max(bar.low, k2.candle.low);
                    k2.candle.time = if k2.candle.high >= bar.high {
                        k2.candle.time
                    } else {
                        bar.time
                    }
                }
            }
        }
    }

    pub fn update(&mut self, bar: &Bar) {
        let len = self.window.len();
        debug_assert!(len <= 3);

        // 初始边界条件验证，前两个candle必须是非包含的
        match len {
            0 => {
                // 队列中没有K线
                self.add_bar(bar);
                return;
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
                    return;
                };

                if k2_include_k1 {
                    // 情况2，忽略K1,清空队列
                    self.window.clear();
                }
                // 当前Bar作为Candle放入队列
                self.add_bar(bar);
            }

            _ => {
                // 队列中有两个及以上的经过包含处理的K线,处理与当前bar的包含关系
                let direction = {
                    let k1 = self.window.get(-2).unwrap();
                    let k2 = self.window.get(-1).unwrap();
                    if k1.candle.high > k2.candle.high {
                        Direction::Down
                    } else {
                        Direction::Up
                    }
                };

                let k2 = self.window.get_mut(-1).unwrap();

                // 检测k2,bar的是否有包含关系
                if (k2.candle.high >= bar.high && k2.candle.low <= bar.low)
                    || (k2.candle.high <= bar.high && k2.candle.low >= bar.low)
                {
                    // 特殊的一字板与前一根K高低点相同情况的处理
                    let high_eq_low = bar.high == bar.low; // 一字板

                    match direction {
                        Direction::Down => {
                            // 下包含，取低低
                            if high_eq_low && bar.low == k2.candle.low {
                                // 一字板特例，不处理，直接忽略当前的bar
                                return;
                            }

                            k2.candle.high = f64::min(bar.high, k2.candle.high);
                            k2.candle.low = f64::min(bar.low, k2.candle.low);
                            k2.candle.time = if k2.candle.low <= bar.low {
                                k2.candle.time
                            } else {
                                bar.time
                            }
                        }
                        Direction::Up => {
                            // 上包含，取高高
                            if high_eq_low && bar.high == k2.candle.high {
                                // 一字板特例，不处理，直接忽略当前的bar
                                return;
                            }

                            k2.candle.high = f64::max(bar.high, k2.candle.high);
                            k2.candle.low = f64::max(bar.low, k2.candle.low);
                            k2.candle.time = if k2.candle.high >= bar.high {
                                k2.candle.time
                            } else {
                                bar.time
                            }
                        }
                    }
                } else {
                    // 无包含关系
                    if self.window.len() == 3 {
                        match self.check_fractal() {
                            Some(f) => self.on_new_fractal(f),
                            _ => {}
                        }
                    }

                    self.add_bar(bar);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
