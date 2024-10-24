use super::*;
use num::{One, Zero};

#[test]
fn test_all() -> Fallible<()> {
    macro_rules! test_laplace_with_ty {
        ($($ty:ty),+) => {$(
            let meas = make_laplace(AtomDomain::<$ty>::default(), Default::default(), 1., None)?;
            meas.invoke(&<$ty>::zero())?;
            meas.map(&<$ty>::one())?;

            let meas = make_laplace(VectorDomain::new(AtomDomain::<$ty>::default()), Default::default(), 1., None)?;
            meas.invoke(&vec![<$ty>::zero()])?;
            meas.map(&<$ty>::one())?;
        )+}
    }
    test_laplace_with_ty!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, f32, f64);
    Ok(())
}
