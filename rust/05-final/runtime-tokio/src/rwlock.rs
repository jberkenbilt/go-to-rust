use base::AsyncRwLock;
use std::ops::{Deref, DerefMut};
use tokio::sync;

#[derive(Default)]
pub struct TokioLockWrapper<T> {
    lock: sync::RwLock<T>,
}

impl<T: Sync + Send> AsyncRwLock<T> for TokioLockWrapper<T> {
    fn new(item: T) -> Self {
        TokioLockWrapper {
            lock: sync::RwLock::new(item),
        }
    }

    async fn read(&self) -> impl Deref<Target = T> + Sync + Send {
        self.lock.read().await
    }

    async fn write(&self) -> impl DerefMut<Target = T> + Sync + Send {
        self.lock.write().await
    }
}

#[cfg(test)]
mod tests;
