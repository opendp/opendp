use super::*;

struct UniformRV;

impl InverseCDF for UniformRV {
    type Edge = RBig;

    fn inverse_cdf(
        &self,
        uniform: RBig,
        _refinements: usize,
        _direction: Direction,
    ) -> Fallible<Option<Self::Edge>> {
        Ok(Some(uniform))
    }
}

#[test]
fn test_value() -> Fallible<()> {
    let mut psrn = PartialSample::new(UniformRV);
    // sampled value will always be in [0, 1]
    let value: f64 = psrn.value()?;
    assert!((0f64..1f64).contains(&value));

    Ok(())
}

#[test]
fn test_greater_than() -> Fallible<()> {
    let (mut l, mut r) = (PartialSample::new(UniformRV), PartialSample::new(UniformRV));

    if l.greater_than(&mut r)? {
        assert!(l.value::<f64>()? > r.value::<f64>()?);
    } else {
        assert!(l.value::<f64>()? < r.value::<f64>()?);
    }

    Ok(())
}

#[test]
fn test_greater_than_equal() -> Fallible<()> {
    struct PointMass;

    impl InverseCDF for PointMass {
        type Edge = RBig;

        fn inverse_cdf(
            &self,
            _uniform: RBig,
            refinements: usize,
            _direction: Direction,
        ) -> Fallible<Option<Self::Edge>> {
            // Equality is already certified, so no refinement is necessary.
            assert_eq!(refinements, 0);
            Ok(Some(RBig::ZERO))
        }
    }

    let (mut l, mut r) = (PartialSample::new(PointMass), PartialSample::new(PointMass));
    assert!(!l.greater_than(&mut r)?);
    Ok(())
}
