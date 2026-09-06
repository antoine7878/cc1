use std::iter::zip;
use std::marker::PhantomData;

use crate::arena::ArenaKey;

#[derive(Debug, Clone, PartialEq)]
pub enum Fact<T> {
    Unknown,
    Poisoned,
    Known(T),
    Borrowed,
}

#[derive(Debug)]
pub struct Facts<Id: ArenaKey, Val> {
    slots: Vec<Fact<Val>>,
    marker: PhantomData<fn() -> Id>,
}

impl<Id: ArenaKey, Val> Default for Facts<Id, Val> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<Id: ArenaKey, Val> Facts<Id, Val> {
    pub fn resize(&mut self, len: usize) {
        if len > self.slots.len() {
            self.slots.resize_with(len, || Fact::Unknown);
        }
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn seen(&self, id: Id) -> bool {
        !matches!(self.slots.get(id.into()), None | Some(Fact::Unknown))
    }

    pub fn poisoned(&self, id: Id) -> bool {
        matches!(self.slots.get(id.into()), Some(Fact::Poisoned))
    }

    pub fn get(&self, id: Id) -> Option<&Val> {
        match self.slots.get(id.into()) {
            Some(Fact::Known(v)) => Some(v),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Val> {
        match self.slots.get_mut(id.into()) {
            Some(Fact::Known(v)) => Some(v),
            _ => None,
        }
    }

    pub fn set(&mut self, id: Id, value: Option<Val>) {
        self.slots[id.into()] = match value {
            Some(v) => Fact::Known(v),
            None => Fact::Poisoned,
        };
    }

    pub fn take(&mut self, id: Id) -> Option<Val> {
        match self.slots.get_mut(id.into()) {
            Some(slot @ Fact::Known(_)) => match std::mem::replace(slot, Fact::Borrowed) {
                Fact::Known(v) => Some(v),
                _ => unreachable!(),
            },
            _ => None,
        }
    }

    pub fn give(&mut self, id: Id, value: Val) {
        self.slots[id.into()] = Fact::Known(value);
    }
}

pub trait HasFacts<Id: ArenaKey, Val> {
    fn facts(&mut self) -> &mut Facts<Id, Val>;
}

pub struct Loan<'h, H: HasFacts<Id, Val>, Id: ArenaKey, Val, const N: usize> {
    holder: &'h mut H,
    ids: [Id; N],
    values: Option<[Val; N]>,
}

impl<'h, H: HasFacts<Id, Val>, Id: ArenaKey, Val, const N: usize> Loan<'h, H, Id, Val, N> {
    pub fn take(holder: &'h mut H, ids: [Id; N]) -> Option<Self> {
        let mut taken = std::array::from_fn(|i| holder.facts().take(ids[i]));
        if taken.iter().any(Option::is_none) {
            for (id, val) in zip(ids, &mut taken) {
                if let Some(val) = val.take() {
                    holder.facts().give(id, val);
                }
            }
            return None;
        }
        Some(Self {
            holder,
            ids,
            values: Some(taken.map(Option::unwrap)),
        })
    }

    pub fn parts(&mut self) -> (&mut H, &mut [Val; N]) {
        (self.holder, self.values.as_mut().unwrap())
    }
}

impl<H: HasFacts<Id, Val>, Id: ArenaKey, Val, const N: usize> Drop for Loan<'_, H, Id, Val, N> {
    fn drop(&mut self) {
        for (id, val) in zip(self.ids, self.values.take().unwrap()) {
            self.holder.facts().give(id, val);
        }
    }
}
