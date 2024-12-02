use super::*;
use crate::{Locker, TokioRuntime};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task;

async fn generic_thing<M>(m: &M)
where
    M: AsyncRwLock<i32>,
{
    {
        // Hold lock across an await point. We don't get warnings for this, and
        // as long as RwLock is implemented using an async-aware RwLock, we're
        // fine.
        let lock = m.read().await;
        // non-Send Future
        async move { std::ptr::null::<*const ()>() }.await;
        assert_eq!(*lock, 3);
    }
    {
        let mut lock = m.write().await;
        // non-Send Future
        async move { std::ptr::null::<*const ()>() }.await;
        *lock = 4;
    }
    {
        let lock = m.read().await;
        assert_eq!(*lock, 4);
        async move {}.await;
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_basic() {
    let m1 = Arc::new(TokioRuntime::new_lock(3));
    generic_thing(m1.as_ref()).await;
    let m2 = m1.clone();
    assert_eq!(*m1.read().await, 4);
    let h = task::spawn(async move {
        let mut lock = m2.write().await;
        // non-Send Future
        async move { std::ptr::null::<*const ()>() }.await;
        *lock = 5;
        1
    });
    assert_eq!(1, h.await.unwrap());
    let lock = m1.read().await;
    assert_eq!(*lock, 5);
}

#[tokio::test(flavor = "current_thread")]
async fn test_lock() {
    // Exercise non-trivial case of waiting for a lock.
    let m1 = Arc::new(TokioRuntime::new_lock(5));
    let (tx, rx) = oneshot::channel::<()>();
    let m2 = m1.clone();
    let h1 = task::spawn(async move {
        // Grab the lock first, then signal to the other task.
        let mut lock = m2.write().await;
        tx.send(()).unwrap();
        // We got the lock first. The other side can't progress.
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(*lock, 5);
        *lock = 10;
        // When we finish, we automatically release the lock.
    });
    let m2 = m1.clone();
    let h2 = task::spawn(async move {
        // Wait for the first the channel, and then grab the lock.
        rx.await.unwrap();
        // Try to get the lock. This will "block" (yield to the runtime) until
        // the lock is available.
        let mut lock = m2.write().await;
        // The other side has finished.
        assert_eq!(*lock, 10);
        *lock = 11;
    });
    // Wait for the jobs to finish.
    h1.await.unwrap();
    h2.await.unwrap();
    let lock = m1.read().await;
    assert_eq!(*lock, 11);
}
