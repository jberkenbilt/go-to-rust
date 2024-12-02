use crate::rwlock::TokioLockWrapper;
use base::{AsyncRwLock, LockBox, Locker, Runtime};
use implbox::ImplBox;
use implbox_macros::implbox_impls;

pub mod rwlock;

#[derive(Default, Clone)]
pub struct TokioRuntime;

impl Locker for TokioRuntime {
    #[implbox_impls(LockBox<T>, TokioLockWrapper<T>)]
    fn new_lock<T: Sync + Send>(item: T) -> impl AsyncRwLock<T> {
        TokioLockWrapper::<T>::new(item)
    }
}

impl Runtime for TokioRuntime {}
