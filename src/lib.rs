use std::any::Any;
use std::fmt;

pub trait DynEq: DynEqHelper {
    fn as_any(&self) -> &dyn Any;
    fn as_dyn_eq_helper(&self) -> &dyn DynEqHelper;
    fn level_one(&self, arg2: &dyn DynEqHelper) -> bool;
}

pub trait DynEqHelper {
    fn level_two(&self, arg1: &dyn DynEq) -> bool;
}

impl<T> DynEq for T
where
    T: PartialEq + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_dyn_eq_helper(&self) -> &dyn DynEqHelper {
        self
    }

    fn level_one(&self, arg2: &dyn DynEqHelper) -> bool {
        arg2.level_two(self)
    }
}

impl<T> DynEqHelper for T
where
    T: PartialEq + 'static,
{
    fn level_two(&self, arg1: &dyn DynEq) -> bool {
        if let Some(other) = arg1.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
}

impl PartialEq for dyn DynEq {
    fn eq(&self, other: &Self) -> bool {
        self.level_one(other.as_dyn_eq_helper())
    }
}

/// A trait that represents opaque data. In this simplified example it
/// seems a bit useless, but in my codebase, I also require OpaqueData
/// to implement Serialize and Deserialize.
pub trait OpaqueData: fmt::Debug + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_dyn_eq(&self) -> &dyn DynEq;
}

impl PartialEq for dyn OpaqueData {
    fn eq(&self, other: &Self) -> bool {
        // Interestingly, we cannot do
        //
        // self.as_dyn_eq() == other.as_dyn_eq()
        //
        // because PartialEq is implemented for `dyn DynEq` (which is
        // 'static) but `as_dyn_eq` returns a `&dyn DynEq`.
        // Therefore rustc rejects this with:
        //
        //   --> code/opaque/src/lib.rs:32:9
        //    |
        // 29 |     fn eq(&self, other: &Self) -> bool {
        //    |           -----
        //    |           |
        //    |           `self` declared here, outside of the associated function body
        //    |           `self` is a reference that is only valid in the associated function body
        //    |           let's call the lifetime of this reference `'1`
        // ...
        // 32 |         self.as_dyn_eq() == other.as_dyn_eq()
        //    |         ^^^^^^^^^^^^^^^^
        //    |         |
        //    |         `self` escapes the associated function body here
        //    |         argument requires that `'1` must outlive `'static`
        //
        self.as_dyn_eq()
            .level_one(other.as_dyn_eq().as_dyn_eq_helper())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone)]
    struct A {
        i: u8,
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    struct B;

    impl OpaqueData for A {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_dyn_eq(&self) -> &dyn DynEq {
            self
        }
    }

    impl OpaqueData for B {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_dyn_eq(&self) -> &dyn DynEq {
            self
        }
    }

    #[test]
    fn test() {
        let a = A { i: 1 };

        // Creation of a blackbox
        let dyn_a: Arc<dyn OpaqueData> = Arc::new(a.clone());

        // Downcasting to the right type
        let downcasted = dyn_a.as_any().downcast_ref::<A>().unwrap();
        assert_eq!(&a, downcasted);

        // Downcasting to the wrong type
        assert!(dyn_a.as_any().downcast_ref::<B>().is_none());

        // // Test equality properties
        // let a_eq : Arc<dyn OpaqueData> = Arc::new(A { i: 1 });
        // let a_ne : Arc<dyn OpaqueData> = Arc::new(A { i: 2 });

        // assert_eq!(&a_eq, &dyn_a);
        // assert_ne!(&a_ne, &dyn_a);
    }
}
