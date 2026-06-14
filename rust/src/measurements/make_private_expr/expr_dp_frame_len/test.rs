use polars::prelude::{col, lit};

use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    metrics::{InsertDeleteDistance, L0PInfDistance, L2Distance},
    transformations::StableExpr,
};


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dp_len() {
        assert!(false);
    }
}