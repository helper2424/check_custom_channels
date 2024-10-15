#[cfg(loom)]
pub(crate) use loom::sync::{Condvar, Mutex, Arc};
#[cfg(loom)]
pub(crate) use loom::sync::atomic::{AtomicUsize, Ordering, AtomicU8};
#[cfg(loom)]
pub(crate) use loom::sync::atomic::AtomicPtr;
#[cfg(loom)]
pub(crate) use loom::thread;

#[cfg(not(loom))]
pub(crate) use std::sync::{Mutex, Condvar, Arc};
#[cfg(not(loom))]
pub(crate) use std::sync::atomic::{AtomicUsize, Ordering, AtomicU8};
#[cfg(not(loom))]
pub(crate) use std::sync::atomic::AtomicPtr;
#[cfg(not(loom))]
pub(crate) use std::thread;
