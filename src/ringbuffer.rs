use std::collections::VecDeque;

pub struct RingBuffer<T> {
    queue: VecDeque<T>,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
        }
    }

    pub fn get(&self, index: isize) -> Option<&T> {
        if index >= 0 {
            self.queue.get(index as usize)
        } else {
            self.queue.get((self.queue.len() as isize + index) as usize)
        }
    }

    pub fn get_mut(&mut self, index: isize) -> Option<&mut T> {
        if index >= 0 {
            self.queue.get_mut(index as usize)
        } else {
            self.queue
                .get_mut((self.queue.len() as isize + index) as usize)
        }
    }

    pub fn push(&mut self, value: T) {
        let len = self.queue.len();
        let cap = self.queue.capacity();
        if len >= cap {
            self.queue.pop_front();
        }
        self.queue.push_back(value)
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.queue.pop_front()
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.queue.pop_back()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn clear(&mut self) {
        self.queue.clear()
    }
}
