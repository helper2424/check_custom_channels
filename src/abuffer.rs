use super::sync::{AtomicUsize, AtomicU8, Arc, thread};
use super::sync::Ordering;
use core::borrow;
use std::{fmt::Display, vec};
use std::cell::{RefCell, UnsafeCell};

struct Slot<T>
where T: Display + Clone
{
    value: UnsafeCell<Option<T>>,
    state: AtomicU8
}

unsafe impl<T> Sync for Slot<T> 
where T: Display + Clone
{}

pub struct ABuffer<T>
where T: Display + Clone
{
    cap: usize,
    data: Vec<Slot<T>>,
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
                value: UnsafeCell::new(None),
                state: AtomicU8::new(0)
            });
        }
        Self{
            cap: size,
            data: data_vec,
            reader: AtomicUsize::new(0),
            writer: AtomicUsize::new(0),
        }
    }

    pub fn try_push(&self, value: T) -> Result<(), T> {
        if self.cap == 0 {
            return Err(value);
        }

        let reader_index = self.reader.load(Ordering::SeqCst);
        let write_index = self.writer.load(Ordering::SeqCst);

        let rb = reader_index % self.cap;
        let wb = write_index % self.cap;

        if wb == rb && reader_index + self.cap == write_index {
            return Err(value);
        }

        if let Err(_) = self.writer.compare_exchange_weak(write_index, write_index + 1, Ordering::SeqCst, Ordering::SeqCst) {
            return Err(value);
        }

        let borrowed_slot = &self.data[wb];

        borrowed_slot.state.fetch_add(1, Ordering::SeqCst);
        unsafe { *borrowed_slot.value.get() = Some(value) }
        borrowed_slot.state.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    pub fn try_pop(&self) -> Option<T> {
        if self.cap == 0 {
            return None;
        }

        let reader_index = self.reader.load(Ordering::SeqCst);
        let write_index = self.writer.load(Ordering::SeqCst);

        let rb = reader_index % self.cap;
        let wb = write_index % self.cap;

        if reader_index == write_index {
            return None;
        }

        let borrowed_slot = &self.data[rb];
        let mut data: *mut Option<T>;

        loop {
            let seq_lock = borrowed_slot.state.load(Ordering::SeqCst);
            data = borrowed_slot.value.get();
            let new_seq_lock = borrowed_slot.state.load(Ordering::SeqCst);

            if seq_lock == new_seq_lock {
                break
            }
        }

        unsafe {
            if (*data).as_ref().is_none()
            {
                return None
            }
        }

        if let Err(old_write_index) = self.reader.compare_exchange(reader_index, reader_index + 1, Ordering::SeqCst, Ordering::SeqCst) {
            return None;
        }

        let res = unsafe { (*data).take() };

        if res.is_none() {
            panic!("Data is none");
        }

        res
    }

    pub fn len(&self) -> usize {
        let write_index = self.writer.load(Ordering::SeqCst);
        let read_index = self.reader.load(Ordering::SeqCst);

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
        let buffer: ABuffer<i32> = ABuffer::<i32>::new(0);

        assert_eq!(buffer.cap(), 0 as usize);

        assert_eq!(buffer.try_push(1), Err(1));
        assert_eq!(buffer.len(), 0 as usize);
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
        assert_eq!(buffer.try_pop(), None::<i32>);

        buffer.try_push(4);

        assert_eq!(buffer.try_pop(), Some(4));
    }

    #[test]
    fn concurrent_access() {
        let buffer = Arc::new(ABuffer::<i32>::new(1));
        let buffer_arc1 = Arc::clone(&buffer);
        let buffer_arc2 = Arc::clone(&buffer);

        let mut producers = vec![];
        let mut consumers = vec![];

        let producers_count: usize = 10;
        let consumers_count: usize = 1;
        let iters_count:usize = 1;
        let should_read = Arc::new(producers_count * iters_count);

        let message_count = Arc::new(AtomicUsize::new(0));
        let pushed = Arc::new(AtomicUsize::new(0));

        let check_sum = Arc::new(AtomicUsize::new(0));
        let sum = Arc::new(producers_count * (iters_count * (iters_count - 1))/ 2 + iters_count * 100 * (producers_count * (producers_count - 1))/ 2);

        for tr in 0..producers_count {
            let tarc = Arc::clone(&buffer_arc1);
            let parc: Arc<AtomicUsize> = Arc::clone(&pushed);
            producers.push(thread::spawn(move || {
                for iter in 0..iters_count {
                    let value = tr * 100 + iter;
                    while let res = tarc.try_push(value as i32) {
                        if res == Ok(()) {
                            parc.fetch_add(1, Ordering::SeqCst);
                            break
                        }
                    }
                }
            }));
        }

        for c in 0..consumers_count {
            let tarc = Arc::clone(&buffer_arc2);
            let cacrc = Arc::clone(&message_count);
            let should_read = Arc::clone(&should_read);
            let check_sum_arc = Arc::clone(&check_sum);
            let parc: Arc<AtomicUsize> = Arc::clone(&pushed);
            consumers.push(thread::spawn(move || {
                let mut readed = cacrc.load(Ordering::SeqCst);

                while readed < *should_read {
                    if let Some(val) = tarc.try_pop() {
                        cacrc.fetch_add(1, Ordering::SeqCst);
                        check_sum_arc.fetch_add(val as usize, Ordering::SeqCst);
                    }
                    readed = cacrc.load(Ordering::SeqCst);
                    println!("Readed {} {}", readed, parc.load(Ordering::SeqCst));
                }
            }));
        }

        producers.into_iter().for_each(|p| p.join().unwrap());
        consumers.into_iter().for_each(|c| c.join().unwrap());

        assert_eq!(buffer.len(), 0);
        assert_eq!(*sum, check_sum.load(Ordering::SeqCst));
    }
}