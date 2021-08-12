use crate::core::{Transformation, Function, StabilityRelation, Domain};
use crate::error::Fallible;
use crate::dist::{SymmetricDistance, IntDistance};
use crate::dom::{VectorDomain, SizedDomain};
use std::cmp::Ordering;
use crate::traits::CheckNull;

pub fn make_resize_constant<DA>(
    atom_domain: DA,
    constant: DA::Carrier, length: usize
) -> Fallible<Transformation<VectorDomain<DA>, SizedDomain<VectorDomain<DA>>, SymmetricDistance, SymmetricDistance>>
    where DA: 'static + Clone + Domain,
          DA::Carrier: 'static + Clone + CheckNull {
    if !atom_domain.member(&constant)? { return fallible!(MakeTransformation, "constant must be a member of DA")}
    if length == 0 { return fallible!(MakeTransformation, "length must be greater than zero") }

    Ok(Transformation::new(
        VectorDomain::new(atom_domain.clone()),
        SizedDomain::new(VectorDomain::new(atom_domain), length),
        Function::new(move |arg: &Vec<DA::Carrier>| match arg.len().cmp(&length) {
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
    use crate::dom::AllDomain;

    #[test]
    fn test() -> Fallible<()> {
        let trans = make_resize_constant(AllDomain::new(),"x", 3)?;
        assert_eq!(trans.function.eval(&vec!["A"; 2])?, vec!["A", "A", "x"]);
        assert_eq!(trans.function.eval(&vec!["A"; 3])?, vec!["A"; 3]);
        assert_eq!(trans.function.eval(&vec!["A"; 4])?, vec!["A", "A", "A"]);

        assert!(trans.stability_relation.eval(&1, &2)?);
        assert!(!trans.stability_relation.eval(&1, &1)?);
        Ok(())
    }
}