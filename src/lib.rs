use std::fmt::Display;
pub mod sync;

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
            debug!("acheieved wait in push {} with len {}", &value, vector.len());
            vector = self.full.wait(vector).unwrap();
        }

        vector.push(value);
        self.empty.notify_one();
    }

    pub fn pop(self: &Self) -> Option<T> {
       let mut vector = self.data.lock().unwrap();

       if vector.len() <= 0 {
            debug!("acheieved wait in pop");
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