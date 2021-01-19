use std::ptr;

#[derive(Default, Debug)]
pub struct FreeList<T> {
    slots: Vec<Slot<T>>,
    head: Option<usize>,
    count: usize,
}

impl<T> FreeList<T> {
    #[must_use]
    pub fn new() -> Self {
        Self { slots: Vec::new(), head: None, count: 0 }
    }

    pub fn insert(&mut self, value: T) -> usize {
        self.count += 1;

        if let Some(index) = self.head {
            let slot = unsafe { self.slots.get_unchecked_mut(index) };
            if let Slot::Vacant { next } = slot {
                self.head = *next;
                *slot = Slot::Occupied { value };
                index
            } else {
                unreachable!()
            }
        } else {
            let index = self.slots.len();
            self.head = Some(index);
            self.slots.push(Slot::Occupied { value });
            index
        }
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if let Slot::Occupied { value } = self.slots.get(index)? {
            Some(value)
        } else {
            None
        }
    }

    #[must_use]
    pub fn remove(&mut self, index: usize) -> T {
        if let Slot::Occupied { value } = self.slots.get(index).unwrap() {
            unsafe {
                let ret = ptr::read(value);
                *self.slots.get_unchecked_mut(index) = Slot::Vacant { next: self.head };
                ret
            }
        } else {
            panic!("Attempted to remove already removed slot.")
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.count
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

// =================
// === INTERNALS ===
// =================

#[derive(Debug)]
enum Slot<T> {
    Occupied { value: T },
    Vacant { next: Option<usize> },
}

#[cfg(tests)]
mod test {
    use super::FreeList;

    #[test]
    fn test() {
        let mut list1 = FreeList::<i32>::new();
        let mut list2 = FreeList::<i32>::new();

        let handle1 = list1.insert(0);
        let handle2 = list2.insert(1);

        println("{}", list1.get(handle2));
    }
}