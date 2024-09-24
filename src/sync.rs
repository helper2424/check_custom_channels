#[cfg(loom)]
pub(crate) use loom::sync::{Condvar, Mutex};
#[cfg(loom)]
pub(crate) use loom::sync::atomic::{AtomicUsize, Ordering, AtomicU8};
#[cfg(loom)]
pub(crate) use loom::cell::RefCell;

#[cfg(not(loom))]
pub(crate) use std::sync::{Mutex, Condvar, Arc};
#[cfg(not(loom))]
pub(crate) use std::sync::atomic::{AtomicUsize, Ordering, AtomicU8};
#[cfg(not(loom))]
pub(crate) use std::cell::RefCell;