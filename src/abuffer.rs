use super::sync::{Mutex, Condvar, AtomicUsize, Arc, RefCell, AtomicU8};
use core::borrow;
use std::{fmt::Display, vec};

struct Slot<T>
where T: Display + Clone
{
    value: Option<T>,
    state: AtomicU8
}

pub struct ABuffer<T>
where T: Display + Clone
{
    cap: usize,
    data: RefCell<Vec<Slot<T>>>,
    reader: AtomicUsize,
    writer: AtomicUsize
}

impl <T> ABuffer<T> 
where T: Display + Clone
{
    pub fn new(size: usize) -> Self {
        let mut data_vec = Vec::with_capacity(size);

        for _ in 0..size {
            data_vec.push(Slot{
                value: None,
                state: AtomicU8::new(0)
            });
        }
        Self{
            cap: size,
            data:RefCell::new(data_vec),
            reader: AtomicUsize::new(0),
            writer: AtomicUsize::new(0),
        }
    }

    pub fn try_push(self: &Self, value: T) -> Result<(), T> {
        if self.cap == 0 {
            return Err(value);
        }

        let reader_index = self.reader.load(std::sync::atomic::Ordering::SeqCst);
        let write_index = self.writer.load(std::sync::atomic::Ordering::SeqCst);

        let rb = reader_index % self.cap;
        let wb = write_index % self.cap;

        if wb == rb && reader_index + self.cap == write_index {
            return Err(value);
        }

        if let Err(old_write_index) = self.writer.compare_exchange(write_index, write_index + 1, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst) {
            return Err(value);
        }

        let mut vector = self.data.borrow_mut();

        let borrowed_slot = &mut vector[wb];

        borrowed_slot.state.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        borrowed_slot.value = Some(value);
        borrowed_slot.state.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        Ok(())
    }

    pub fn try_pop(self: &Self) -> Option<T> {
        if self.cap == 0 {
            return None;
        }

        let reader_index = self.reader.load(std::sync::atomic::Ordering::SeqCst);
        let write_index = self.writer.load(std::sync::atomic::Ordering::SeqCst);

        let rb = reader_index % self.cap;
        let wb = write_index % self.cap;

        if wb == rb && reader_index == write_index {
            return None;
        }

        if let Err(old_write_index) = self.reader.compare_exchange(reader_index, reader_index + 1, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst) {
            return None;
        }

        let vector = self.data.borrow();

        let borrowed_slot = &vector[rb];
        let mut cond = true;
        let mut data = None;

        while cond {
            let seq_lock = borrowed_slot.state.load(std::sync::atomic::Ordering::SeqCst);
            data = borrowed_slot.value.clone();
            let new_seq_lock = borrowed_slot.state.load(std::sync::atomic::Ordering::SeqCst);

            cond = seq_lock != new_seq_lock;
        }

        data
    }

    pub fn len(&self) -> usize {
        let write_index = self.writer.load(std::sync::atomic::Ordering::SeqCst);
        let read_index = self.reader.load(std::sync::atomic::Ordering::SeqCst);

        if read_index > write_index {
            self.cap - read_index + write_index
        } else {
            write_index - read_index
        }
    }

    pub fn cap(&self) -> usize {
        self.cap
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_cap() {
        let buffer = ABuffer::<i32>::new(0);

        assert_eq!(buffer.cap(), 0);

        assert_eq!(buffer.try_push(1), Err(1));
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.try_pop(), None);
    }

    #[test]
    fn test_push() {
        let buffer = ABuffer::<i32>::new(3);

        assert_eq!(buffer.cap(), 3);

        assert_eq!(buffer.try_push(1), Ok(()));
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.try_push(2), Ok(()));
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.try_push(3), Ok(()));
        assert_eq!(buffer.len(), 3);

        assert_eq!(buffer.try_push(4), Err(4));
        assert_eq!(buffer.len(), 3);
    }

    #[test]
    fn test_pop() {
        let buffer = ABuffer::<i32>::new(3);

        buffer.try_push(1);
        buffer.try_push(2);
        buffer.try_push(3);

        assert_eq!(buffer.try_pop(), Some(1));
        assert_eq!(buffer.try_pop(), Some(2));
        assert_eq!(buffer.try_pop(), Some(3));
        assert_eq!(buffer.try_pop(), None);

        buffer.try_push(4);

        assert_eq!(buffer.try_pop(), Some(4));
    }
}