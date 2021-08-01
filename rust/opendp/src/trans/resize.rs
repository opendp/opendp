use crate::core::{Transformation, Function, StabilityRelation};
use crate::error::Fallible;
use crate::dist::{SymmetricDistance, IntDistance};
use crate::dom::{VectorDomain, AllDomain, SizedDomain};
use std::cmp::Ordering;
use crate::traits::CheckNull;

pub fn make_resize_constant<T: 'static + Clone + CheckNull>(
    constant: T, length: usize
) -> Fallible<Transformation<VectorDomain<AllDomain<T>>, SizedDomain<VectorDomain<AllDomain<T>>>, SymmetricDistance, SymmetricDistance>> {
    if length == 0 { return fallible!(MakeTransformation, "length must be greater than zero") }
    if constant.is_null() { return fallible!(MakeTransformation, "constant may not be null") }

    Ok(Transformation::new(
        VectorDomain::new_all(),
        SizedDomain::new(VectorDomain::new_all(), length),
        Function::new(move |arg: &Vec<T>| match arg.len().cmp(&length) {
            Ordering::Less => arg.iter().chain(vec![&constant; length - arg.len()]).cloned().collect(),
            Ordering::Equal => arg.clone(),
            Ordering::Greater => arg[..length].to_vec()
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityRelation::new(move |d_in: &IntDistance, d_out: &IntDistance| *d_out >= d_in + d_in % 2)
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() -> Fallible<()> {
        let trans = make_resize_constant("x", 3)?;
        assert_eq!(trans.function.eval(&vec!["A"; 2])?, vec!["A", "A", "x"]);
        assert_eq!(trans.function.eval(&vec!["A"; 3])?, vec!["A"; 3]);
        assert_eq!(trans.function.eval(&vec!["A"; 4])?, vec!["A", "A", "A"]);

        assert!(trans.stability_relation.eval(&1, &2)?);
        assert!(!trans.stability_relation.eval(&1, &1)?);
        Ok(())
    }
}