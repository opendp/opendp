use super::*;

pub fn test_progression<D: InverseCDF>(
    sampler: &mut PSRN<D>,
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

        if sampler.refinements() >= min_refinements {
            return (l, r);
        }
    }
}
