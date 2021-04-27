use crate::fractal::Fractal;
use crate::time::Time;
#[derive(Debug, Clone, Copy)]
pub struct Point {
    time: Time,
    price: f64,
}

impl Point {
    fn new(time: Time, price: f64) -> Self {
        Self { time, price }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    start: Point,
    end: Point,
    extreme_point: Option<Point>,
    merged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeDirection {
    Up,
    Down,
}

impl Line {
    pub fn new(from_time: Time, from_price: f64, to_time: Time, to_price: f64) -> Self {
        Self {
            start: Point::new(from_time, from_price),
            end: Point::new(to_time, to_price),
            extreme_point: None,
            merged: false,
        }
    }
    pub fn new_from_pen(from: &Fractal, to: &Fractal) -> Self {
        Self {
            start: Point::new(from.time(), from.price()),
            end: Point::new(to.time(), to.price()),
            extreme_point: None,
            merged: false,
        }
    }

    pub fn high(&self) -> f64 {
        if self.start.price > self.end.price {
            self.start.price
        } else {
            self.end.price
        }
    }

    pub fn low(&self) -> f64 {
        if self.start.price < self.end.price {
            self.start.price
        } else {
            self.end.price
        }
    }

    pub fn is_top_fractal(d1: &Line, d2: &Line, d3: &Line) -> bool {
        if d1.high() < d2.high() && d2.high() > d3.high() {
            true
        } else {
            false
        }
    }

    pub fn is_bottom_fractal(s1: &Line, s2: &Line, s3: &Line) -> bool {
        if s1.low() > s2.low() && s2.low() > s3.low() {
            true
        } else {
            false
        }
    }

    pub fn merge(&mut self, rhs: &Line, dir: MergeDirection) -> bool {
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

    pub fn merge_up(&mut self, rhs: &Line) {}
    pub fn merge_down(&mut self, rhs: &Line) {}
}
