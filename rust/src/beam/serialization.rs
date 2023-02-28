use std::any::Any;
use crate::error::Fallible;
use crate::ffi::util::Type;

pub trait Operation<A, R> {
    fn into_closure(self) -> Box<dyn Fn(&A) -> R>;
    fn serialize(&self) -> Serialization;
    fn deserializer(&self) -> fn(&Serialization) -> Fallible<Box<dyn Any>>;
}

pub struct Serialization {
    E: Type,
    A: Type,
    R: Type,
    env: String,
}
impl Serialization {
    pub fn new(E: Type, A: Type, R: Type, env: String) -> Self {
        Self { E, A, R, env }
    }
}

pub struct ConcreteFn<A, R>(Box<dyn Fn(&A) -> R>);

pub struct StructOperation<A, R> {
    closure: Box<dyn Fn(&A) -> R>,
    serializer: Box<dyn Fn(&Self) -> Serialization>,
    deserializer: fn(&Serialization) -> Fallible<Box<dyn Any>>,
}
impl<A: 'static, R: 'static> StructOperation<A, R> {
    pub fn new(closure: Box<dyn Fn(&A) -> R>, serializer: Box<dyn Fn(&Self) -> Serialization>, deserializer: fn(&Serialization) -> Fallible<Box<dyn Any>>) -> Self {
        Self { closure, serializer, deserializer }
    }
}
impl<A, R> Operation<A, R> for StructOperation<A, R> {
    fn into_closure(self) -> Box<dyn Fn(&A) -> R> {
        self.closure
    }
    fn serialize(&self) -> Serialization {
        (self.serializer)(&self)
    }
    fn deserializer(&self) -> fn(&Serialization) -> Fallible<Box<dyn Any>> {
        self.deserializer
    }
}

pub fn go_internal<A, R, O: Operation<A, R>>(arg: Vec<A>, op: O) -> Vec<R> {
    let f = op.into_closure();
    arg.iter().map(f).collect()
}

pub fn go_external<A: 'static, R: 'static, O: Operation<A, R>>(arg: Vec<A>, op: O) -> Vec<R> {
    let deserializer = op.deserializer();
    let serialized = op.serialize();
    let f = (deserializer)(&serialized).unwrap();
    let f: ConcreteFn<A, R>  = *f.downcast().unwrap();
    arg.iter().map(f.0).collect()
}

pub mod m1 {
    use std::any::Any;
    use serde::Deserialize;
    use crate::beam::serialization::{ConcreteFn, Serialization, StructOperation};
    use crate::error::Fallible;
    use crate::ffi::util::Type;
    use crate::traits::Number;

    pub fn make_mul<T: Number>(x: T) -> StructOperation<T, T> {
        fn make_closure<T: Number>(env: T) -> Box<dyn Fn(&T) -> T> {
            Box::new(move |x| *x * env)
        }
        let closure = make_closure::<T>(x);
        let serializer = Box::new(|s: &StructOperation<T, T>| {
            Serialization::new(Type::of::<T>(), Type::of::<T>(), Type::of::<T>(), serde_json::to_string(&s.env).unwrap())
        });
        fn deserializer(serialization: &Serialization) -> Fallible<Box<dyn Any>> {
            fn monomorphize<'a, T: Number + Deserialize<'a>>(serialized_env: &'a str) -> Fallible<Box<dyn Any>> {
                let env: T = serde_json::from_str(serialized_env).unwrap();
                let closure = make_closure(env);
                let closure = ConcreteFn(closure);
                Ok(Box::new(closure))
            }
            dispatch!(monomorphize, [
                (serialization.E, @numbers)
            ], (&serialization.env))
        }

        StructOperation::new(closure, serializer, deserializer)
    }

    #[cfg(test)]
    pub mod test {
        use super::super::*;
        use super::*;
        use crate::error::Fallible;

        #[test]
        fn test_mul_internal() -> Fallible<()> {
            let op = make_mul(3);
            let arg = vec![1, 2, 3];
            let res = go_internal(arg, op);
            assert_eq!(res, vec![3, 6, 9]);
            Ok(())
        }

        #[test]
        fn test_mul_external() -> Fallible<()> {
            let op = make_mul(3);
            let arg = vec![1, 2, 3];
            let res = go_external(arg, op);
            assert_eq!(res, vec![3, 6, 9]);
            Ok(())
        }

    }
}

