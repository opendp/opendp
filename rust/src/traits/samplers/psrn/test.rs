use super::*;

pub fn test_progression<RV: PSRN>(sampler: &mut RV, min_refinements: usize) -> (RV::Edge, RV::Edge)
where
    RV::Edge: PartialOrd,
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
