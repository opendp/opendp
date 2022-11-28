use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::AllDomain;
use crate::error::Fallible;
use crate::metrics::{AbsoluteDistance, SymmetricDistance};
use crate::traits::CheckNull;

pub struct PCollection {}
impl CheckNull for PCollection {
    fn is_null(&self) -> bool {
        false
    }
}

pub fn make_beam_sum() -> Fallible<
    Transformation<
        AllDomain<PCollection>,
        AllDomain<f64>,
        SymmetricDistance,
        AbsoluteDistance<f64>,
    >,
> {
    Ok(Transformation::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new(|arg| 99.9),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new(|_d_in| 1.0),
    ))
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_sum() -> Fallible<()> {
        let sum = make_beam_sum()?;
        let arg = PCollection {};
        let res = sum.invoke(&arg)?;
        assert_eq!(res, 99.9);
        Ok(())
    }
}
