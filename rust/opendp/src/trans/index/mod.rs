use crate::core::{Function, StabilityRelation, Transformation};
use crate::dist::SymmetricDistance;
use crate::dom::{AllDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::CheckNull;
use std::hash::Hash;
use std::collections::HashMap;
use std::iter::FromIterator;


pub fn make_find<TIA>(
    categories: Vec<TIA>
) -> Fallible<Transformation<VectorDomain<AllDomain<TIA>>, VectorDomain<AllDomain<usize>>, SymmetricDistance, SymmetricDistance>>
    where TIA: 'static + CheckNull + Clone + Hash + Eq {
    let categories_len = categories.len();
    let indexes = HashMap::<TIA, usize>::from_iter(categories.into_iter()
        .enumerate().map(|(i, v)| (v, i)));

    if indexes.len() != categories_len {
        return fallible!(MakeTransformation, "categories must be unique")
    }

    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new(move |arg: &Vec<TIA>| arg.iter()
            .map(|v| indexes.get(v).cloned().unwrap_or(categories_len))
            .collect()),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityRelation::new_from_constant(1)
    ))
}

pub fn make_find_bin<TIA>(
    edges: Vec<TIA>
) -> Fallible<Transformation<VectorDomain<AllDomain<TIA>>, VectorDomain<AllDomain<usize>>, SymmetricDistance, SymmetricDistance>>
    where TIA: 'static + PartialOrd + CheckNull {
    if !edges.windows(2).all(|pair| pair[0] < pair[1]) {
        return fallible!(MakeTransformation, "edges must be unique and ordered")
    }
    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new(move |arg: &Vec<TIA>| arg.iter().map(|v| edges
            .iter().enumerate()
            .find(|(_, edge)| v < edge).map(|(i, _)| i)
            .unwrap_or(edges.len()))
            .collect()),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityRelation::new_from_constant(1)
    ))
}

pub fn make_index<TOA>(
    categories: Vec<TOA>, null: TOA
) -> Fallible<Transformation<VectorDomain<AllDomain<usize>>, VectorDomain<AllDomain<TOA>>, SymmetricDistance, SymmetricDistance>>
    where TOA: 'static + CheckNull + Clone {
    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new(move |arg: &Vec<usize>| arg.iter()
            .map(|v| categories.get(*v).unwrap_or(&null).clone())
            .collect()),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityRelation::new_from_constant(1)
    ))
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find() -> Fallible<()> {
        let find = make_find(vec!["1", "3", "4"])?;
        assert_eq!(
            find.invoke(&vec!["1", "2", "3", "4", "5"])?,
            vec![0, 3, 1, 2, 3]);
        Ok(())
    }

    #[test]
    fn test_bin() -> Fallible<()> {
        let bin = make_find_bin(vec![2, 3, 5])?;
        assert_eq!(
            bin.invoke(&(1..10).collect())?,
            vec![0, 1, 2, 2, 3, 3, 3, 3, 3]);
        Ok(())
    }

    #[test]
    fn test_index() -> Fallible<()> {
        let index = make_index(vec!["A", "B", "C"], "NA")?;
        assert_eq!(
            index.invoke(&vec![0, 1, 3, 1, 5])?,
            vec!["A", "B", "NA", "B", "NA"]);
        Ok(())
    }
}