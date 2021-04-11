pub struct Emitter<'a, T> {
    observer: Option<Box<dyn FnMut(&T) + 'a>>,
}

impl<'a, T> Emitter<'a, T> {
    pub fn new() -> Self {
        Self { observer: None }
    }

    pub fn set_observer<F>(&mut self, observer: F)
    //-> Option<Box<dyn Fn(&T)>>
    where
        F: 'a + FnMut(&T),
    {
        //let result = self.observers;
        self.observer = Some(Box::new(observer));
        //result
    }

    pub fn emit(&mut self, item: &T) {
        if self.observer.is_none() {
            return;
        }

        let f = self.observer.as_mut().unwrap();
        (*f)(item)
    }
}
