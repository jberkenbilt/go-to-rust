//! ImplBox provides a workaround for the lack of ability to assign an
//! opaque type (`impl SomeTrait`) to a struct field or use it in many
//! other places. As of rust 1.83, an impl type may appear in a
//! function parameter or return type, but you can't use it in any
//! other place, including closures, struct fields, or simple variable
//! declarations. There are various proposals for language changes
//! that would make that possible. See
//! <https://github.com/rust-lang/rust/issues/63063>.
//!
//! ImplBox works by storing the impl type as an untyped raw pointer
//! and providing a mechanism to delegate conversion of the raw
//! pointer back to a reference to a concrete implication, which can
//! return it as a reference to the impl type. In this way, it acts as
//! a proxy so that the ImplBox can be stored where you would want to
//! store the impl type reference. ImplBox uses unsafe code, but as
//! long as it is used properly, it is safe. To assist, ImplBox
//! provides some macros to generate correct code.
//!
//! # Typical Usage
//!
//! See the example for a concrete explanation with comments.
//!
//! Use ImplBox if you have a trait that has an associated function
//! that returns an impl type and you want to store the result. Let's
//! call the trait `Thing`. To use [ImplBox] as a proxy for `Thing`:
//! - Create a new trait with an associated function that returns
//!   `impl Thing`, probably by proxying to an associated function in
//!   `Thing`. Let's call it `ThingMaker`.
//! - In the Trait's declaration, declare a method whose name starts
//!   with `new_` and that returns an opaque type, e.g. `new_thing()
//!   -> impl Thing`. It can take any additional arguments that may be
//!   required.
//! - Annotate the declaration with `#[implbox_decl]`. If your
//!   function is called `new_thing`, this will create `box_thing`,
//!   `unbox_thing`, and `drop_thing`.
//! - In the implementation of `ThingMaker` for some concrete type,
//!   annotate the implementation of `new_thing` with
//!   `#[implbox_impls]`.
//! - In code that needs to use `&impl Thing`:
//!   - Call `new_thing_implbox` instead of `new_thing`. This returns
//!     an `ImplBox`, which can be stored anywhere.
//!   - To get the `&impl Thing`, call the associated
//!     `unbox_new_thing` method with a reference to the `ImplBox`.
//!     This returns a reference to the thing. It is useful to create
//!     a separate method that does this.
//!   - You never call `drop_thing` -- it is called automatically when
//!     the `ImplBox` is dropped.
//!
//! The [ImplBox] type has a generic type parameter. There is no
//! specifically defined relationship between that type and the type
//! the [ImplBox] is proxying. The type can never be the exact type
//! since `impl SomeTrait` is not a concrete type. The generic type
//! for [ImplBox] is used in the following ways:
//! - A specific [ImplBox] will be `Sync` and `Send` if and only if
//!   the generic type is `Sync` and `Send`
//! - Using a unique type for each thing you are storing in an
//!   [ImplBox] enables you to get a compile error if you try to pass
//!   an [ImplBox] to the wrong unbox method. There is also a runtime
//!   check for this in case you get it wrong. Therefore, a good
//!   strategy is to create a unique "shadow type" with the same
//!   generics, if any, as the type you are boxing. That way, there
//!   will be an exact, one-to-one correspondence between a concrete
//!   instantiation of `ImplBox` and the type it contains. This
//!   ensures that you will get a compile error if you try to convert
//!   an `ImplBox` to the wrong type, and along with the macros,
//!   guarantees safety of `ImplBox`.
//!
//! # Safety
//! - You must only convert an ImplBox's pointer back to the concrete
//!   type that it originally came from. This can only be done by the
//!   concrete type.
//! - You must not do anything with the pointer that wouldn't be
//!   allowed by borrowing rules, such as returning a mutable
//!   reference from an immutable ImplBox.
//! - You must use a generic type with [ImplBox] whose `Sync`/`Send`
//!   status is the same as for the concrete trait implementation that
//!   you are storing.
//! - Using the macros and shadow type as described above guarantees
//!   that all these constraints are satisfied. To prevent
//!   accidentally passing an `ImplBox` to the wrong concrete
//!   implementation of the trait, runtime checks using `TypeId`
//!   supplement these compile-time checks and would be sufficient if
//!   the compile-time helper types were used incorrectly.
//!
//! # Example
//! ```
//! use implbox::ImplBox;
//! use implbox_macros::{implbox_decls, implbox_impls};
//! use std::marker::PhantomData;
//!
//! // This generic trait has an associated function that returns an
//! // impl type. A concrete Food type would implement this trait.
//! trait Food<T: Clone> {
//!     fn new(prep: T) -> impl Food<T>;
//!     fn prep(&self) -> T;
//! }
//!
//! // Here's a concrete food implementation.
//! struct Potato<T> {
//!     prep: T,
//! }
//! impl<T: Clone> Food<T> for Potato<T> {
//!     fn new(prep: T) -> impl Food<T> {
//!         Self { prep }
//!     }
//!
//!     fn prep(&self) -> T {
//!         self.prep.clone()
//!     }
//! }
//!
//! // We can't store `&impl Food<T>` in a struct field, so enhance
//! // `Food` by gluing it to `ImplBox`.
//!
//! // Create a dummy type that to provide extra compile-time
//! // checking. By using `FoodBox<T>` as the generic type for any
//! // `ImplBox` that actually holds some `Food<T>`, we make it a
//! // compile error if we try to get a concrete `Food` out of the
//! // wrong type of `ImplBox`. Runtime checks using `TypeId` ensure
//! // that we actually have the right concrete type.
//! struct FoodBox<T>(PhantomData<T>);
//!
//! // This trait provides the glue between the original trait and the
//! // ImplBox. For each concrete implementation of the original
//! // trait, create a corresponding concrete implementation for the
//! // helper that creates the corresponding concrete implementation
//! // of the trait. Note the use of the implbox macros in declaring
//! // and defining the methods. All we have to supply is some proxy
//! // around the `Food` constructor. It has to be called
//! // `new_something`. Note that `FoodHelper` is not a generic type.
//! // Since the generic parameter is attached to `new_food`, a type
//! // that implements `FoodHelper` can create a `Food` of any type.
//! // The generic type in `FoodBox<T>` comes from the <T> of
//! // `new_food`. The argument to `implbox_decls` is the generic type
//! // of `ImplBox`. If it has any of its own generics, they are taken
//! // from the generics of the `new` function that is being
//! // annotated.
//! trait FoodHelper {
//!     #[implbox_decls(FoodBox<T>)]
//!     fn new_food<T: Clone>(prep: T) -> impl Food<T>;
//! }
//!
//! // We need a concrete `FoodHelper` for each concrete `Food`
//! // implementation. The arguments to `implbox_impls` are the
//! // generic type for the `ImplBox` and the concrete type that is
//! // being stored.
//! struct PotatoHelper;
//! impl FoodHelper for PotatoHelper {
//!     #[implbox_impls(FoodBox<T>, Potato<T>)]
//!     fn new_food<T: Clone>(prep: T) -> impl Food<T> {
//!         Potato::new(prep)
//!     }
//! }
//!
//! // Here's a struct that holds an impl type of Food. We can't make
//! // the field `food` have type `&impl Food<String>`, so we make it
//! // have type `ImplBox<FoodBox<String>>` instead. See how we use
//! // `FoodBox`. The `ImplBox` can't be `ImplBox<impl Food<String>>`
//! // because impl types are not valid in that position, and it can't
//! // be the concrete type because we don't know what the concrete
//! // type is in the trait declaration. We can't just make `FoodT` a
//! // generic type and store a `FoodT` directly because want
//! // `FoodHelper` to have to know at compile time what types of
//! // items it will create.
//! struct Refrigerator<FoodHelperT: FoodHelper> {
//!     food: ImplBox<FoodBox<String>>,
//!     _f: PhantomData<FoodHelperT>,
//! }
//! // Add a convenience method to get the food out.
//! impl<FoodHelperT: FoodHelper> Refrigerator<FoodHelperT> {
//!     fn food(&self) -> &(impl Food<String> + '_) {
//!         FoodHelperT::unbox_food(&self.food)
//!     }
//! }
//!
//! // This shows how to use it. Instead of storing the return value
//! // of `new_food` in a field of type `impl Food`, we store the
//! // return value of `box_food` in a field of type `ImplBox`. Then
//! // we ask the concrete type to get the impl back out.
//! let r = Refrigerator::<PotatoHelper> {
//!     food: PotatoHelper::box_food("baked".to_string()),
//!     _f: Default::default(),
//! };
//! // If `food` where `impl Food`, we could just call
//! // `r.food.prep()`. Instead, we call `r.food().prep()` to
//! // indirect through the ImplBox.
//! assert_eq!(r.food().prep(), "baked");
//! ```

use std::any::TypeId;
use std::marker::PhantomData;

unsafe impl<T: Send> Send for ImplBox<T> {}
unsafe impl<T: Sync> Sync for ImplBox<T> {}
pub struct ImplBox<T> {
    id: TypeId,
    ptr: *const (),
    destroy: fn(*const ()),
    _t: PhantomData<T>,
}
impl<T> ImplBox<T> {
    pub fn new(id: TypeId, destroy: fn(*const ()), ptr: *const ()) -> Self {
        Self {
            id,
            ptr,
            destroy,
            _t: Default::default(),
        }
    }

    pub fn with<F, Ret>(&self, id: TypeId, f: F) -> Ret
    where
        F: FnOnce(*const ()) -> Ret,
    {
        if self.id == id {
            f(self.ptr)
        } else {
            panic!("id mismatch");
        }
    }
}
impl<T> Drop for ImplBox<T> {
    fn drop(&mut self) {
        (self.destroy)(self.ptr);
    }
}
