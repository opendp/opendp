use super::*;

#[test]
fn test_make_bounded_float_ordered_sum() -> Fallible<()> {
    let trans = make_bounded_float_ordered_sum::<Sequential<f64>>(4, (1., 10.))?;
    let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
    assert_eq!(sum, 10.);

    let trans = make_bounded_float_ordered_sum::<Pairwise<f32>>(4, (1., 10.))?;
    let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
    assert_eq!(sum, 10.);

    Ok(())
}

#[test]
fn test_make_sized_bounded_float_ordered_sum() -> Fallible<()> {
    let trans = make_sized_bounded_float_ordered_sum::<Sequential<f64>>(4, (1., 10.))?;
    let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
    assert_eq!(sum, 10.);

    let trans = make_sized_bounded_float_ordered_sum::<Pairwise<f32>>(4, (1., 10.))?;
    let sum = trans.invoke(&vec![1., 2., 3., 4.])?;
    assert_eq!(sum, 10.);

    Ok(())
}
