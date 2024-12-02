// TODO: Use stable features only
#![feature(async_closure)]
//! This is a simple function-based wrapper around [Controller] that
//! operates on a singleton. You must call [init] first, and then you
//! can call the other functions, which call methods on the singleton.

use controller::Controller;
use std::error::Error;
use std::sync::{LazyLock, RwLock};

struct Wrapper {
    rt: tokio::runtime::Runtime,
    controller: RwLock<Option<Controller>>,
}

static CONTROLLER: LazyLock<Wrapper> = LazyLock::new(|| Wrapper {
    rt: tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap(),
    controller: Default::default(),
});

/// This is a generic dispatcher that is used by the wrapper API to
/// call methods on the singleton. It takes a closure that takes a
/// &[Controller] and an arg, calls the closure using the singleton,
/// and returns the result.
fn run_method<ArgT, ResultT, FnT>(
    f: FnT,
    arg: ArgT,
) -> Result<ResultT, Box<dyn Error + Sync + Send>>
where
    // 2024-11-27: `async FnOnce` and `AsyncFnOnce` are unstable, and it is not yet decided
    // which syntax will win. See
    // - https://blog.rust-lang.org/inside-rust/2024/08/09/async-closures-call-for-testing.html
    // - https://rust-lang.zulipchat.com/#narrow/stream/213817-t-lang/topic/Async.20closures.20bounds.20syntax
    FnT: async FnOnce(&Controller, ArgT) -> Result<ResultT, Box<dyn Error + Sync + Send>>,
    // OR:
    // FnT: std::ops::AsyncFnOnce(&Controller, ArgT) -> Result<ResultT, Box<dyn Error + Sync + Send>>,
{
    let lock = CONTROLLER.controller.read().unwrap();
    let Some(controller) = &*lock else {
        return Err("call init first".into());
    };
    CONTROLLER.rt.block_on(f(controller, arg))
}

pub fn init() {
    let mut controller = CONTROLLER.controller.write().unwrap();
    *controller = Some(Controller::new());
}

pub fn one(val: i32) -> Result<i32, Box<dyn Error + Sync + Send>> {
    run_method(Controller::one, val)
}

pub fn two(val: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    run_method(Controller::two, val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        // This is a duplication of the controller test using the
        // wrapper API.
        assert_eq!(two("quack").err().unwrap().to_string(), "call init first");
        init();
        assert_eq!(one(5).unwrap(), 1);
        assert_eq!(one(3).err().unwrap().to_string(), "sorry, not that one");
        assert_eq!(two("potato").unwrap(), "two?val=potato&seq=2");
    }
}