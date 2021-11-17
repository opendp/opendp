#[cfg(feature="ffi")]
mod ffi;

use crate::core::{Transformation, Function, StabilityRelation, Domain};
use crate::error::Fallible;
use crate::dist::{SymmetricDistance};
use crate::dom::{VectorDomain, SizedDomain};
use std::cmp::Ordering;
use crate::traits::CheckNull;

pub fn make_resize_constant<DA>(
    size: usize, atom_domain: DA,
    constant: DA::Carrier
) -> Fallible<Transformation<VectorDomain<DA>, SizedDomain<VectorDomain<DA>>, SymmetricDistance, SymmetricDistance>>
    where DA: 'static + Clone + Domain,
          DA::Carrier: 'static + Clone + CheckNull {
    if !atom_domain.member(&constant)? { return fallible!(MakeTransformation, "constant must be a member of DA")}
    if size == 0 { return fallible!(MakeTransformation, "row size must be greater than zero") }

    Ok(Transformation::new(
        VectorDomain::new(atom_domain.clone()),
        SizedDomain::new(VectorDomain::new(atom_domain), size),
        Function::new(move |arg: &Vec<DA::Carrier>| match arg.len().cmp(&size) {
            Ordering::Less => arg.iter().chain(vec![&constant; size - arg.len()]).cloned().collect(),
            Ordering::Equal => arg.clone(),
            Ordering::Greater => arg[..size].to_vec()
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityRelation::new_from_constant(2)
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dom::AllDomain;

    #[test]
    fn test() -> Fallible<()> {
        let trans = make_resize_constant(3, AllDomain::new(), "x")?;
        assert_eq!(trans.invoke(&vec!["A"; 2])?, vec!["A", "A", "x"]);
        assert_eq!(trans.invoke(&vec!["A"; 3])?, vec!["A"; 3]);
        assert_eq!(trans.invoke(&vec!["A"; 4])?, vec!["A", "A", "A"]);

        assert!(trans.check(&1, &2)?);
        assert!(!trans.check(&1, &1)?);
        Ok(())
    }
}