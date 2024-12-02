// Higher-ranked trait bounds (HRTB)

// Concepts

// - Monomorphic Function -- a function whose arguments are specified static
//   types. These are "normal", non-generic functions.
// - Polymorphic Function -- a function with an argument whose type is
//   determined at the time it is called. In rust, polymorphic functions are
//   functions that take at least one generic argument.
// - Higher-order function -- a function that takes other functions are
//   arguments
// - Rank
//   - All monomorphic functions have rank 0
//   - A polymorphic function that has no arguments that are polymorphic
//     functions has rank 1
//   - A higher-order polymorphic function's rank is 1 more than the highest
//     rank of its polymorphic arguments. In other words, if a polymorphic
//     function has generic types but none of those types are functions, it is
//     rank 1. If a polymorphic function has a generic argument constrained to
//     be a polymorphic function of rank 1, it has rank 2. See examples.

// Case 1: A simple polymorphic function with no lifetimes (check_len)
// is called by a higher-order function (call_polymorphic1). The type F in
// call_polymorphic1 is a higher-ranked type because it is defined in terms of
// another generic type.

// This is a polymorphic function of rank 1. It polymorphic because it has an
// argument of generic type `T`. It is rank 1 because type none of its arguments
// are polymorphic functions. This function returns true if the given type
// reference a string of the given length.
pub fn check_len_rank1<T: AsRef<str>>(v: T, len: usize) -> bool {
    v.as_ref().len() == len
}

// This is a polymorphic function of rank 2. It is polymorphic because it has
// generic arguments. It is rank 2 because generic type `F` is a polymorphic
// function of rank 1. `F` is polymorphic because it has an argument of generic
// type `T` and is rank 1 because `T` is not a function.
pub fn rank2<F, T, Ret>(f: F, v: T, len: usize) -> Ret
where
    F: FnOnce(T, usize) -> Ret,
{
    f(v, len)
}

// Case 2: A polymorphic function that requires a shorter lifetime than the
// caller's scope.

// This is monomorphic but has an explicit lifetime.
pub trait WithLifetime<'a> {
    fn check_len(&'a self, len: usize) -> bool;
}
impl<'a> WithLifetime<'a> for &str {
    fn check_len(&'a self, len: usize) -> bool {
        check_len_rank1(self, len)
    }
}

// This function has rank 1 it is polymorphic, and none of its arguments are
// polymorphic functions. The trait bound makes `T` act like a function since we
// can call its methods. The `WithLifetime` trait requires a lifetime because it
// was defined that way. What lifetime do we specify? We are moving `v` into the
// function, so `v` will not live as long as the caller's scope. That means
// hanging the lifetime off of the definition of the function will specify a
// lifetime that's too long. Using the `for<'a>` syntax allows us to define a
// lifetime `'a` whose scope is not connected with the caller's lifetime.
pub fn call_with_lifetime<T>(v: T, len: usize) -> bool
where
    for<'a> T: WithLifetime<'a>,
{
    v.check_len(len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hrtb() {
        let s = "quack".to_string();
        assert!(rank2(check_len_rank1, &s, 5));
        assert!(call_with_lifetime(s.as_ref(), 5));
    }
}
