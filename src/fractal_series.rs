use crate::fractal::Fractal;
use crate::fractal_util::{MergeAction, _is_pen, _merge_same_type};
use crate::pen::Pen;
use crate::ringbuffer::RingBuffer;

// 考虑一种特殊情况就是顶分型高点相等或者底分型低点相等
pub struct FractalQueue {
    window: RingBuffer<Fractal>,
    current_pen: Option<Pen>,
}

// 一、寻找第一笔
// state 0
// +-0-+                            +-1-+
// |   |<-----A             =====>  | A |
// +---+                            +---+
// 保留A，转state1

// state 1
// +-1-+                            +-1-+               +--2-3--+
// | A |<-----B             ======> |A/B|        or     | A | B |
// +---+                            +---+               +---+---+
// 1.1 AB同类型，按同类合并规则处理AB，转state1
// 1.2 AB不同类型，保存B
// 1.2.1 AB不成笔转state2
// 1.2.2 AB成笔,新建笔变量，转state3

// state 2
// +---2---+                        +---3---+           +--2-3--+       +-1-+
// | A | B |<-----C         =====>  | B | C |     or    | A |B/C|   or  |A/C|
// +---+---+                        +---+---+           +---+---+       +---+
// 前提：AB不成笔
// 2.1 BC成笔 ---- 去掉A，保留BC，新建笔变量，转state3
// 2.2 BC不成笔
// 2.2.1 BC同类，按同类合并规则处理
// 2.2.1.1 如果保留C，要检测AC是否成笔
// 2.2.1.1.1如果成笔，新建笔变量, 转state3
// 2.2.1.1.2如果不成笔，转state 2
// 2.2.1.2 如果保留B，抛弃C，转state2
// 2.2.2 BC不同类，按同类合并规则处理AC
// 2.2.2.1 如果保留A，则抛弃C
// 2.2.2.2 如果保留C，抛弃AB，转state1

// 二、已经有笔
// state 3
// +---3---+                        +---3---+           +---+-4-+---+
// | A | B |<-----C         =====>  | B | C |      or   | A | B | C |
// +---+---+                        +---+---+           +---+---+---+
// 前提 AB成笔
//  3.1 BC成笔   --- AB笔完成，emit笔完成事件，去掉A，剩下BC,更新笔变量，转state3
//  3.2 BC不成笔，
//  3.2.1 BC类型不同，保留C，转state4
//  3.2.2 BC类型相同，按同类合并规则处理BC
//  3.2.2.1如果保留C，更新笔端点，转state3
//  3.2.2.2如果保留B，抛弃C，转state3

// state 4
// +---+-4-+---+                    +---+-4-+---+       +---3---+
// | A | B | C |<-----D     =====>  | A | B |C/D|   or  | A |B/D|
// +---+---+---+                    +---+---+---+       +---+---+
// 前提 AB成笔且BC类型不同且BC不成笔
// 4.1 CD同类型-----按同类合并规则处理CD
// 4.1.1 如果保留D，要检测BD是否成笔
// 4.1.1.1如果BD成笔，AB笔完成，emit笔完成事件，去掉A，剩下BD,更新笔变量，转state3
// 4.1.1.2如果不成笔，转 state4
// 4.1.2 如果保留C，抛弃D，转state4
// 4.2 CD不同类-----去掉C，按同类合并规则处理BD
// 4.2.1 如果保留B,转state3
// 4.2.2 如果保留D，更新笔端点，转state3

impl FractalQueue {
    pub fn new() -> Self {
        Self {
            window: RingBuffer::new(3),
            current_pen: None,
        }
    }

    fn ab_pen_complete_bc_pen_new(&mut self) {
        debug_assert!(self.window.len() == 3);
        let pen = self.current_pen.as_mut().unwrap();
        pen.commit();
        self.bc_new_pen_and_pop_a();
    }

    fn bc_new_pen_and_pop_a(&mut self) {
        self.window.pop_front();
        self.ab_new_pen();
    }

    fn ab_new_pen(&mut self) {
        debug_assert!(self.window.len() == 2);
        let from = self.window.get(0).unwrap();
        let to = self.window.get(1).unwrap();
        let pen = Pen::new(from.clone(), to.clone());
        self.current_pen = Some(pen);
    }

    fn _is_pen(&self, start_index: usize) -> bool {
        debug_assert!(self.window.len() >= 2 + start_index);
        _is_pen(
            self.window.get(start_index as isize).unwrap(),
            self.window.get((start_index + 1) as isize).unwrap(),
        )
    }

    fn bc_is_pen(&self) -> bool {
        self._is_pen(1)
    }

    fn ab_is_pen(&self) -> bool {
        self._is_pen(0)
    }

    fn ab_pen_update(&mut self) {
        debug_assert!(self.window.len() == 2);
        let new_to = self.window.get(1).unwrap();
        self.current_pen.as_mut().unwrap().update_to(new_to.clone());
    }

    fn state0(&mut self, f: Fractal) {
        debug_assert!(self.window.len() == 0 && self.current_pen.is_none());
        self.window.push(f)
    }

    fn state1(&mut self, f: Fractal) {
        debug_assert!(self.window.len() == 1 && self.current_pen.is_none());
        let last = self.window.get(-1).unwrap();
        if last.fractal_type() == f.fractal_type() {
            // 1.1
            let action = _merge_same_type(last, &f);
            if action == MergeAction::Replace {
                self.window.pop_back();
                self.window.push(f);
            }
        } else {
            // 1.2
            self.window.push(f);
            if self.ab_is_pen() {
                // 1.2.2
                self.ab_new_pen();
            }
        }
    }

    fn state2(&mut self, f: Fractal) {
        debug_assert!(!self.ab_is_pen() && self.current_pen.is_none() && self.window.len() == 2);

        let b = self.window.get(-1).unwrap();
        let bc_is_pen = _is_pen(b, &f);
        if bc_is_pen {
            // 2.1
            self.window.push(f);
            self.bc_new_pen_and_pop_a();
        } else {
            // 2.2
            if b.is_same_type(&f) {
                // 2.2.1
                let action = _merge_same_type(b, &f);
                if action == MergeAction::Replace {
                    // 2.2.1.1
                    self.window.pop_back(); // pop b
                    self.window.push(f);
                    // test ac is pen?
                    if self.ab_is_pen() {
                        // 2.2.1.1.1
                        self.ab_new_pen();
                    }
                }
            } else {
                // 2.2.2
                let a = self.window.get(-2).unwrap();
                let action = _merge_same_type(a, &f);
                if action == MergeAction::Replace {
                    // 2.2.2.2
                    self.window.clear();
                    self.window.push(f);
                }
            }
        }
    }

    fn state3(&mut self, f: Fractal) {
        debug_assert!(self.ab_is_pen() && self.current_pen.is_some() && self.window.len() == 2);
        let b = self.window.get(-1).unwrap();
        let bc_is_pen = _is_pen(b, &f);
        if bc_is_pen {
            // 3.1
            self.window.push(f);
            self.ab_pen_complete_bc_pen_new();
        } else {
            if b.is_same_type(&f) {
                let action = _merge_same_type(b, &f);
                if action == MergeAction::Replace {
                    // 3.2.2.1
                    self.window.pop_back();
                    self.window.push(f);
                    self.ab_pen_update();
                }
            } else {
                // 3.2.1
                self.window.push(f);
            }
        }
    }

    fn state4(&mut self, f: Fractal) {
        debug_assert!({
            let ab_is_pen = self.ab_is_pen();
            let bc_is_same_type = self
                .window
                .get(-2)
                .unwrap()
                .is_same_type(self.window.get(-1).unwrap());
            let bc_is_pen = _is_pen(self.window.get(-2).unwrap(), self.window.get(-1).unwrap());
            ab_is_pen
                && !bc_is_same_type
                && !bc_is_pen
                && self.current_pen.is_some()
                && self.window.len() == 3
        });

        let c = self.window.get(-1).unwrap();
        if c.is_same_type(&f) {
            // 4.1
            let action = _merge_same_type(c, &f);
            if action == MergeAction::Replace {
                // 4.1.1
                self.window.pop_back();
                self.window.push(f);
                if self.bc_is_pen() {
                    // 4.1.1.1
                    self.ab_pen_complete_bc_pen_new();
                }
            }
        } else {
            // 4.2
            self.window.pop_back();
            let b = self.window.get(-1).unwrap();
            let action = _merge_same_type(b, &f);
            if action == MergeAction::Replace {
                // 4.2.2
                self.window.pop_back();
                self.window.push(f);
                self.ab_pen_update();
            }
        }
    }

    pub fn on_new_fractal(&mut self, f: Fractal) {
        let len = self.window.len();
        let is_some = self.current_pen.is_some();

        match (is_some, len) {
            (false, 0) => self.state0(f),
            (false, 1) => self.state1(f),
            (false, 2) => self.state2(f),
            (true, 2) => self.state3(f),
            (true, 3) => self.state4(f),
            (_, _) => {
                unreachable!()
            }
        }
    }
}
