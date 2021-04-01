use crate::fractal::Fractal;
use crate::fractal_util::{MergeAction, _is_pen, _merge_same_type};
use crate::pen::Pen;
use crate::ringbuffer::RingBuffer;

// 考虑一种特殊情况就是顶分型高点相等或者底分型低点相等
pub struct FractalQueue {
    window: RingBuffer<Fractal>,
    current_pen: Option<Pen>,
}

// 寻找第一笔
// case 0
// +---+                            +---+
// |   |<-----A             =====>  | A |
// +---+                            +---+
// 转case1

// case 1
// +---+                            +---+               +---+---+
// | A |<-----B             ======> |A/B|        or     | A | B |
// +---+                            +---+               +---+---+
// AB同类型，合并AB，转case1
// AB不同类型，保存B
// 1.1 AB不成笔转case2
// 1.2 AB成笔转case3

// case 2
// +---+---+                        +---+---+           +---+---+       +---+
// | A | B |<-----C         =====>  | B | C |     or    | A |B/C|   or  |A/C|
// +---+---+                        +---+---+           +---+---+       +---+
// 前提：AB不成笔
// 1.1 BC成笔 ---- 去掉A，保留BC，转case3
// 1.2 BC不成笔
// 1.2.1 BC同类，按同类合并规则处理决定保留B或者C, 转case2
// 1.2.2 BC不同类，那么AC同类，按同类合并规则处理决定保留A或者C，B被丢弃，转case1 // TODO  需要更好的策略

// 已经有笔
// case 3
// +---+---+                        +---+---+           +---+---+---+
// | A | B |<-----C         =====>  | B | C |      or   | A | B | C |
// +---+---+                        +---+---+           +---+---+---+
// 前提 AB成笔
//  1.1 BC成笔   --- AB笔完成，emit笔完成事件，去掉A，剩下BC,转case3
//  1.2 BC不成笔，
//  1.2.1 BC类型不同，保留C，转case4
//  1.2.2 BC类型相同，按同类合并规则处理决定保留B或者C，如果C覆盖B，更新笔，转case3

// case 4
// +---+---+---+                    +---+---+---+       +---+---+
// | A | B | C |<-----D     =====>  | A | B |C/D|   or  | A |B/D|
// +---+---+---+                    +---+---+---+       +---+---+
// 前提 AB成笔且BC类型不同且BC不成笔
// 1.1 CD同类型-----按同类合并规则处理决定保留C或者D,转case4
// 1.1.1 如果D覆盖C，要检测BD是否成笔,如果成笔，AB笔完成，emit笔完成事件，去掉A，转case3
// 1.2 CD不同类-----去掉C，BD按同类合并规则处理决定保留B或者D,转case3

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
        self.window.pop_front();
        let new_a = self.window.get(0).unwrap();
        let new_b = self.window.get(1).unwrap();
        let new_pen = Pen::new(new_a.clone(), new_b.clone());
        self.current_pen = Some(new_pen);
    }

    fn ab_pen_update(&mut self) {
        debug_assert!(self.window.len() == 2);
        let new_b = self.window.get(1).unwrap();
        self.current_pen.as_mut().unwrap().update_to(new_b.clone());
    }

    fn case0(&mut self, f: Fractal) {
        debug_assert!(self.window.len() == 0 && self.current_pen.is_none());
        self.window.push(f)
    }

    fn case1(&mut self, f: Fractal) {
        debug_assert!(self.window.len() == 1 && self.current_pen.is_none());
        let last = self.window.get(-1).unwrap();
        if last.fractal_type() == f.fractal_type() {
            let action = _merge_same_type(last, &f);
            if action == MergeAction::Replace {
                self.window.pop_back();
                self.window.push(f);
            }
        } else {
            self.window.push(f);
        }
    }

    fn case2(&mut self, f: Fractal) {
        debug_assert!(
            !_is_pen(self.window.get(-2).unwrap(), self.window.get(-1).unwrap())
                && self.current_pen.is_none()
        );

        let b = self.window.get(-1).unwrap();
        let bc_is_pen = _is_pen(b, &f);
        if bc_is_pen {
            self.window.pop_front();
            self.window.push(f);
            let new_b = self.window.get(-2).unwrap();
            let new_c = self.window.get(-1).unwrap();
            let pen = Pen::new(new_b.clone(), new_c.clone());
            self.current_pen = Some(pen);
        } else {
            if b.is_same_type(&f) {
                let action = _merge_same_type(b, &f);
                if action == MergeAction::Replace {
                    self.window.pop_back();
                    self.window.push(f);
                }
            } else {
                let a = self.window.get(-2).unwrap();
                let action = _merge_same_type(a, &f);
                self.window.pop_back(); // remove b
                if action == MergeAction::Replace {
                    self.window.pop_back();
                    self.window.push(f);
                }
            }
        }
    }

    fn case3(&mut self, f: Fractal) {
        debug_assert!(
            _is_pen(self.window.get(-2).unwrap(), self.window.get(-1).unwrap())
                && self.current_pen.is_some()
        );
        let b = self.window.get(-1).unwrap();
        let bc_is_pen = _is_pen(b, &f);
        if bc_is_pen {
            self.window.push(f);
            self.ab_pen_complete_bc_pen_new();
        } else {
            if b.is_same_type(&f) {
                let action = _merge_same_type(b, &f);
                if action == MergeAction::Replace {
                    self.window.pop_back();
                    self.window.push(f);
                    self.ab_pen_update();
                }
            } else {
                self.window.push(f);
            }
        }
    }

    fn case4(&mut self, f: Fractal) {
        debug_assert!({
            let ab_is_pen = _is_pen(self.window.get(-3).unwrap(), self.window.get(-2).unwrap());
            let bc_is_same_type = self
                .window
                .get(-2)
                .unwrap()
                .is_same_type(self.window.get(-1).unwrap());
            let bc_is_pen = _is_pen(self.window.get(-2).unwrap(), self.window.get(-1).unwrap());
            ab_is_pen && !bc_is_same_type && !bc_is_pen && self.current_pen.is_some()
        });

        let c = self.window.get(-1).unwrap();
        if c.is_same_type(&f) {
            let action = _merge_same_type(c, &f);
            if action == MergeAction::Replace {
                self.window.pop_back();
                self.window.push(f);
                let bc_is_pen = _is_pen(self.window.get(-2).unwrap(), self.window.get(-1).unwrap());
                if bc_is_pen {
                    self.ab_pen_complete_bc_pen_new();
                }
            }
        } else {
            self.window.pop_back();
            let b = self.window.get(-1).unwrap();
            let action = _merge_same_type(b, &f);
            if action == MergeAction::Replace {
                self.window.pop_back();
                self.window.push(f);
            }
        }
    }

    pub fn on_new_fractal(&mut self, f: Fractal) -> Option<Pen> {
        let len = self.window.len();
        let is_some = self.current_pen.is_some();

        match (is_some, len) {
            (false, 0) => self.case0(f),
            (false, 1) => self.case1(f),
            (false, 2) => self.case2(f),
            (true, 2) => self.case3(f),
            (true, 3) => self.case4(f),
            (_, _) => {}
        }

        None
    }
}
