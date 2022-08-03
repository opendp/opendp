#[cfg(feature = "ffi")]
mod ffi;

use ndarray::Array2;
use num::One;
use opendp_derive::bootstrap;


use crate::metrics::{SymmetricDistance, IntDistance};
use crate::domains::{AllDomain, VectorDomain, Array2Domain};
use crate::core::{Domain, Function, Metric, StabilityMap, Transformation};
use crate::error::*;
use crate::traits::{CheckNull, DistanceConstant};
use std::convert::TryFrom;

pub trait DatasetDomain: Domain {
    type RowDomain: Domain;
    type Row;
    fn new(row_domain: Self::RowDomain) -> Self;
}

impl<D: Domain> DatasetDomain for VectorDomain<D> {
    type RowDomain = D;
    type Row = D::Carrier;
    fn new(row_domain: Self::RowDomain) -> Self {
        VectorDomain::new(row_domain)
    }
}

impl<D: Domain> DatasetDomain for Array2Domain<D> {
    type RowDomain = D;
    type Row = Vec<D::Carrier>;
    fn new(row_domain: Self::RowDomain) -> Self {
        Array2Domain::new(row_domain)
    }
}

pub trait RowByRowDomain<DO: DatasetDomain>: DatasetDomain {
    fn apply_rows(
        value: &Self::Carrier,
        row_function: &impl Fn(&Self::Row) -> Fallible<DO::Row>,
    ) -> Fallible<DO::Carrier>;
}

impl<DIA: Domain, DOA: Domain> RowByRowDomain<VectorDomain<DOA>> for VectorDomain<DIA> {
    fn apply_rows(
        value: &Self::Carrier,
        row_function: &impl Fn(&Self::Row) -> Fallible<DOA::Carrier>,
    ) -> Fallible<Vec<DOA::Carrier>> {
        value.iter().map(row_function).collect()
    }
}

impl<DIA: Domain, DOA: Domain> RowByRowDomain<Array2Domain<DOA>> for Array2Domain<DIA>
where
    DIA::Carrier: Clone,
{
    fn apply_rows(
        value: &Self::Carrier,
        row_function: &impl Fn(&Self::Row) -> Fallible<Vec<DOA::Carrier>>,
    ) -> Fallible<Array2<DOA::Carrier>> {
        let shape =
            <[usize; 2]>::try_from(value.shape()).unwrap_assert("input is always of shape 2");

        let data = (value.rows())
            .into_iter()
            .map(|row| row_function(&row.to_vec()))
            .try_fold(vec![], |mut acc, v| {
                acc.extend(v?);
                Fallible::Ok(acc)
            })?;

        Array2::from_shape_vec(shape, data)
            .map_err(|_| err!(FailedFunction, "func must preserve the same number of rows"))
    }
}


impl<DIA: Domain, DOA: Domain> RowByRowDomain<VectorDomain<DOA>> for Array2Domain<DIA>
where
    DIA::Carrier: Clone,
{
    fn apply_rows(
        value: &Self::Carrier,
        row_function: &impl Fn(&Self::Row) -> Fallible<DOA::Carrier>,
    ) -> Fallible<Vec<DOA::Carrier>> {
        (value.rows())
            .into_iter()
            .map(|row| row_function(&row.to_vec()))
            .collect()
    }
}

/// Constructs a [`Transformation`] representing an arbitrary row-by-row transformation.
pub(crate) fn make_row_by_row<DI, DO, M>(
    input_row_domain: DI::RowDomain,
    output_row_domain: DO::RowDomain,
    row_function: impl 'static + Fn(&DI::Row) -> DO::Row,
) -> Fallible<Transformation<DI, DO, M, M>>
where
    DI: RowByRowDomain<DO>,
    DO: DatasetDomain,
    M: Metric<Distance=IntDistance>
{
    let row_function = move |arg: &DI::Row| Ok(row_function(arg));
    Ok(Transformation::new(
        DI::new(input_row_domain),
        DO::new(output_row_domain),
        Function::new_fallible(move |arg: &DI::Carrier| DI::apply_rows(arg, &row_function)),
        M::default(),
        M::default(),
        StabilityMap::new_from_constant(1),
    ))
}

/// Constructs a [`Transformation`] representing an arbitrary row-by-row transformation.
pub(crate) fn make_row_by_row_fallible<DI, DO, M>(
    input_row_domain: DI::RowDomain,
    output_row_domain: DO::RowDomain,
    row_function: impl 'static + Fn(&DI::Row) -> Fallible<DO::Row>,
) -> Fallible<Transformation<DI, DO, M, M>>
where
    DI: RowByRowDomain<DO>,
    DO: DatasetDomain,
    M: Metric<Distance=IntDistance> {
    Ok(Transformation::new(
        DI::new(input_row_domain),
        DO::new(output_row_domain),
        Function::new_fallible(move |arg: &DI::Carrier| DI::apply_rows(arg, &row_function)),
        M::default(),
        M::default(),
        StabilityMap::new_from_constant(1),
    ))
}

/// Constructs a [`Transformation`] representing the identity function.
pub fn make_identity<D, M>(domain: D, metric: M) -> Fallible<Transformation<D, D, M, M>> where
    D: Domain,
    D::Carrier: Clone,
    M: Metric,
    M::Distance: DistanceConstant<M::Distance> + One + Clone,
{
    Ok(Transformation::new(
        domain.clone(),
        domain,
        Function::new(|arg: &D::Carrier| arg.clone()),
        metric.clone(),
        metric,
        StabilityMap::new_from_constant(M::Distance::one()),
    ))
}

#[bootstrap(features("contrib"))]
/// Make a Transformation that checks if each element is equal to `value`.
/// 
/// # Arguments
/// * `value` - value to check against
/// 
/// # Generics
/// * `TIA` - Atomic Input Type. Type of elements in the input vector
pub fn make_is_equal<TIA>(
    value: TIA,
) -> Fallible<
    Transformation<
        VectorDomain<AllDomain<TIA>>,
        VectorDomain<AllDomain<bool>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
>
where
    TIA: 'static + PartialEq + CheckNull,
{
    make_row_by_row(AllDomain::new(), AllDomain::new(), move |v| v == &value)
}

#[bootstrap(features("contrib"))]
/// Make a Transformation that checks if each element in a vector is null.
/// 
/// # Generics
/// * `DIA` - Atomic Input Domain. Can be any domain for which the carrier type has a notion of nullity.
pub fn make_is_null<DIA>() -> Fallible<
    Transformation<
        VectorDomain<DIA>,
        VectorDomain<AllDomain<bool>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
>
where
    DIA: Domain + Default,
    DIA::Carrier: 'static + CheckNull,
{
    make_row_by_row(
        DIA::default(),
        AllDomain::default(),
        |v: &DIA::Carrier| v.is_null()
    )
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::metrics::ChangeOneDistance;
    use crate::domains::{AllDomain, InherentNullDomain};


    #[test]
    fn test_identity() {
        let identity = make_identity(AllDomain::new(), ChangeOneDistance).unwrap_test();
        let arg = 99;
        let ret = identity.invoke(&arg).unwrap_test();
        assert_eq!(ret, 99);
    }

    #[test]
    fn test_is_equal() -> Fallible<()> {
        let is_equal = make_is_equal("alpha".to_string())?;
        let arg = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let ret = is_equal.invoke(&arg)?;
        assert_eq!(ret, vec![true, false, false]);
        assert!(is_equal.check(&1, &1)?);
        Ok(())
    }

    #[test]
    fn test_is_null() -> Fallible<()> {
        let is_equal = make_is_null::<InherentNullDomain<AllDomain<_>>>()?;
        let arg = vec![f64::NAN, 1., 2.];
        let ret = is_equal.invoke(&arg)?;
        assert_eq!(ret, vec![true, false, false]);
        assert!(is_equal.check(&1, &1)?);
        Ok(())
    }
}


pub fn make_bin_grid_array2(
    lower_edges: Vec<f64>,
    upper_edges: Vec<f64>,
    bin_count: usize,
) -> Fallible<
    Transformation<
        Array2Domain<AllDomain<f64>>,
        Array2Domain<AllDomain<usize>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
> {
    use crate::traits::RoundCast;
    
    let bin_count = bin_count as f64;
    make_row_by_row_fallible(
        AllDomain::new(),
        AllDomain::new(),
        move |row: &Vec<f64>| {
            row.iter()
                .zip(lower_edges.iter().zip(upper_edges.iter()))
                .map(|(v, (u, l))| usize::round_cast(((v - l) / (u - l) * bin_count).floor()))
                .collect()
        },
    )
}