#[cfg(feature="ffi")]
mod ffi;

use std::collections::Bound;

use crate::core::Transformation;
use crate::dist::SymmetricDistance;
use crate::dom::{AllDomain, BoundedDomain, VectorDomain};
use crate::error::*;
use crate::traits::{CheckNull, TotalOrd};
use crate::trans::{make_row_by_row, make_row_by_row_fallible};

pub fn make_clamp<T: 'static + Clone + TotalOrd + CheckNull>(
    bounds: (T, T)
) -> Fallible<Transformation<VectorDomain<AllDomain<T>>, VectorDomain<BoundedDomain<T>>, SymmetricDistance, SymmetricDistance>> {
    make_row_by_row_fallible(
        AllDomain::new(),
        BoundedDomain::new_closed(bounds.clone())?,
        move |arg: &T| arg.clone().total_clamp(bounds.0.clone(), bounds.1.clone()))
}

pub fn make_unclamp<T: 'static + Clone + TotalOrd + CheckNull>(
    bounds: (Bound<T>, Bound<T>)
) -> Fallible<Transformation<VectorDomain<BoundedDomain<T>>, VectorDomain<AllDomain<T>>, SymmetricDistance, SymmetricDistance>> {
    make_row_by_row(
        BoundedDomain::new(bounds)?,
        AllDomain::new(),
        |arg| arg.clone())
}


#[cfg(test)]
mod tests {
    use crate::trans::{make_clamp, make_unclamp};

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
