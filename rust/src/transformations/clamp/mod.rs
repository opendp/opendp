#[cfg(feature = "ffi")]
mod ffi;

use opendp_derive::bootstrap;

use crate::core::Transformation;
use crate::domains::{AllDomain, VectorDomain};
use crate::error::*;
use crate::metrics::SymmetricDistance;
use crate::traits::{TotalOrd, CheckAtom};
use crate::transformations::make_row_by_row_fallible;

#[bootstrap(features("contrib"), generics(TA(example = "$get_first(bounds)")))]
/// Make a Transformation that clamps numeric data in `Vec<TA>` to `bounds`.
///
/// If datum is less than lower, let datum be lower.
/// If datum is greater than upper, let datum be upper.
///
/// # Arguments
/// * `bounds` - Tuple of inclusive lower and upper bounds.
///
/// # Generics
/// * `TA` - Atomic Type
pub fn make_clamp<TA: 'static + Clone + TotalOrd + CheckAtom>(
    bounds: (TA, TA),
) -> Fallible<
    Transformation<
        VectorDomain<AllDomain<TA>>,
        VectorDomain<AllDomain<TA>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
> {
    make_row_by_row_fallible(
        AllDomain::default(),
        AllDomain::new_closed(bounds.clone())?,
        move |arg: &TA| arg.clone().total_clamp(bounds.0.clone(), bounds.1.clone()),
    )
}

#[cfg(test)]
mod tests {
    use crate::transformations::make_clamp;

    use super::*;

    #[test]
    fn test_make_clamp() {
        let transformation = make_clamp((0, 10)).unwrap_test();
        let arg = vec![-10, -5, 0, 5, 10, 20];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = vec![0, 0, 0, 5, 10, 10];
        assert_eq!(ret, expected);
    }
}
