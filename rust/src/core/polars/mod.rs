use polars_plan::dsl::Expr;

use super::Function;
use crate::error::Fallible;
use crate::traits::InfCast;

pub trait ExprFunction {
    fn new_expr(function: impl Fn(Expr) -> Expr + 'static + Send + Sync) -> Self;
}

impl<F: Clone> ExprFunction for Function<(F, Expr), (F, Expr)> {
    fn new_expr(function: impl Fn(Expr) -> Expr + 'static + Send + Sync) -> Self {
        Self::new(move |arg: &(F, Expr)| (arg.0.clone(), function(arg.1.clone())))
    }
}
impl<F: Clone> ExprFunction for Function<(F, Expr), Expr> {
    fn new_expr(function: impl Fn(Expr) -> Expr + 'static + Send + Sync) -> Self {
        Self::new(move |arg: &(F, Expr)| function(arg.1.clone()))
    }
}

impl ExprFunction for Function<Expr, Expr> {
    fn new_expr(function: impl Fn(Expr) -> Expr + 'static + Send + Sync) -> Self {
        Self::new(move |arg: &Expr| function(arg.clone()))
    }
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "polars", derive(serde::Serialize, serde::Deserialize))]
pub enum Scalar {
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

impl Scalar {
    pub fn f64(&self) -> Fallible<f64> {
        match self {
            Scalar::I32(v) => f64::inf_cast(*v),
            Scalar::I64(v) => f64::inf_cast(*v),
            Scalar::U32(v) => f64::inf_cast(*v),
            Scalar::U64(v) => f64::inf_cast(*v),
            Scalar::F32(v) => f64::inf_cast(*v),
            Scalar::F64(v) => f64::inf_cast(*v),
        }
    }
}

impl PartialOrd for Scalar {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Scalar::I32(l), Scalar::I32(r)) => l.partial_cmp(r),
            (Scalar::I64(l), Scalar::I64(r)) => l.partial_cmp(r),
            (Scalar::U32(l), Scalar::U32(r)) => l.partial_cmp(r),
            (Scalar::U64(l), Scalar::U64(r)) => l.partial_cmp(r),
            (Scalar::F32(l), Scalar::F32(r)) => l.partial_cmp(r),
            (Scalar::F64(l), Scalar::F64(r)) => l.partial_cmp(r),
            _ => None,
        }
    }
}

macro_rules! impl_from_for_scalar {
    ($t:ty, $s:ident) => {
        impl From<$t> for Scalar {
            fn from(value: $t) -> Self {
                Scalar::$s(value)
            }
        }
    };
}
impl_from_for_scalar!(i32, I32);
impl_from_for_scalar!(i64, I64);
impl_from_for_scalar!(u32, U32);
impl_from_for_scalar!(u64, U64);
impl_from_for_scalar!(f32, F32);
impl_from_for_scalar!(f64, F64);
