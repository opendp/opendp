use super::*;
use crate::core::Domain;

#[test]
fn test_make_sum() -> Fallible<()> {
    macro_rules! test_sum {
        ($bounds:expr, $data:expr, $expected:expr, $metric:expr) => {{
            let input_space = (VectorDomain::new(AtomDomain::new_closed($bounds)?), $metric);
            let transformation = (input_space >> then_sum())?;
            let ret = transformation.invoke(&$data)?;
            assert_eq!(ret, $expected);
        }};
    }
    test_sum!((0, 10), vec![1, 2, 3, 4, 5], 15, SymmetricDistance);
    test_sum!((0, 10), vec![1, 2, 3, 4, 5], 15, InsertDeleteDistance);
    test_sum!((0., 10.), vec![1., 2., 3., 4., 5.], 15., SymmetricDistance);
    test_sum!(
        (0., 10.),
        vec![1., 2., 3., 4., 5.],
        15.,
        InsertDeleteDistance
    );
    Ok(())
}

// Counterexamples reported by Maegereg in https://github.com/opendp/opendp/issues/3083.
// Compare halves so that the difference itself cannot overflow.
#[test]
fn test_make_sum_float_overflow_unsized() -> Fallible<()> {
    let k = f64::MAX / 4.001;
    let x = [2., -1., -1., 2., 2., 2., 0., -1., -1., -1., -1.]
        .map(|v| v * k)
        .to_vec();
    let mut y = x.clone();
    y.remove(0);
    assert_eq!(y, x[1..]); // exactly one deletion

    let domain = VectorDomain::new(AtomDomain::new_closed((-k, 2. * k))?);
    let trans = make_sum(domain, InsertDeleteDistance)?;
    assert!(trans.input_domain.member(&x)?);
    assert!(trans.input_domain.member(&y)?);

    let (fx, fy, d_out) = (trans.invoke(&x)?, trans.invoke(&y)?, trans.map(&1)?);
    assert!(fx.is_finite() && fy.is_finite() && d_out.is_finite());
    assert!(
        (fx / 2. - fy / 2.).abs() <= d_out / 2.,
        "{fx:e} {fy:e} {d_out:e}"
    );
    Ok(())
}

macro_rules! test_make_sum_float_overflow_sized {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() -> Fallible<()> {
            let k = <$ty>::MAX / 2.001;
            let x = [-1., -1., 1., 1., 1., 1., -1., -1., -1., -1.]
                .map(|v| v * k)
                .to_vec();
            let mut y = x.clone();
            y.pop();
            y.insert(0, k);
            assert_eq!(x.len(), y.len());
            assert_eq!(&x[..x.len() - 1], &y[1..]); // one deletion and one insertion

            let domain = VectorDomain::new(AtomDomain::new_closed((-k, k))?).with_size(10);
            let trans = make_sum(domain, InsertDeleteDistance)?;
            assert!(trans.input_domain.member(&x)?);
            assert!(trans.input_domain.member(&y)?);

            let (fx, fy, d_out) = (trans.invoke(&x)?, trans.invoke(&y)?, trans.map(&2)?);
            assert!(fx.is_finite() && fy.is_finite() && d_out.is_finite());
            assert!(
                (fx / 2. - fy / 2.).abs() <= d_out / 2.,
                "{fx:e} {fy:e} {d_out:e}"
            );
            Ok(())
        }
    };
}
test_make_sum_float_overflow_sized!(test_make_sum_float_overflow_sized_f64, f64);
test_make_sum_float_overflow_sized!(test_make_sum_float_overflow_sized_f32, f32);

#[test]
fn test_make_sum_preserves_pairwise_checked_sum() -> Fallible<()> {
    let domain = VectorDomain::new(AtomDomain::new_closed((-1e16, 1e16))?).with_size(4);
    let trans = make_sum(domain, InsertDeleteDistance)?;
    let data = vec![1e16, 1., -1e16, 1.];
    assert!(trans.input_domain.member(&data)?);
    // Pairwise summation gives zero here, while sequential summation gives one.
    // The checked path can retain the tighter pairwise error bound.
    assert_eq!(trans.invoke(&data)?, 0.);
    let sequential = make_sized_bounded_float_checked_sum::<Sequential<f64>>(4, (-1e16, 1e16))?;
    assert_eq!(sequential.invoke(&data)?, 1.);
    assert!(trans.map(&2)? < sequential.map(&2)?);
    Ok(())
}

#[test]
fn test_make_sum_float_overflow_unrepresentable_relaxation() -> Fallible<()> {
    let k = f32::MAX / 4.001;
    let domain = VectorDomain::new(AtomDomain::new_closed((-k, 2. * k))?);
    // With unknown size, the default size limit makes the conservative sequential
    // error bound exceed f32::MAX. Reject instead of returning an invalid bound.
    assert!(make_sum(domain.clone(), InsertDeleteDistance).is_err());
    assert!(make_sum(domain, SymmetricDistance).is_err());
    Ok(())
}

#[test]
fn test_make_sum_float_overflow_symmetric_relaxation() -> Fallible<()> {
    // Both shuffled fallbacks must account for sequential, not pairwise, rounding.
    let k = f64::MAX / 4.001;
    let bounds = (-k, 2. * k);
    let domain = VectorDomain::new(AtomDomain::new_closed(bounds)?);
    let unsized_sum = make_sum(domain.clone(), SymmetricDistance)?;
    let sequential = make_bounded_float_ordered_sum::<Sequential<f64>>(DEFAULT_SIZE_LIMIT, bounds)?;
    assert_eq!(unsized_sum.map(&1)?, sequential.map(&1)?);

    let sized = make_sum(domain.with_size(10), SymmetricDistance)?;
    let sequential = make_sized_bounded_float_ordered_sum::<Sequential<f64>>(10, bounds)?;
    assert_eq!(sized.map(&2)?, sequential.map(&2)?);
    Ok(())
}
