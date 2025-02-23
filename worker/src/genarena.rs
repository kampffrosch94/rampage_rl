#![allow(unused)]
use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

pub struct GenArena<T> {
    slots: Vec<Slot<T>>,
    next_free: usize,
}

pub struct Key<T> {
    gen: u32,
    index: u32,
    _type: PhantomData<T>,
}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Key<T> {
        Self { gen: self.gen, index: self.index, _type: PhantomData }
    }
}

impl<T> Copy for Key<T> {}

enum Content<T> {
    Filled(T),
    Empty(usize), // next free index
}

struct Slot<T> {
    gen: u32,
    content: Content<T>,
}

impl<T> GenArena<T> {
    pub fn new() -> Self {
        Self { slots: Vec::new(), next_free: 0 }
    }

    pub fn push(&mut self, x: T) -> Key<T> {
        if self.next_free >= self.slots.len() {
            let index = self.slots.len();
            self.slots.push(Slot { gen: 0, content: Content::Filled(x) });
            self.next_free += 1;
            return Key { gen: 0, index: index as u32, _type: PhantomData };
        } else {
            let index = self.next_free;
            let slot = &mut self.slots[index];
            self.next_free = match slot.content {
                Content::Empty(next) => next,
                Content::Filled(_) => {
                    unreachable!("This should be the next free value, so can't be filled.")
                }
            };
            slot.gen += 1;
            slot.content = Content::Filled(x);
            return Key { index: index as u32, gen: slot.gen, _type: PhantomData };
        }
    }

    /// indempotent
    pub fn remove(&mut self, key: Key<T>) {
        // index is always in bounds if we gave the key out
        let slot = &mut self.slots[key.index as usize];
        if slot.gen != key.gen {
            return;
        }
        slot.gen += 1;
        slot.content = Content::Empty(self.next_free);
        self.next_free = key.index as usize;
    }

    pub fn get(&self, key: Key<T>) -> Option<&T> {
        // index is always in bounds if we gave the key out
        let slot = &self.slots[key.index as usize];
        if slot.gen != key.gen {
            return None;
        }
        return match slot.content {
            Content::Empty(_) => None,
            Content::Filled(ref x) => Some(x),
        };
    }

    pub fn get_mut(&mut self, key: Key<T>) -> Option<&mut T> {
        // index is always in bounds if we gave the key out
        let slot = &mut self.slots[key.index as usize];
        if slot.gen != key.gen {
            return None;
        }
        return match slot.content {
            Content::Empty(_) => None,
            Content::Filled(ref mut x) => Some(x),
        };
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.slots.iter().filter_map(|it| match it.content {
            Content::Filled(ref content) => Some(content),
            Content::Empty(_) => None,
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.slots.iter_mut().filter_map(|it| match it.content {
            Content::Filled(ref mut content) => Some(content),
            Content::Empty(_) => None,
        })
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = (Key<T>, &T)> {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            let key = Key { index: index as u32, gen: slot.gen, _type: PhantomData };
            match slot.content {
                Content::Filled(ref content) => Some((key, content)),
                Content::Empty(_) => None,
            }
        })
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }
}

impl<T> Index<Key<T>> for GenArena<T> {
    type Output = T;

    fn index(&self, index: Key<T>) -> &Self::Output {
        self.get(index).expect("Key was invalid")
    }
}

impl<T> IndexMut<Key<T>> for GenArena<T> {
    fn index_mut(&mut self, index: Key<T>) -> &mut Self::Output {
        self.get_mut(index).expect("Key was invalid")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_insert() {
        let mut arena = GenArena::new();
        let key1 = arena.push("1".to_string());
        assert_eq!("1", arena[key1]);
        let key2 = arena.push("2".into());
        assert_eq!("2", arena[key2]);
        assert_eq!("1", arena[key1]);
    }

    #[test]
    fn test_remove() {
        let mut arena = GenArena::new();
        let key1 = arena.push("1".to_string());
        let key2 = arena.push("2".into());
        arena.remove(key1);
        let _key3 = arena.push("1".into());
        assert_eq!(None, arena.get(key1));
        assert_eq!("2", arena[key2]);
        assert_eq!(2, arena.len()); // slot got reused
        let key4 = arena.push("3".into());
        assert_eq!("3", arena[key4]);
        assert_eq!(3, arena.len());
    }

    #[test]
    fn test_iter() {
        let mut arena = GenArena::new();
        let _key1 = arena.push(1_u32);
        let key2 = arena.push(2);
        let _key3 = arena.push(3);
        arena.remove(key2);
        assert_eq!(&[1, 3], arena.iter().cloned().collect::<Vec<_>>().as_slice());
    }
}
