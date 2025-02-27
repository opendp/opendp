use crate::metrics::{AbsoluteDistance, L2Distance};

use super::*;
use num::{One, Zero};

#[test]
fn test_all() -> Fallible<()> {
    macro_rules! test_gaussian_with_ty {
        ($($ty:ty),+) => {$(
            let meas = make_gaussian::<_, ZeroConcentratedDivergence, _>(AtomDomain::<$ty>::default(), AbsoluteDistance::<$ty>::default(), 1., None)?;
            meas.invoke(&<$ty>::zero())?;
            meas.map(&<$ty>::one())?;

            let meas = make_gaussian::<_, ZeroConcentratedDivergence, _>(VectorDomain::new(AtomDomain::<$ty>::default()), L2Distance::<$ty>::default(), 1., None)?;
            meas.invoke(&vec![<$ty>::zero()])?;
            meas.map(&<$ty>::one())?;
        )+}
    }
    test_gaussian_with_ty!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, f32, f64);
    Ok(())
}

#[test]
fn test_other_qi() -> Fallible<()> {
    macro_rules! test_gaussian_with_ty {
        ($($ty:ty),+) => {$(
            let meas = make_gaussian::<_, ZeroConcentratedDivergence, _>(AtomDomain::<$ty>::default(), AbsoluteDistance::<f64>::default(), 1., None)?;
            meas.invoke(&<$ty>::zero())?;
            meas.map(&1.)?;

            let meas = make_gaussian::<_, ZeroConcentratedDivergence, _>(VectorDomain::new(AtomDomain::<$ty>::default()), L2Distance::<f64>::default(), 1., None)?;
            meas.invoke(&vec![<$ty>::zero()])?;
            meas.map(&1.)?;
        )+}
    }
    test_gaussian_with_ty!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128);
    Ok(())
}
