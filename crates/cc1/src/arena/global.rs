use std::cell::Cell;

pub struct Global<T: 'static> {
    slot: Cell<Option<&'static T>>,
    phase: &'static str,
}

impl<T: 'static> Global<T> {
    pub const fn new(phase: &'static str) -> Self {
        Self { slot: Cell::new(None), phase }
    }

    pub fn install(&self, value: T) -> &'static T {
        let value = Box::leak(Box::new(value));
        self.slot.set(Some(value));
        value
    }

    pub fn get(&self) -> &'static T {
        self.slot.get().unwrap_or_else(|| panic!("{} is not installed", self.phase))
    }
}
