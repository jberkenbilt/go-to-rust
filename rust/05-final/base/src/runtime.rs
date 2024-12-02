use implbox::ImplBox;
use implbox_macros::implbox_decls;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub trait Runtime: Locker {}

/// The [AsyncRwLock::read] and [AsyncRwLock::write] functions must return
/// actual async-aware lock guards that maintain the lock until they are out of
/// scope. They must not block the thread while holding the lock.
pub trait AsyncRwLock<T> {
    fn new(item: T) -> Self;
    fn read(
        &self,
    ) -> impl std::future::Future<Output = impl Deref<Target = T> + Sync + Send> + Send;
    fn write(
        &self,
    ) -> impl std::future::Future<Output = impl DerefMut<Target = T> + Sync + Send> + Send;
}

/// This is an empty structure that we use as the generic type for ImplBox.
pub struct LockBox<T>(PhantomData<T>);
/// This trait glues ImplBox to AsyncRwLock and enables creation of AsyncRwLocks
/// of any type.
pub trait Locker {
    #[implbox_decls(LockBox<T>)]
    fn new_lock<T: Sync + Send>(item: T) -> impl AsyncRwLock<T>;
}
