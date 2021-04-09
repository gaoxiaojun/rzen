use crate::bar::Bar;
use crate::time::Time;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub struct Candle {
    // index的作用是为了计算Candle之间的距离，严格笔要求分型之间有5根K，通过index2 - index1就很容易检测是否满足条件，而无需保存整个Candle序列
    // 检测到分型的时候，分型的index就是分型中间Candle的index
    pub index: u64,
    pub time: Time,
    pub high: f64,
    pub low: f64,
}

impl Candle {
    pub(crate) fn new(index: u64, time: Time, high: f64, low: f64) -> Self {
        Self {
            index,
            time,
            high,
            low,
        }
    }

    // 检测包含方向
    pub fn check_direction(k1: &Candle, k2: &Candle) -> Direction {
        debug_assert!(k1.index != k2.index);
        if k1.high + k1.low > k2.high + k2.low {
            Direction::Down
        } else {
            Direction::Up
        }
    }

    // 检测并处理包含关系
    // 返回值: true:存在包含关系， false:没有包含关系
    pub fn check_contain(direction: Direction, current: &mut Candle, bar: &Bar) -> bool {
        // current,bar是否有包含关系
        if (current.high >= bar.high && current.low <= bar.low)
            || (current.high <= bar.high && current.low >= bar.low)
        {
            // 特殊的一字板与前一根K高低点相同情况的处理
            let high_eq_low = bar.high == bar.low; // 一字板

            match direction {
                Direction::Down => {
                    // 下包含，取低低
                    if high_eq_low && bar.low == current.low {
                        // 一字板特例，不处理，直接忽略当前的bar
                        return true;
                    }

                    if current.low > bar.low {
                        current.time = bar.time;
                    }
                    current.high = f64::min(bar.high, current.high);
                    current.low = f64::min(bar.low, current.low);
                }

                Direction::Up => {
                    // 上包含，取高高
                    if high_eq_low && bar.high == current.high {
                        // 一字板特例，不处理，直接忽略当前的bar
                        return true;
                    }

                    if current.high < bar.high {
                        current.time = bar.time;
                    }
                    current.high = f64::max(bar.high, current.high);
                    current.low = f64::max(bar.low, current.low);
                }
            }
            //current.close = bar.close;
            true
        } else {
            false
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_contain_relationship() {
        let bar = Bar::new(10002, 100.0, 110.0, 95.0, 99.0);
        let c1 = Candle::new(1, 10000, 100.0, 50.0);
        let mut c2 = Candle::new(11, 10001, 120.0, 90.0);
        let direction = Candle::check_direction(&c1, &c2);
        let is_contained = Candle::check_contain(direction, &mut c2, &bar);
        assert_eq!(is_contained, true);
        assert_eq!(direction, Direction::Up);
        assert_eq!(c2.high, 120.0);
        assert_eq!(c2.low, 95.0);
    }

    #[test]
    fn test_candle_util() {
        let c1 = Candle::new(0, 1052779380000, 1.15642, 1.15627);
        let c2 = Candle::new(10, 1052779380000, 1.15645, 1.15634);

        let direction = Candle::check_direction(&c1, &c2);
        assert!(direction == Direction::Up);

        let b1 = Bar::new(1052692740000, 1.15166, 1.15176, 1.15156, 1.15176);

        let mut current = c1.clone();
        let is_contain = Candle::check_contain(Direction::Up, &mut current, &b1);
        assert!(!is_contain);

        let mut con = c1.clone();

        let b2 = Bar::new(1052692740000, 1.15636, 1.15639, 1.15635, 1.15638);
        let is_contain = Candle::check_contain(Direction::Up, &mut con, &b2);
        assert!(is_contain);
        assert!(con.high == 1.15642);
        assert!(con.low == 1.15635);
    }
}
