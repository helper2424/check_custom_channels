#[cfg(loom)]
pub(crate) use loom::sync::{Condvar, Mutex};

#[cfg(not(loom))]
pub(crate) use std::sync::{Mutex, Condvar};