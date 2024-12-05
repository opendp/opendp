use super::*;

#[test]
fn test_make_sum() -> Fallible<()> {
    macro_rules! test_sum {
        ($bounds:expr, $data:expr, $expected:expr, $metric:expr) => {{
            let input_space = (
                VectorDomain::new(AtomDomain::new_closed($bounds)?),
                SymmetricDistance::default(),
            );
            let transformation = (input_space >> then_sum())?;
            let ret = transformation.invoke(&$data)?;
            assert_eq!(ret, $expected);
        }};
    }
    test_sum!(
        (0, 10),
        vec![1, 2, 3, 4, 5],
        15,
        SymmetricDistance::default()
    );
    test_sum!(
        (0, 10),
        vec![1, 2, 3, 4, 5],
        15,
        InsertDeleteDistance::default()
    );
    test_sum!(
        (0., 10.),
        vec![1., 2., 3., 4., 5.],
        15.,
        SymmetricDistance::default()
    );
    test_sum!(
        (0., 10.),
        vec![1., 2., 3., 4., 5.],
        15.,
        InsertDeleteDistance::default()
    );
    Ok(())
}
