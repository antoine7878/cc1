use std::iter::zip;
use std::marker::PhantomData;

use crate::arena::store::out_of_bounds;
use crate::arena::ArenaKey;
use crate::semantic::Diagnosis;

#[derive(Debug, Clone, PartialEq)]
pub enum Slot<T> {
    Unknown,
    Poisoned,
    Known(T),
    Borrowed,
}

#[derive(Debug)]
pub struct SideTable<Id: ArenaKey, Val> {
    slots: Vec<Slot<Val>>,
    marker: PhantomData<fn() -> Id>,
}

impl<Id: ArenaKey, Val> Default for SideTable<Id, Val> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<Id: ArenaKey, Val> SideTable<Id, Val> {
    pub fn resize(&mut self, len: usize) {
        if len > self.slots.len() {
            self.slots.resize_with(len, || Slot::Unknown);
        }
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn seen(&self, id: Id) -> bool {
        !matches!(self.slots.get(id.into()), None | Some(Slot::Unknown))
    }

    pub fn poisoned(&self, id: Id) -> bool {
        matches!(self.slots.get(id.into()), Some(Slot::Poisoned))
    }

    pub fn get(&self, id: Id) -> Option<&Val> {
        match self.slots.get(id.into()) {
            Some(Slot::Known(v)) => Some(v),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Val> {
        match self.slots.get_mut(id.into()) {
            Some(Slot::Known(v)) => Some(v),
            _ => None,
        }
    }

    pub fn set(&mut self, id: Id, value: Option<Val>) {
        let slot = match value {
            Some(v) => Slot::Known(v),
            None => Slot::Poisoned,
        };
        *self.slot_mut(id) = slot;
    }

    pub fn take(&mut self, id: Id) -> Option<Val> {
        match self.slots.get_mut(id.into()) {
            Some(slot @ Slot::Known(_)) => match std::mem::replace(slot, Slot::Borrowed) {
                Slot::Known(v) => Some(v),
                _ => unreachable!(),
            },
            _ => None,
        }
    }

    pub fn give(&mut self, id: Id, value: Val) {
        *self.slot_mut(id) = Slot::Known(value);
    }

    fn slot_mut(&mut self, id: Id) -> &mut Slot<Val> {
        let len = self.slots.len();
        match self.slots.get_mut(id.into()) {
            Some(slot) => slot,
            None => out_of_bounds::<Id, Val>("side table", id, len),
        }
    }
}

pub trait HasTable<Id: ArenaKey, Val> {
    fn table(&mut self) -> &mut SideTable<Id, Val>;
}

pub struct Loan<'h, H: HasTable<Id, Val>, Id: ArenaKey, Val, const N: usize> {
    holder: &'h mut H,
    ids: [Id; N],
    values: Option<[Val; N]>,
}

impl<'h, H: HasTable<Id, Val>, Id: ArenaKey, Val, const N: usize> Loan<'h, H, Id, Val, N> {
    pub fn take(holder: &'h mut H, ids: [Id; N]) -> Option<Self> {
        let mut taken = std::array::from_fn(|i| holder.table().take(ids[i]));
        if taken.iter().any(Option::is_none) {
            for (id, val) in zip(ids, &mut taken) {
                if let Some(val) = val.take() {
                    holder.table().give(id, val);
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

impl<H: HasTable<Id, Val>, Id: ArenaKey, Val, const N: usize> Drop for Loan<'_, H, Id, Val, N> {
    fn drop(&mut self) {
        for (id, val) in zip(self.ids, self.values.take().unwrap()) {
            self.holder.table().give(id, val);
        }
    }
}

pub trait OptionPoisoned<T> {
    fn ok_poisoned(self) -> Result<T, Diagnosis>;
}

impl<T> OptionPoisoned<T> for Option<T> {
    fn ok_poisoned(self) -> Result<T, Diagnosis> {
        self.ok_or(Diagnosis::Poisoned)
    }
}
