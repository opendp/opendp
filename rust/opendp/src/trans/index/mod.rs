use crate::core::{Function, StabilityRelation, Transformation};
use crate::dist::SymmetricDistance;
use crate::dom::{AllDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::CheckNull;
use std::hash::Hash;
use std::collections::HashMap;
use std::iter::FromIterator;


pub fn make_find<TI>(
    categories: Vec<TI>
) -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, VectorDomain<AllDomain<usize>>, SymmetricDistance, SymmetricDistance>>
    where TI: 'static + CheckNull + Clone + Hash + Eq {
    let categories_len = categories.len();
    let indexes = HashMap::<TI, usize>::from_iter(categories.into_iter()
        .enumerate().map(|(i, v)| (v, i)));

    if indexes.len() != categories_len {
        return fallible!(MakeTransformation, "categories must be unique")
    }

    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new(move |arg: &Vec<TI>| arg.iter()
            .map(|v| indexes.get(v).cloned().unwrap_or(categories_len))
            .collect()),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityRelation::new_from_constant(1)
    ))
}

pub fn make_bin_edges<TI, TO>(
    edges: Vec<TI>
) -> Fallible<Transformation<VectorDomain<AllDomain<TI>>, VectorDomain<AllDomain<usize>>, SymmetricDistance, SymmetricDistance>>
    where TI: 'static + PartialOrd + CheckNull {
    if !edges.windows(2).all(|pair| pair[0] < pair[1]) {
        return fallible!(MakeTransformation, "edges must be unique and ordered")
    }
    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new(move |arg: &Vec<TI>| arg.iter().map(|v| edges
            .iter().enumerate()
            .find(|(_, edge)| v < edge).map(|(i, _)| i)
            .unwrap_or(edges.len()))
            .collect()),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityRelation::new_from_constant(1)
    ))
}

pub fn make_index<TO>(
    categories: Vec<TO>, null: TO
) -> Fallible<Transformation<VectorDomain<AllDomain<usize>>, VectorDomain<AllDomain<TO>>, SymmetricDistance, SymmetricDistance>>
    where TO: 'static + CheckNull + Clone {
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
