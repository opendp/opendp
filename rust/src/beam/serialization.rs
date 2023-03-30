use std::any::{Any, type_name};
use crate::error::Fallible;
use crate::ffi::util::Type;


pub trait Operation<A, R> {
    fn into_closure(self) -> Box<dyn Fn(&A) -> R>;
    fn serialize(&self) -> Serialization;
    fn deserializer_op(&self) -> fn(&Serialization) -> Fallible<AnyOperation>;
    // fn deserializer_fn(&self) -> fn(&Serialization) -> Fallible<Box<dyn Any>>;
}

pub struct Serialization {
    E: Type,
    #[allow(dead_code)]
    A: Type,
    #[allow(dead_code)]
    R: Type,
    env: String,
}
impl Serialization {
    pub fn new(E: Type, A: Type, R: Type, env: String) -> Self {
        Self { E, A, R, env }
    }
}

struct FnHolder<A, R>(Box<dyn Fn(&A) -> R>);

pub struct AnyOperation {
    holder: Box<dyn Any>,
}
impl AnyOperation {
    pub fn new<A: 'static, R: 'static>(operation: impl Operation<A, R>) -> Self {
        let holder = FnHolder(operation.into_closure());
        let holder = Box::new(holder) as Box<dyn Any>;
        Self { holder }
    }
    pub fn into_closure<A: 'static, R: 'static>(self) -> Fallible<Box<dyn Fn(&A) -> R>> {
        let holder = self.holder.downcast::<FnHolder<A, R>>();
        let holder = holder.map_err(|_| err!(FailedCast, "Failed downcast of AnyOperation to FnHolder<{}, {}>", type_name::<A>(), type_name::<R>()));
        let holder = holder.unwrap();
        Ok((*holder).0)
    }
}

pub struct StructOperation<A, R> {
    closure: Box<dyn Fn(&A) -> R>,
    serializer: Box<dyn Fn(&Self) -> Serialization>,
    deserializer: fn(&Serialization) -> Fallible<AnyOperation>,
}
impl<A: 'static, R: 'static> StructOperation<A, R> {
    pub fn new(closure: Box<dyn Fn(&A) -> R>, serializer: Box<dyn Fn(&Self) -> Serialization>, deserializer: fn(&Serialization) -> Fallible<AnyOperation>) -> Self {
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
    fn deserializer_op(&self) -> fn(&Serialization) -> Fallible<AnyOperation> {
        unimplemented!()
    }
}

pub fn go_internal<A, R, O: Operation<A, R>>(arg: Vec<A>, op: O) -> Vec<R> {
    let f = op.into_closure();
    arg.iter().map(f).collect()
}

pub fn go_external<A: 'static, R: 'static, O: Operation<A, R>>(arg: Vec<A>, op: O) -> Vec<R> {
    let do_op = true;
    if do_op {
        go_external_op(arg, op)
    } else {
        go_external_fn(arg, op)
    }
}

pub fn go_external_op<A: 'static, R: 'static, O: Operation<A, R>>(arg: Vec<A>, op: O) -> Vec<R> {
    let deserializer = op.deserializer_op();
    let serialized = op.serialize();
    let op = (deserializer)(&serialized).unwrap();
    let f = op.into_closure().unwrap();
    arg.iter().map(f).collect()
}

pub fn go_external_fn<A: 'static, R: 'static, O: Operation<A, R>>(arg: Vec<A>, op: O) -> Vec<R> {
    // let deserializer = op.deserializer_fn();
    // let serialized = op.serialize();
    // let op = (deserializer)(&serialized).unwrap();
    // let f = op.downcast::<FnHolder<A, R>>().unwrap();
    // let f = f.0;
    // arg.iter().map(f).collect()
    todo!()
}

pub mod m1 {
    use std::any::Any;
    use std::marker::PhantomData;
    use serde::{Deserialize, Serialize};
    use crate::beam::serialization::{AnyOperation, FnHolder, Operation, Serialization};
    use crate::error::Fallible;
    use crate::ffi::util::Type;
    use crate::traits::Number;

    pub struct EnvOperation<E, A, R> {
        _a: PhantomData<A>,
        _r: PhantomData<R>,
        env: E,
        f: fn(env: &E, arg: &A) -> R,
    }
    impl<E: 'static, A: 'static, R: 'static> Operation<A, R> for EnvOperation<E, A, R> {
        fn into_closure(self) -> Box<dyn Fn(&A) -> R> {
            let env = self.env;
            let f = self.f;
            let clos = move |arg: &A| -> R {
                f(&env, arg)
            };
            Box::new(clos)
        }
        fn serialize(&self) -> Serialization {
            todo!()
        }
        fn deserializer_op(&self) -> fn(&Serialization) -> Fallible<AnyOperation> {
            todo!()
        }
        // fn deserializer_fn(&self) -> fn(&Serialization) -> Fallible<Box<dyn Any>> {
        //     todo!()
        // }
    }

    pub struct MulOperation<T: Number> {
        env: T
    }
    impl<T: Number + Serialize> Operation<T, T> for MulOperation<T> {
        fn into_closure(self) -> Box<dyn Fn(&T) -> T> {
            Box::new(move |arg| *arg * self.env)
        }
        fn serialize(&self) -> Serialization {
            Serialization::new(Type::of::<T>(), Type::of::<T>(), Type::of::<T>(), serde_json::to_string(&self.env).unwrap())
        }
        fn deserializer_op(&self) -> fn(&Serialization) -> Fallible<AnyOperation> {
            fn deserializer(serialization: &Serialization) -> Fallible<AnyOperation> {
                fn monomorphize<'a, T: Number + Serialize + Deserialize<'a>>(serialized_env: &'a str) -> Fallible<AnyOperation> {
                    let env: T = serde_json::from_str(serialized_env).unwrap();
                    let op = MulOperation { env };
                    Ok(AnyOperation::new(op))
                }
                dispatch!(monomorphize, [
                    (serialization.E, @numbers)
                ], (&serialization.env))
            }
            deserializer
        }
        // fn deserializer_fn(&self) -> fn(&Serialization) -> Fallible<Box<dyn Any>> {
        //     fn deserializer(serialization: &Serialization) -> Fallible<Box<dyn Any>> {
        //         fn monomorphize<'a, T: Number + Serialize + Deserialize<'a>>(serialized_env: &'a str) -> Fallible<Box<dyn Any>> {
        //             let env: T = serde_json::from_str(serialized_env).unwrap();
        //             let op = MulOperation { env };
        //             let f = op.into_closure();
        //             Ok(Box::new(FnHolder(f)))
        //         }
        //         dispatch!(monomorphize, [
        //             (serialization.E, @numbers)
        //         ], (&serialization.env))
        //     }
        //     deserializer
        // }
    }

    pub fn make_mul<T: Number>(x: T) -> MulOperation<T> {
        MulOperation { env: x }
    }

    // pub fn make_mul<T: Number>(x: T) -> StructOperation<T, T> {
        // fn make_closure<T: Number>(env: T) -> Box<dyn Fn(&T) -> T> {
        //     Box::new(move |x| *x * env)
        // }
        // let closure = make_closure::<T>(x);
        // let serializer = Box::new(|s: &StructOperation<T, T>| {
        //     Serialization::new(Type::of::<T>(), Type::of::<T>(), Type::of::<T>(), serde_json::to_string(&s.env).unwrap())
        // });
        // fn deserializer(serialization: &Serialization) -> Fallible<Box<dyn Any>> {
        //     fn monomorphize<'a, T: Number + Deserialize<'a>>(serialized_env: &'a str) -> Fallible<Box<dyn Any>> {
        //         let env: T = serde_json::from_str(serialized_env).unwrap();
        //         let closure = make_closure(env);
        //         let closure = ConcreteFn(closure);
        //         Ok(Box::new(closure))
        //     }
        //     dispatch!(monomorphize, [
        //         (serialization.E, @numbers)
        //     ], (&serialization.env))
        // }
        //
        // StructOperation::new(closure, serializer, deserializer)
    // }

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

