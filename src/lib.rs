use std::fmt::Display;
pub mod sync;
pub mod abuffer;

use crate::sync::{Mutex, Condvar};
use log::{debug};

pub struct Buffer <T> 
    where T: Display
{
    data: Mutex<Vec<T>>,
    full: Condvar,
    empty: Condvar
}

impl <T> Buffer<T> 
where T: Display
{
    pub fn new(size: usize) -> Self {
        Self{
            data: Mutex::new(Vec::with_capacity(size)),
            full: Condvar::new(),
            empty: Condvar::new()
        }
    }

    pub fn push(self: &Self, value: T) {
        let mut vector = self.data.lock().unwrap();

        if vector.len() >= vector.capacity() {
            // debug!("acheieved wait in push {} with len {}", &value, vector.len());
            vector = self.full.wait(vector).unwrap();
        }

        vector.push(value);
        self.empty.notify_one();
    }

    pub fn pop(self: &Self) -> Option<T> {
       let mut vector = self.data.lock().unwrap();

       if vector.len() <= 0 {
            // debug!("acheievedgss wait in pop");
            vector = self.empty.wait(vector).unwrap();
       }

       let res = vector.pop();
       self.full.notify_one();
       res
    }

    pub fn len(&self) -> usize {
        let vector = self.data.lock().unwrap();

        vector.len()
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_buffer() {
        let t = super::Buffer::<i32>::new(3);

        t.push(1);
        t.push(2);
        t.push(3);

        assert_eq!(t.len(), 3);

        let v = t.pop();
        assert_eq!(v, Some(3));

        let v = t.pop();
        assert_eq!(v, Some(2));

        let v = t.pop();
        assert_eq!(v, Some(1));

        assert_eq!(t.len(), 0);
    }
}