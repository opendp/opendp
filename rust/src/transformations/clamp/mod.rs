#[cfg(feature="ffi")]
mod ffi;

use std::collections::Bound;

use opendp_derive::bootstrap;

use crate::core::Transformation;
use crate::metrics::SymmetricDistance;
use crate::domains::{AllDomain, BoundedDomain, VectorDomain};
use crate::error::*;
use crate::traits::{CheckNull, TotalOrd};
use crate::transformations::{make_row_by_row, make_row_by_row_fallible};

#[bootstrap(
    features("contrib"),
    generics(TA(example(get_first("bounds"))))
)]
/// Make a Transformation that clamps numeric data in Vec<`T`> to `bounds`.
/// If datum is less than lower, let datum be lower. 
/// If datum is greater than upper, let datum be upper.
/// 
/// # Arguments
/// * `bounds` - Tuple of inclusive lower and upper bounds.
/// 
/// # Generics
/// * `TA` - Atomic Type
pub fn make_clamp<TA: 'static + Clone + TotalOrd + CheckNull>(
    bounds: (TA, TA)
) -> Fallible<Transformation<VectorDomain<AllDomain<TA>>, VectorDomain<BoundedDomain<TA>>, SymmetricDistance, SymmetricDistance>> {
    make_row_by_row_fallible(
        AllDomain::new(),
        BoundedDomain::new_closed(bounds.clone())?,
        move |arg: &TA| arg.clone().total_clamp(bounds.0.clone(), bounds.1.clone()))
}

#[bootstrap(
    features("contrib"),
    arguments(bounds(rust_type(id="(TA, TA)"))),
    generics(TA(example(get_first("bounds")))),
)]
/// Make a Transformation that unclamps numeric data in Vec<`T`>.
/// Used to convert a `VectorDomain<BoundedDomain<T>>` to a `VectorDomain<AllDomain<T>>`.
/// 
/// # Arguments
/// * `bounds` - Tuple of lower and upper bounds.
/// 
/// # Generics
/// * `TA` - Atomic Type
pub fn make_unclamp<TA: 'static + Clone + TotalOrd + CheckNull>(
    bounds: (Bound<TA>, Bound<TA>)
) -> Fallible<Transformation<VectorDomain<BoundedDomain<TA>>, VectorDomain<AllDomain<TA>>, SymmetricDistance, SymmetricDistance>> {
    make_row_by_row(
        BoundedDomain::new(bounds)?,
        AllDomain::new(),
        |arg| arg.clone())
}


#[cfg(test)]
mod tests {
    use crate::transformations::{make_clamp, make_unclamp};

    use super::*;

    #[test]
    fn test_make_clamp() {
        let transformation = make_clamp((0, 10)).unwrap_test();
        let arg = vec![-10, -5, 0, 5, 10, 20];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = vec![0, 0, 0, 5, 10, 10];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_unclamp() -> Fallible<()> {
        let clamp = make_clamp((2, 3))?;
        let unclamp = make_unclamp((Bound::Included(2), Bound::Included(3)))?;
        let chained = (clamp >> unclamp)?;
        chained.invoke(&vec![1, 2, 3])?;
        assert!(chained.check(&1, &1)?);
        Ok(())
    }
}
