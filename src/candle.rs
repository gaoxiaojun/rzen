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
    pub bar: Bar,
}

impl Candle {
    #[allow(dead_code)]
    pub(crate) fn new(index: u64, time: Time, open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            index,
            bar: Bar::new(time, open, high, low, close),
        }
    }

    pub(crate) fn from_bar(index: u64, bar: &Bar) -> Self {
        Self {
            index,
            bar: bar.clone(),
        }
    }

    // 检测包含方向
    pub fn check_direction(k1: &Candle, k2: &Candle) -> Direction {
        debug_assert!(k1.index != k2.index);
        if k1.bar.high + k1.bar.low > k2.bar.high + k2.bar.low {
            Direction::Down
        } else {
            Direction::Up
        }
    }

    // 检测并处理包含关系
    // 返回值: true:存在包含关系， false:没有包含关系
    pub fn merge(direction: Direction, current: &mut Candle, bar: &Bar) -> bool {
        // current,bar是否有包含关系
        if (current.bar.high >= bar.high && current.bar.low <= bar.low)
            || (current.bar.high <= bar.high && current.bar.low >= bar.low)
        {
            // 特殊的一字板与前一根K高低点相同情况的处理
            let high_eq_low = bar.high == bar.low; // 一字板

            match direction {
                Direction::Down => {
                    // 下包含，取低低
                    if high_eq_low && bar.low == current.bar.low {
                        // 一字板特例，不处理，直接忽略当前的bar
                        return true;
                    }

                    if current.bar.low > bar.low {
                        current.bar.time = bar.time;
                    }
                    current.bar.high = f64::min(bar.high, current.bar.high);
                    current.bar.low = f64::min(bar.low, current.bar.low);
                    current.bar.open = current.bar.high;
                    current.bar.close = current.bar.low;
                }

                Direction::Up => {
                    // 上包含，取高高
                    if high_eq_low && bar.high == current.bar.high {
                        // 一字板特例，不处理，直接忽略当前的bar
                        return true;
                    }

                    if current.bar.high < bar.high {
                        current.bar.time = bar.time;
                    }
                    current.bar.high = f64::max(bar.high, current.bar.high);
                    current.bar.low = f64::max(bar.low, current.bar.low);
                    current.bar.close = current.bar.high;
                    current.bar.open = current.bar.low;
                }
            }
            //current.bar.close = bar.close;
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
        let c1 = Candle::new(1, 10000, 100.0, 100.0, 50.0, 50.0);
        let mut c2 = Candle::new(11, 10001, 120.0, 120.0, 90.0, 90.0);
        let direction = Candle::check_direction(&c1, &c2);
        let is_contained = Candle::merge(direction, &mut c2, &bar);
        assert_eq!(is_contained, true);
        assert_eq!(direction, Direction::Up);
        assert_eq!(c2.bar.high, 120.0);
        assert_eq!(c2.bar.low, 95.0);
    }

    #[test]
    fn test_candle_util() {
        let c1 = Candle::new(0, 1052779380000, 1.15642, 1.15642, 1.15627, 1.15627);
        let c2 = Candle::new(10, 1052779380000, 1.15645, 1.15645, 1.15634, 1.15634);

        let direction = Candle::check_direction(&c1, &c2);
        assert!(direction == Direction::Up);

        let b1 = Bar::new(1052692740000, 1.15166, 1.15176, 1.15156, 1.15176);

        let mut current = c1.clone();
        let is_contain = Candle::merge(Direction::Up, &mut current, &b1);
        assert!(!is_contain);

        let mut con = c1.clone();

        let b2 = Bar::new(1052692740000, 1.15636, 1.15639, 1.15635, 1.15638);
        let is_contain = Candle::merge(Direction::Up, &mut con, &b2);
        assert!(is_contain);
        assert!(con.bar.high == 1.15642);
        assert!(con.bar.low == 1.15635);
    }
    #[test]
    fn test_candle_merge() {
        // eurusd 2021-03-16 3:43-3:59
        let b1 = Bar::new(1615866240000, 1.1926, 1.1927, 1.19257, 1.19269);
        let b2 = Bar::new(1615866300000, 1.1927, 1.19276, 1.19269, 1.19276);
        let b3 = Bar::new(1615866360000, 1.19275, 1.19276, 1.19273, 1.19273);
        let b4 = Bar::new(1615866420000, 1.19274, 1.19275, 1.19268, 1.19273);
        let b5 = Bar::new(1615866480000, 1.19272, 1.19272, 1.19264, 1.19264);
        let b6 = Bar::new(1615866540000, 1.19265, 1.19274, 1.19264, 1.19274);
        let b7 = Bar::new(1615866600000, 1.19273, 1.19284, 1.1927, 1.19283);
        let b8 = Bar::new(1615866660000, 1.19282, 1.19282, 1.1926, 1.19265);
        let b9 = Bar::new(1615866720000, 1.19264, 1.19269, 1.1926, 1.19264);
        let b10 = Bar::new(1615866780000, 1.19264, 1.19269, 1.19263, 1.19267);
        let b11 = Bar::new(1615866840000, 1.19267, 1.19274, 1.19265, 1.19274);
        let b12 = Bar::new(1615866900000, 1.19275, 1.19284, 1.19275, 1.19284);
        let b13 = Bar::new(1615866960000, 1.19284, 1.19286, 1.19281, 1.19284);
        let b14 = Bar::new(1615867020000, 1.19284, 1.19285, 1.19277, 1.1928);
        let b15 = Bar::new(1615867080000, 1.19278, 1.19288, 1.19278, 1.19288);
        let b16 = Bar::new(1615867140000, 1.19289, 1.19302, 1.19288, 1.19298);

        let mut bars: Vec<Bar> = Vec::new();
        bars.push(b3);
        bars.push(b4);
        bars.push(b5);
        bars.push(b6);
        bars.push(b7);
        bars.push(b8);
        bars.push(b9);
        bars.push(b10);
        bars.push(b11);
        bars.push(b12);
        bars.push(b13);
        bars.push(b14);
        bars.push(b15);
        bars.push(b16);

        let mut candles: Vec<Candle> = Vec::new();
        let mut index = 0;
        let c1 = Candle::from_bar(index, &b1);
        index += 1;
        let mut c2 = Candle::from_bar(index, &b2);
        index += 1;

        let dir = Candle::check_direction(&c1, &c2);
        let mut current = &mut c2;
        for bar in &bars {
            let is_merged = Candle::merge(dir, current, bar);
            if !is_merged {
                candles.push(current.clone());
            }
        }
    }
}
