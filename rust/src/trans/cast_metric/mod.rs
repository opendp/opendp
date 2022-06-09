use crate::{
    core::{Domain, Function, StabilityRelation, Transformation},
    dist::IntDistance,
    dom::SizedDomain,
    error::Fallible,
};

use self::traits::{
    BoundedMetric, OrderedMetric, ShuffleableDomain, UnboundedMetric, UnorderedMetric,
};

#[cfg(feature = "ffi")]
mod ffi;
mod traits;

pub fn make_random_ordering<D, MI>(
    domain: D,
) -> Fallible<Transformation<D, D, MI, MI::OrderedMetric>>
where
    D: ShuffleableDomain,
    D::Carrier: Clone,
    MI: UnorderedMetric<Distance = IntDistance>,
{
    Ok(Transformation::new(
        domain.clone(),
        domain,
        Function::new_fallible(|val: &D::Carrier| D::shuffle(val.clone())),
        MI::default(),
        MI::OrderedMetric::default(),
        StabilityRelation::new_from_constant(1),
    ))
}

pub fn make_unordered<D, MI>(domain: D) -> Fallible<Transformation<D, D, MI, MI::UnorderedMetric>>
where
    D: Domain,
    D::Carrier: Clone,
    MI: OrderedMetric<Distance = IntDistance>,
{
    Ok(Transformation::new(
        domain.clone(),
        domain,
        Function::new(|val: &D::Carrier| val.clone()),
        MI::default(),
        MI::UnorderedMetric::default(),
        StabilityRelation::new_from_constant(1),
    ))
}

pub fn make_metric_unbounded<D, MI>(
    domain: SizedDomain<D>,
) -> Fallible<Transformation<SizedDomain<D>, SizedDomain<D>, MI, MI::UnboundedMetric>>
where
    D: Domain,
    D::Carrier: Clone,
    MI: BoundedMetric<Distance = IntDistance>,
{
    Ok(Transformation::new(
        domain.clone(),
        domain,
        Function::new(|arg: &D::Carrier| arg.clone()),
        MI::default(),
        MI::UnboundedMetric::default(),
        StabilityRelation::new_from_forward(|d_in| Ok(d_in * 2)),
    ))
}

pub fn make_metric_bounded<D, MI>(
    domain: SizedDomain<D>,
) -> Fallible<Transformation<SizedDomain<D>, SizedDomain<D>, MI, MI::BoundedMetric>>
where
    D: Domain,
    D::Carrier: Clone,
    MI: UnboundedMetric<Distance = IntDistance>,
{
    Ok(Transformation::new(
        domain.clone(),
        domain,
        Function::new(|arg: &D::Carrier| arg.clone()),
        MI::default(),
        MI::BoundedMetric::default(),
        StabilityRelation::new_from_forward(|d_in| Ok(d_in / 2)),
    ))
}


#[cfg(test)]
mod test {
    use crate::{dom::VectorDomain, dist::SymmetricDistance};

    use super::*;

    #[test]
    fn test_ordering() -> Fallible<()> {
        let domain = VectorDomain::new_all();
        let ord_trans = make_random_ordering::<_, SymmetricDistance>(domain.clone())?;
        let data = vec![1i32, 2, 3];
        assert_eq!(ord_trans.invoke(&data)?.len(), 3);


        let ident_trans = (ord_trans >> make_unordered(domain)?)?;
        assert_eq!(ident_trans.invoke(&data)?.len(), 3);
        Ok(())
    }

    #[test]
    fn test_bounded() -> Fallible<()> {
        let domain = SizedDomain::new(VectorDomain::new_all(), 3);
        let bdd_trans = make_metric_bounded::<_, SymmetricDistance>(domain.clone())?;
        let data = vec![1i32, 2, 3];
        assert_eq!(bdd_trans.invoke(&data)?.len(), 3);


        let ident_trans = (bdd_trans >> make_metric_unbounded(domain)?)?;
        assert_eq!(ident_trans.invoke(&data)?.len(), 3);
        Ok(())
    }
}