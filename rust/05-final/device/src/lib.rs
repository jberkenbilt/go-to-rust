//! This is a simple function-based wrapper around [Controller] that
//! operates on a singleton. You must call [init] first, and then you
//! can call the other functions, which call methods on the singleton.

use controller::Controller;
use runtime_tokio::TokioRuntime;
use std::error::Error;
use std::future::Future;
use std::sync::{LazyLock, RwLock};

struct Wrapper {
    rt: tokio::runtime::Runtime,
    controller: RwLock<Option<Controller<TokioRuntime>>>,
}

static CONTROLLER: LazyLock<Wrapper> = LazyLock::new(|| Wrapper {
    rt: tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap(),
    controller: Default::default(),
});

// We want to create a dispatcher that blocks on an async method call.
// At the time of this writing (latest nightly rust = 1.84), async
// closures are not stable, but with the `async_closure` feature and a
// nightly build, this solution, using `async FnOnce` (or
// `AsyncFnOnce` -- it is not yet determined which syntax will win)
// and a higher-ranked trait bound, works:

// fn run_method<ArgT, ResultT, FnT>(
//     f: FnT,
//     arg: ArgT,
// ) -> Result<ResultT, Box<dyn Error + Sync + Send>>
// where
//     FnT: async FnOnce(&Controller, ArgT) -> Result<ResultT, Box<dyn Error + Sync + Send>>,
//     // OR:
//     // FnT: std::ops::AsyncFnOnce(&Controller, ArgT) -> Result<ResultT, Box<dyn Error + Sync + Send>>,
// {
//     let lock = CONTROLLER.controller.read().unwrap();
//     let Some(controller) = &*lock else {
//         return Err("call init first".into());
//     };
//     CONTROLLER.rt.block_on(f(controller, arg))
// }

// For more information about that, see
// - https://blog.rust-lang.org/inside-rust/2024/08/09/async-closures-call-for-testing.html
// - https://rust-lang.zulipchat.com/#narrow/stream/213817-t-lang/topic/Async.20closures.20bounds.20syntax
//

// In the meantime, we can try the standard workaround of specifying
// the function type and the future type as two separate generic
// types, as in this:

// fn run_method<ArgT, ResultT, FnT, Fut>(
//     f: FnT,
//     arg: ArgT,
// ) -> Result<ResultT, Box<dyn Error + Sync + Send>>
// where
//     FnT: FnOnce(&Controller, ArgT) -> Fut,
//     Fut: Future<Output=Result<ResultT, Box<dyn Error + Sync + Send>>>,
// {
//     let lock = CONTROLLER.controller.read().unwrap();
//     let Some(controller) = &*lock else {
//         return Err("call init first".into());
//     };
//     CONTROLLER.rt.block_on(f(controller, arg))
// }

// This doesn't work. We get an error on the method calls that "one
// type is more general than the other" with a suggestion of using a
// higher-ranked trait bound. So what's the actual problem?
//
// Our dispatcher has three lifetimes:
// - The outer lifetime, which is the default lifetime of references
//   passed into the dispatcher
// - The lifetime of the controller object, which is shorter than the
//   outer lifetime since the controller is a reference to the item
//   inside the mutex
// - The lifetime captured by the future.
//
// fn dispatcher() {                                     <-+
//     let fut = obj.method(arg);                          |
//         ^     ^                                         |
//         |     +---- object that contains the method '2  |-- outer '1
//         +---------- future '3                           |
// }                                                     <-+
//
// As written, the lifetime of the `Controller` arg to `f` has the
// outer lifetime '1, but the controller doesn't live that long
// because it is actually created locally inside the call to the
// dispatcher. We need to use a higher-ranked trait bound (HRTB) to
// disconnect the lifetime of the controller from the outer lifetime.
// The problem is that we can't use a higher-ranked trait bound for
// `FnT` because we need `FnT` and `Fut` to share a lifetime. We want
// something like this:
//
// for <'a> {
//     FnT: FnOnce(&Controller, ArgT) -> Fut,
//     Fut: Future<Output=Result<ResultT, Box<dyn Error + Sync + Send>>>,
// }
//
// but there is no such syntax. So how can we create a higher rank
// trait bound that applies to both trait bounds?
//
// The solution is to create a custom trait that extends FnOnce and
// has an associated type that carries the Future's output type. If we
// put a lifetime on that trait, it will apply to the whole thing.
// Then we can use HRTB with that trait.
//
// The MethodCaller trait does not other than to apply the lifetime
// associated with the trait to the controller and tie it to the
// future. Since we need a concrete implementation, we provide a
// trivial blanket implementation that just includes a parameter with
// the same bounds as the associated type and then uses it as the
// associated type. Now we can attach our HRTB to a parameter bound by
// _this_ trait, and the lifetime will apply to the controller and the
// future together. Effectively, this makes '2 and '3 above the same
// as each other and distinct from '1.

trait MethodCaller<'a, ArgT, ResultT>: FnOnce(&'a Controller<TokioRuntime>, ArgT) -> Self::Fut {
    type Fut: Future<Output = Result<ResultT, Box<dyn Error + Sync + Send>>>;
}
impl<
        'a,
        ArgT,
        ResultT,
        FnT: FnOnce(&'a Controller<TokioRuntime>, ArgT) -> Fut,
        Fut: Future<Output = Result<ResultT, Box<dyn Error + Sync + Send>>>,
    > MethodCaller<'a, ArgT, ResultT> for FnT
{
    type Fut = Fut;
}

/// This is a generic dispatcher that is used by the wrapper API to
/// call methods on the singleton. It takes a closure that takes a
/// &[Controller] and an arg, calls the closure using the singleton,
/// and returns the result. The [MethodCaller] trait ties the lifetime
/// of the controller to the lifetime of the Future.
fn run_method<ArgT, ResultT, FnT>(
    f: FnT,
    arg: ArgT,
) -> Result<ResultT, Box<dyn Error + Sync + Send>>
where
    for<'a> FnT: MethodCaller<'a, ArgT, ResultT>,
    // Some day, one of these will work:
    // FnT: async FnOnce(&Controller, ArgT) -> Result<ResultT, Box<dyn Error + Sync + Send>>,
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
