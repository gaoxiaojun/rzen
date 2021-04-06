use crate::bar::Bar;
use crate::candle::Candle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone)]
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

// 检测包含方向
pub(crate) fn _check_direction(k1: &CandleWithIndex, k2: &CandleWithIndex) -> Direction {
    debug_assert!(k1.index != k2.index);
    if k1.candle.high + k1.candle.low > k2.candle.high + k2.candle.low {
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
                if current.candle.low > bar.low {
                    current.candle.time = bar.time;
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
                if current.candle.high < bar.high {
                    current.candle.time = bar.time;
                }
            }
        }
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_contain_relationship() {
        let bar = Bar::new(10002, 100.0, 110.0, 95.0, 99.0);
        let k1 = Candle::new(10000, 100.0, 50.0);
        let k2 = Candle::new(10001, 120.0, 90.0);
        let c1 = CandleWithIndex::new(10, k1);
        let mut c2 = CandleWithIndex::new(11, k2);
        let direction = _check_direction(&c1, &c2);
        let is_contained = _check_contain(direction, &mut c2, &bar);
        assert_eq!(is_contained, true);
        assert_eq!(direction, Direction::Up);
        assert_eq!(c2.candle.high, 120.0);
        assert_eq!(c2.candle.low, 95.0);
    }

    #[test]
    fn test_candle_util() {
        let k1 = Candle::new(1052779380000, 1.15642, 1.15627);
        let k2 = Candle::new(1052779380000, 1.15645, 1.15634);

        let c1 = CandleWithIndex::new(0, k1);
        let c2 = CandleWithIndex::new(10, k2);

        let direction = _check_direction(&c1, &c2);
        assert!(direction == Direction::Up);

        let b1 = Bar::new(1052692740000, 1.15166, 1.15176, 1.15156, 1.15176);

        let mut current = c1.clone();
        let is_contain = _check_contain(Direction::Up, &mut current, &b1);
        assert!(!is_contain);

        let mut con = c1.clone();

        let b2 = Bar::new(1052692740000, 1.15636, 1.15639, 1.15635, 1.15638);
        let is_contain = _check_contain(Direction::Up, &mut con, &b2);
        assert!(is_contain);
        assert!(con.candle.high == 1.15642);
        assert!(con.candle.low == 1.15635);
    }
}
