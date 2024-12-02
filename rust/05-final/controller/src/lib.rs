//! This is an internal implementation of sample API. The
//! implementation pretends to make network calls and accesses locked
//! data. It is wrapped by a function-based API that operates a
//! singleton.
use base::{AsyncRwLock, LockBox, Runtime};
use implbox::ImplBox;
use std::error::Error;
use std::marker::PhantomData;
use std::ops::DerefMut;

#[derive(Default)]
struct ReqData {
    seq: i32,
    last_path: String,
}

pub struct Controller<RuntimeT: Runtime> {
    req_data: ImplBox<LockBox<ReqData>>,
    _r: PhantomData<RuntimeT>,
}

impl<RuntimeT: Runtime> Default for Controller<RuntimeT> {
    fn default() -> Self {
        Self {
            req_data: RuntimeT::box_lock(Default::default()),
            _r: Default::default(),
        }
    }
}

impl<RuntimeT: Runtime> Controller<RuntimeT> {
    pub fn new() -> Self {
        Default::default()
    }

    fn req_data(&self) -> &(impl AsyncRwLock<ReqData> + '_) {
        RuntimeT::unbox_lock(&self.req_data)
    }

    async fn request(&self, path: &str) -> Result<(), Box<dyn Error + Sync + Send>> {
        let mut lock = self.req_data().write().await;
        let ref_data: &mut ReqData = lock.deref_mut();
        ref_data.seq += 1;
        // A real implementation would make a network call here. Call await to make this
        // non-trivially async.
        async {
            ref_data.last_path = format!("{path}&seq={}", ref_data.seq);
        }
        .await;
        Ok(())
    }

    /// Send a request and return the sequence of the request.
    pub async fn one(&self, val: i32) -> Result<i32, Box<dyn Error + Sync + Send>> {
        if val == 3 {
            return Err("sorry, not that one".into());
        }
        self.request(&format!("one?val={val}")).await?;
        Ok(self.req_data().read().await.seq)
    }

    /// Send a request and return the path of the request.
    pub async fn two(&self, val: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
        self.request(&format!("two?val={val}")).await?;
        Ok(self.req_data().read().await.last_path.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_tokio::TokioRuntime;

    #[tokio::test]
    async fn test_basic() {
        let c = Controller::<TokioRuntime>::new();
        assert_eq!(c.one(5).await.unwrap(), 1);
        assert_eq!(
            c.one(3).await.err().unwrap().to_string(),
            "sorry, not that one"
        );
        assert_eq!(c.two("potato").await.unwrap(), "two?val=potato&seq=2");
    }
}
