use super::*;
use crate::core::Domain;

#[test]
fn test_make_bounded_float_ordered_sum() -> Fallible<()> {
    let trans = make_bounded_float_ordered_sum::<Sequential<f64>>(4, (1., 10.))?;
    let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
    assert_eq!(sum, 10.);

    let trans = make_bounded_float_ordered_sum::<Sequential<f32>>(4, (1., 10.))?;
    let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
    assert_eq!(sum, 10.);

    Ok(())
}

#[test]
fn test_make_sized_bounded_float_ordered_sum() -> Fallible<()> {
    let trans = make_sized_bounded_float_ordered_sum::<Sequential<f64>>(4, (1., 10.))?;
    let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
    assert_eq!(sum, 10.);

    let trans = make_sized_bounded_float_ordered_sum::<Sequential<f32>>(4, (1., 10.))?;
    let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
    assert_eq!(sum, 10.);

    Ok(())
}

#[test]
fn test_float_ordered_sum_empty() -> Fallible<()> {
    let empty = vec![];
    let trans = make_bounded_float_ordered_sum::<Sequential<f64>>(0, (0., 1.))?;
    assert!(trans.input_domain.member(&empty)?);
    assert_eq!(trans.invoke(&empty)?, 0.);
    assert_eq!(trans.map(&0)?, 0.);

    let empty = vec![];
    let trans = make_sized_bounded_float_ordered_sum::<Sequential<f32>>(0, (0., 1.))?;
    assert!(trans.input_domain.member(&empty)?);
    assert_eq!(trans.invoke(&empty)?, 0.);
    assert_eq!(trans.map(&0)?, 0.);
    Ok(())
}

#[test]
fn test_float_ordered_sum_truncates_to_size_limit() -> Fallible<()> {
    let k = f64::MAX / 2.001;
    let trans = make_bounded_float_ordered_sum::<Sequential<f64>>(2, (-k, k))?;
    let data = vec![k, k, -k];
    assert!(trans.input_domain.member(&data)?);
    assert_eq!(trans.invoke(&data)?, 2. * k);
    assert!(trans.map(&1)?.is_finite());
    Ok(())
}

macro_rules! test_float_ordered_sum_saturates {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() -> Fallible<()> {
            let k = <$ty>::MAX / 4.001;
            let trans = make_sized_bounded_float_ordered_sum::<Sequential<$ty>>(4, (-k, 2. * k))?;
            let data = vec![2. * k, 2. * k, 2. * k, -k];
            assert!(trans.input_domain.member(&data)?);
            assert_eq!(trans.invoke(&data)?, <$ty>::MAX - k);

            let trans = make_sized_bounded_float_ordered_sum::<Sequential<$ty>>(4, (-2. * k, k))?;
            let data = vec![-2. * k, -2. * k, -2. * k, k];
            assert!(trans.input_domain.member(&data)?);
            assert_eq!(trans.invoke(&data)?, <$ty>::MIN + k);
            Ok(())
        }
    };
}
test_float_ordered_sum_saturates!(test_float_ordered_sum_saturates_f32, f32);
test_float_ordered_sum_saturates!(test_float_ordered_sum_saturates_f64, f64);

#[test]
fn test_sequential_float_ordered_sum_stability() -> Fallible<()> {
    // Maegereg's counterexamples from #3083 remain within the advertised bound
    // when the caller explicitly requests sequential ordered summation.
    let k = f64::MAX / 4.001;
    let x = [2., -1., -1., 2., 2., 2., 0., -1., -1., -1., -1.]
        .map(|v| v * k)
        .to_vec();
    let mut y = x.clone();
    y.remove(0);
    assert_eq!(y, x[1..]);
    let trans = make_bounded_float_ordered_sum::<Sequential<f64>>(1 << 20, (-k, 2. * k))?;
    assert!(trans.input_domain.member(&x)?);
    assert!(trans.input_domain.member(&y)?);
    let (fx, fy, d_out) = (trans.invoke(&x)?, trans.invoke(&y)?, trans.map(&1)?);
    assert!(fx.is_finite() && fy.is_finite() && d_out.is_finite());
    assert!((fx / 2. - fy / 2.).abs() <= d_out / 2.);

    let k = f64::MAX / 2.001;
    let x = [-1., -1., 1., 1., 1., 1., -1., -1., -1., -1.]
        .map(|v| v * k)
        .to_vec();
    let mut y = x.clone();
    y.pop();
    y.insert(0, k);
    assert_eq!(x.len(), y.len());
    assert_eq!(&x[..x.len() - 1], &y[1..]);
    let trans = make_sized_bounded_float_ordered_sum::<Sequential<f64>>(10, (-k, k))?;
    assert!(trans.input_domain.member(&x)?);
    assert!(trans.input_domain.member(&y)?);
    let (fx, fy, d_out) = (trans.invoke(&x)?, trans.invoke(&y)?, trans.map(&2)?);
    assert!(fx.is_finite() && fy.is_finite() && d_out.is_finite());
    assert!((fx / 2. - fy / 2.).abs() <= d_out / 2.);
    Ok(())
}
