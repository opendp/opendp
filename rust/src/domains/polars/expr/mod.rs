use polars::lazy::dsl::Expr;

use crate::{core::Domain, error::Fallible};

use super::LazyFrameDomain;


#[derive(Clone, PartialEq)]
struct ExprDomain {
    lazyframe_domain: LazyFrameDomain
}

impl Domain for ExprDomain {
    type Carrier = (Expr, LazyFrameDomain);

    fn member(&self, _val: &Self::Carrier) -> Fallible<bool> {
        unimplemented!()
    }
}

impl std::fmt::Debug for ExprDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExprDomain").field("lazyframe_domain", &self.lazyframe_domain).finish()
    }
}
