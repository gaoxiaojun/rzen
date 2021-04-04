use crate::fractal::Fractal;
use crate::fractal_series::PenEvent;
use crate::pen::Pen;
use crate::time::Time;
use std::{cell::Cell, collections::BTreeMap};

#[derive(Debug)]
pub struct PenStore {
    map: BTreeMap<Time, Pen>,
    count: Cell<u64>,
}

impl PenStore {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            count: Cell::new(0),
        }
    }

    pub fn on_event(&self, event: PenEvent) {
        match event {
            PenEvent::Complete(pen) => {
                //println!("PenComplete: {:?}", pen);
            }
            PenEvent::New(pen) => {
                //println!("PenNew: {:?}", pen);
                let c = self.count.get() + 1;
                self.count.set(c);
            }

            PenEvent::UpdateTo(pen, to) => {
                //println!("PenUpdateTo: {:?} -> {:?}", pen, to);
            }
        }
    }

    pub fn count(&self) -> u64 {
        self.count.get()
    }
}
