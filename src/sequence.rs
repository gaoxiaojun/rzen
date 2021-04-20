use crate::time::Time;

// 向上的线段采用向上合并
// 向下的线段采用向下合并

#[derive(Debug)]
pub struct Seq {
    from_index: usize,
    from_time: Time,
    from_price: f64,
    to_index: usize,
    to_time: Time,
    to_price: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeDirection {
    Up,
    Down,
}

impl Seq {
    pub fn new(
        from_index: usize,
        from_time: Time,
        from_price: f64,
        to_index: usize,
        to_time: Time,
        to_price: f64,
    ) -> Self {
        Self {
            from_index,
            from_time,
            from_price,
            to_index,
            to_time,
            to_price,
        }
    }

    pub fn from_index(&self) -> usize {
        self.from_index
    }

    pub fn to_index(&self) -> usize {
        self.to_index
    }

    pub fn start(&self) -> (Time, f64) {
        (self.from_time, self.from_price)
    }

    pub fn end(&self) -> (Time, f64) {
        (self.to_time, self.to_price)
    }

    pub fn high(&self) -> f64 {
        if self.from_price > self.to_price {
            self.from_price
        } else {
            self.to_price
        }
    }

    pub fn low(&self) -> f64 {
        if self.from_price < self.to_price {
            self.from_price
        } else {
            self.to_price
        }
    }

    pub fn merge_up(&mut self, rhs: &Seq) {
        let lhs_length = self.to_price - self.from_price;
        let rhs_length = self.to_price - self.from_price;
        let is_same =
            (lhs_length < 0.0 && rhs_length < 0.0) || (lhs_length > 0.0 && rhs_length > 0.0);

        let is_large = (lhs_length.abs() - rhs_length.abs()) > 0.0;

        match (is_large, is_same) {
            (false, true) => {
                self.from_time = self.to_time;
                self.from_price = self.to_price;
                self.to_time = rhs.from_time;
                self.to_price = rhs.from_price;
            }
            (false, false) => {
                self.to_time = rhs.from_time;
                self.to_price = rhs.from_price;
            }
            (true, true) => {
                self.to_time = rhs.to_time;
                self.to_price = rhs.to_price;
            }
            (true, false) => {
                self.from_time = self.to_time;
                self.from_price = self.to_price;
                self.to_time = rhs.to_time;
                self.to_price = rhs.to_price;
            }
        }
    }

    pub fn merge_down(&mut self, rhs: &Seq) {
        let lhs_length = self.to_price - self.from_price;
        let rhs_length = self.to_price - self.from_price;
        let is_same =
            (lhs_length < 0.0 && rhs_length < 0.0) || (lhs_length > 0.0 && rhs_length > 0.0);

        let is_large = (lhs_length.abs() - rhs_length.abs()) > 0.0;

        match (is_large, is_same) {
            (false, true) => {
                self.from_time = self.to_time;
                self.from_price = self.to_price;
                self.to_time = rhs.from_time;
                self.to_price = rhs.from_price;
            }
            (false, false) => {
                self.to_time = rhs.from_time;
                self.to_price = rhs.from_price;
            }
            (true, true) => {
                self.to_time = rhs.to_time;
                self.to_price = rhs.to_price;
            }
            (true, false) => {
                self.from_time = self.to_time;
                self.from_price = self.to_price;
                self.to_time = rhs.to_time;
                self.to_price = rhs.to_price;
            }
        }
    }

    pub fn merge(&mut self, rhs: &Seq, dir: MergeDirection) -> bool {
        let is_contain_1 = self.high() < rhs.high() && self.low() > rhs.low();
        let is_contain_2 = self.high() > rhs.high() && self.low() < rhs.low();
        let is_contain = is_contain_1 || is_contain_2;

        if !is_contain {
            return false;
        }

        match dir {
            MergeDirection::Up => self.merge_up(rhs),
            MergeDirection::Down => self.merge_down(rhs),
        }

        true
    }

    pub fn is_top_fractal(s1: &Seq, s2: &Seq, s3: &Seq) -> bool {
        if s1.high() < s2.high() && s2.high() > s3.high() {
            true
        } else {
            false
        }
    }

    pub fn is_bottom_fractal(s1: &Seq, s2: &Seq, s3: &Seq) -> bool {
        if s1.low() > s2.low() && s2.low() > s3.low() {
            true
        } else {
            false
        }
    }
}
