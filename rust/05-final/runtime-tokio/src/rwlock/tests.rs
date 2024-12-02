use super::*;
use crate::TokioRuntime;
use base::{LockBox, Locker};
use implbox::ImplBox;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task;

struct Thing<LockerT: Locker> {
    lock: ImplBox<LockBox<i32>>,
    _l: PhantomData<LockerT>,
}
impl<LockerT: Locker> Thing<LockerT> {
    fn new(item: i32) -> Self {
        Self {
            lock: LockerT::box_lock(item),
            _l: Default::default(),
        }
    }
    fn lock(&self) -> &(impl AsyncRwLock<i32> + '_) {
        LockerT::unbox_lock(&self.lock)
    }
    async fn do_thing(&self) -> i32 {
        let mut m = self.lock().write().await;
        async move { std::ptr::null::<*const ()>() }.await;
        *m += 1;
        *m
    }
}

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
    let l1 = Arc::new(TokioRuntime::box_lock(3));
    let m1 = TokioRuntime::unbox_lock(l1.as_ref());
    generic_thing(m1).await;
    let l2 = l1.clone();
    assert_eq!(*m1.read().await, 4);
    let h = task::spawn(async move {
        let m2 = TokioRuntime::unbox_lock(l2.as_ref());
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

#[tokio::test(flavor = "current_thread")]
async fn test_locker() {
    let th = Thing::<TokioRuntime>::new(3);
    let m = TokioRuntime::unbox_lock(&th.lock);
    generic_thing(m).await;
    assert_eq!(th.do_thing().await, 5);
    async {}.await;
    assert_eq!(th.do_thing().await, 6);
}
