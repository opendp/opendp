#[cfg(feature = "ffi")]
mod ffi;

use crate::core::{Transformation, Function, StabilityMap, Domain, Metric};
use crate::core::{SymmetricDistance, InsertDeleteDistance, IntDistance};
use crate::core::{VectorDomain, SizedDomain};
use crate::error::Fallible;
use crate::traits::samplers::Shuffle;
use crate::traits::CheckNull;
use std::cmp::Ordering;

pub trait IsMetricOrdered: Metric {
    const ORDERED: bool;
}
impl IsMetricOrdered for SymmetricDistance {
    const ORDERED: bool = false;
}
impl IsMetricOrdered for InsertDeleteDistance {
    const ORDERED: bool = true;
}

pub fn make_resize_constant<DA, MI, MO>(
    size: usize,
    atom_domain: DA,
    constant: DA::Carrier,
) -> Fallible<Transformation<VectorDomain<DA>, SizedDomain<VectorDomain<DA>>, MI, MO>>
where
    DA: 'static + Clone + Domain,
    DA::Carrier: 'static + Clone + CheckNull,
    MI: IsMetricOrdered<Distance = IntDistance>,
    MO: IsMetricOrdered<Distance = IntDistance>,
{
    if !atom_domain.member(&constant)? {
        return fallible!(MakeTransformation, "constant must be a member of DA");
    }
    if size == 0 {
        return fallible!(MakeTransformation, "row size must be greater than zero");
    }

    Ok(Transformation::new(
        VectorDomain::new(atom_domain.clone()),
        SizedDomain::new(VectorDomain::new(atom_domain), size),
        Function::new_fallible(move |arg: &Vec<DA::Carrier>| {
            Ok(match arg.len().cmp(&size) {
                Ordering::Less | Ordering::Equal => {
                    let mut data = arg
                        .iter()
                        .chain(vec![&constant; size - arg.len()])
                        .cloned()
                        .collect::<Vec<DA::Carrier>>();
                    // if output metric is ordered, then shuffle the imputed values into the data
                    if MO::ORDERED {
                        data.shuffle()?;
                    }
                    data
                }
                Ordering::Greater => {
                    let mut data = arg.clone();
                    // if input metric is not ordered, then shuffle so that the slice is a random draw from the data
                    if !MI::ORDERED {
                        data.shuffle()?;
                    }
                    arg[..size].to_vec()
                }
            })
        }),
        MI::default(),
        MO::default(),
        // Consider when a dataset has zero records and is resized to length 1.
        // The resulting dataset will be `vec![constant]`
        // Now consider a neighboring dataset that differs by one addition of `value`.
        // The resulting dataset will be `vec![value]`.
        // `vec![constant]` and `vec![value]` differ by an addition and deletion, or distance 2.
        // In the worst case, for each addition in the input, there are two changes in the output
        StabilityMap::new_from_constant(2),
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::AllDomain;

    #[test]
    fn test() -> Fallible<()> {
        let trans = make_resize_constant::<_, SymmetricDistance, SymmetricDistance>(
            3,
            AllDomain::new(),
            "x",
        )?;
        assert_eq!(trans.invoke(&vec!["A"; 2])?, vec!["A", "A", "x"]);
        assert_eq!(trans.invoke(&vec!["A"; 3])?, vec!["A"; 3]);
        assert_eq!(trans.invoke(&vec!["A"; 4])?, vec!["A", "A", "A"]);

        assert!(trans.check(&1, &2)?);
        assert!(!trans.check(&1, &1)?);
        Ok(())
    }
}
