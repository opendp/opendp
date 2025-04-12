use super::*;

pub fn assert_ordered_progression<D: InverseCDF>(
    sampler: &mut PartialSample<D>,
    min_refinements: usize,
) -> (D::Edge, D::Edge)
where
    D::Edge: PartialOrd,
{
    loop {
        sampler.refine().unwrap();
        let Some((l, r)) = sampler.lower().zip(sampler.upper()) else {
            continue;
        };
        assert!(l <= r);

        if sampler.refinements >= min_refinements {
            return (l, r);
        }
    }
}

struct UniformRV;

impl InverseCDF for UniformRV {
    type Edge = RBig;

    fn inverse_cdf<R: ODPRound>(&self, uniform: RBig, _refinements: usize) -> Option<Self::Edge> {
        Some(uniform)
    }
}

#[test]
fn test_value() -> Fallible<()> {
    let mut psrn = PartialSample::new(UniformRV);
    // sampled value will always be in [0, 1]
    assert!((0f64..1f64).contains(&psrn.value()?));

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
