use crate::bar::Bar;
use crate::candle::Candle;
use crate::fractal::{Fractal, FractalType};

pub(crate) struct CandleWithIndex {
    // index的作用是为了计算Candle之间的距离，严格笔要求分型之间有5根K，通过index2 - index1就很容易检测是否满足条件，而无需保存整个Candle序列
    // 检测到分型的时候，分型的index就是分型中间Candle的index
    pub index: u64,
    pub candle: Candle,
}

impl CandleWithIndex {
    pub(crate) fn new(index: u64, candle: Candle) -> Self {
        Self { index, candle }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
    Up,
    Down,
}

//  ------k2---------
//  ------|----------
//  -k1-|---|-k3-----
//  ------|----------
//  -----k2----------
// 检查分型
pub(crate) fn _check_fractal(
    k1: &CandleWithIndex,
    k2: &CandleWithIndex,
    k3: &CandleWithIndex,
) -> Option<Fractal> {
    if (k1.candle.high < k2.candle.high) && (k2.candle.high > k3.candle.high) {
        debug_assert!(
            k1.candle.low <= k2.candle.low && k2.candle.low >= k3.candle.low,
            "顶分型的底不是最高的"
        );
        return Some(Fractal::new(
            FractalType::Top,
            k2.index,
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
            k2.index,
            k1.candle.clone(),
            k2.candle.clone(),
            k3.candle.clone(),
        ));
    }

    None
}

// 检测包含方向
pub(crate) fn _check_direction(k1: &CandleWithIndex, k2: &CandleWithIndex) -> Direction {
    if k1.candle.high > k2.candle.high {
        Direction::Down
    } else {
        Direction::Up
    }
}

// 检测并处理包含关系
// 返回值: true:存在包含关系， false:没有包含关系
pub(crate) fn _check_contain(
    direction: Direction,
    current: &mut CandleWithIndex,
    bar: &Bar,
) -> bool {
    // current,bar是否有包含关系
    if (current.candle.high >= bar.high && current.candle.low <= bar.low)
        || (current.candle.high <= bar.high && current.candle.low >= bar.low)
    {
        // 特殊的一字板与前一根K高低点相同情况的处理
        let high_eq_low = bar.high == bar.low; // 一字板

        match direction {
            Direction::Down => {
                // 下包含，取低低
                if high_eq_low && bar.low == current.candle.low {
                    // 一字板特例，不处理，直接忽略当前的bar
                    return true;
                }

                current.candle.high = f64::min(bar.high, current.candle.high);
                current.candle.low = f64::min(bar.low, current.candle.low);
                current.candle.time = if current.candle.low <= bar.low {
                    current.candle.time
                } else {
                    bar.time
                }
            }

            Direction::Up => {
                // 上包含，取高高
                if high_eq_low && bar.high == current.candle.high {
                    // 一字板特例，不处理，直接忽略当前的bar
                    return true;
                }

                current.candle.high = f64::max(bar.high, current.candle.high);
                current.candle.low = f64::max(bar.low, current.candle.low);
                current.candle.time = if current.candle.high >= bar.high {
                    current.candle.time
                } else {
                    bar.time
                }
            }
        }
        true
    } else {
        false
    }
}
