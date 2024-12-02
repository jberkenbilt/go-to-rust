use crate::rwlock::TokioLockWrapper;
use base::{AsyncRwLock, Locker, Runtime};

pub mod rwlock;

#[derive(Default, Clone)]
pub struct TokioRuntime;

impl Locker for TokioRuntime {
    fn new_lock<T: Sync + Send>(item: T) -> impl AsyncRwLock<T> {
        TokioLockWrapper::<T>::new(item)
    }
}

impl Runtime for TokioRuntime {}
