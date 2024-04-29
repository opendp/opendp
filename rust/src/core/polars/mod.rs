use polars_plan::dsl::Expr;

use super::Function;

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
