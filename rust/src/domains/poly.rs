use std::any::Any;
use std::any::{self, type_name};

use crate::core::{Domain, Function, Measure, Metric, Transformation, Measurement};
use crate::error::*;
use crate::interactive::Queryable;
use std::fmt::{Debug, Formatter};

use super::QueryableDomain;

/// A polymorphic Domain. This admits any value of any type (represented as a Box<dyn Any>).
#[derive(PartialEq, Clone)]
pub struct PolyDomain {}

impl PolyDomain {
    pub fn new() -> Self {
        PolyDomain {}
    }
}
impl Default for PolyDomain {
    fn default() -> Self {
        Self::new()
    }
}
impl Debug for PolyDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "PolyDomain()")
    }
}
impl Domain for PolyDomain {
    type Carrier = Box<dyn Any>;
    fn member(&self, _val: &Self::Carrier) -> Fallible<bool> {
        Ok(true)
    }
}

impl<DI, DO> Function<DI, DO>
where
    DI: 'static + Domain,
    DI::Carrier: 'static,
    DO: 'static + Domain,
    DO::Carrier: 'static,
{
    /// Converts this Function into one with polymorphic output.
    pub fn into_poly(self) -> Function<DI, PolyDomain> {
        let function = move |arg: &DI::Carrier| -> Fallible<<PolyDomain as Domain>::Carrier> {
            let res = self.eval(arg);
            res.map(|o| Box::new(o) as Box<dyn Any>)
        };
        Function::new_fallible(function)
    }
}

impl<DI, DOQ, DOA> Function<DI, QueryableDomain<DOQ, DOA>>
where
    DI: 'static + Domain,
    DI::Carrier: 'static,
    DOQ: 'static + Domain,
    DOQ::Carrier: 'static,
    DOA: 'static + Domain,
    DOA::Carrier: 'static,
{
    /// Converts this Function into one with polymorphic output.
    pub fn into_poly_queryable(self) -> Function<DI, QueryableDomain<PolyDomain, PolyDomain>> {
        let function = move |arg: &DI::Carrier| -> Fallible<Queryable<Box<dyn Any>, Box<dyn Any>>> {
            let mut res = self.eval(arg)?;
            Ok(Queryable::new_concrete(move |query: &Box<dyn Any>| {
                let query = query.downcast_ref::<DOQ::Carrier>().ok_or_else(|| {
                    err!(
                        FailedFunction,
                        "failed to downcast query to {}",
                        type_name::<DI::Carrier>()
                    )
                })?;
                res.eval(query).map(|o| Box::new(o) as Box<dyn Any>)
            }))
        };
        Function::new_fallible(function)
    }
}

// TODO: eval poly for QueryableDomain<PolyDomain, PolyDomain> and family
impl<DI: Domain> Function<DI, PolyDomain> {
    pub fn eval_poly<T: 'static>(&self, arg: &DI::Carrier) -> Fallible<T> {
        self.eval(arg)?
            .downcast()
            .map_err(|_| {
                err!(
                    FailedCast,
                    "Failed downcast of eval_poly result to {}",
                    any::type_name::<T>()
                )
            })
            .map(|res| *res)
    }
}

impl<DI: Domain> Function<DI, QueryableDomain<PolyDomain, PolyDomain>> {
    pub fn eval_poly<Q: 'static + Clone, A: 'static>(&self, arg: &DI::Carrier) -> Fallible<Queryable<Q, A>> {
        let mut queryable = self.eval(arg)?;

        Ok(Queryable::new_concrete(move |query: &Q| {
            queryable.eval(&(Box::new(query.clone()) as Box<dyn Any>))?.downcast()
            .map_err(|_| {
                err!(
                    FailedCast,
                    "Failed downcast of eval_poly result to {}",
                    any::type_name::<A>()
                )
            })
            .map(|res| *res)
        }))
    }

    pub fn eval1_poly<A: 'static>(&self, arg: &DI::Carrier) -> Fallible<A> {
        self.eval_poly::<(), A>(arg)?.get()
    }
}

impl<DI, DOQ, DOA, MI, MO> Measurement<DI, DOQ, DOA, MI, MO>
where
    DI: 'static + Domain,
    DI::Carrier: 'static,
    DOQ: 'static + Domain,
    DOQ::Carrier: 'static,
    DOA: 'static + Domain,
    DOA::Carrier: 'static,
    MI: 'static + Metric,
    MO: 'static + Measure,
{
    /// Converts this Measurement into one with polymorphic output. This is useful for composition
    /// of heterogeneous Measurements.
    pub fn into_poly(self) -> Measurement<DI, PolyDomain, PolyDomain, MI, MO> {
        Measurement::new(
            self.input_domain,
            PolyDomain::new(), 
            PolyDomain::new(),
            self.function.into_poly_queryable(),
            self.input_metric,
            self.output_measure,
            self.privacy_map,
        )
    }
}

impl<DI, DO, MI, MO> Transformation<DI, DO, MI, MO>
where
    DI: 'static + Domain,
    DI::Carrier: 'static,
    DO: 'static + Domain,
    DO::Carrier: 'static,
    MI: 'static + Metric,
    MO: 'static + Metric,
{
    /// Converts this Transformation into one with polymorphic output. It's not clear if we'll need this,
    /// but it's provided for symmetry with Measurement.
    pub fn into_poly(self) -> Transformation<DI, PolyDomain, MI, MO> {
        Transformation::new(
            self.input_domain,
            PolyDomain::new(),
            self.function.into_poly(),
            self.input_metric,
            self.output_metric,
            self.stability_map,
        )
    }
}

#[cfg(all(test, feature = "untrusted"))]
mod tests {
    use crate::domains::AllDomain;
    use crate::error::*;
    use crate::measurements;
    use crate::metrics::ChangeOneDistance;
    use crate::transformations;

    #[test]
    fn test_poly_measurement() -> Fallible<()> {
        let op_plain = measurements::make_base_laplace::<AllDomain<_>>(0.0, None)?;
        let arg = 100.;
        let res_plain = op_plain.invoke1(&arg)?;
        assert_eq!(res_plain, arg);
        let op_poly = op_plain.into_poly();
        let res_poly = op_poly.invoke1_poly::<f64>(&arg)?;
        assert_eq!(res_poly, arg);
        let res_bogus = op_poly.invoke1_poly::<i32>(&arg);
        assert_eq!(
            res_bogus.err().unwrap_test().variant,
            ErrorVariant::FailedCast
        );
        Ok(())
    }

    #[test]
    fn test_poly_transformation() -> Fallible<()> {
        let op_plain =
            transformations::make_identity(AllDomain::new(), ChangeOneDistance::default())?;
        let arg = 99.9;
        let res_plain = op_plain.invoke(&arg)?;
        assert_eq!(res_plain, arg);
        let op_poly = op_plain.into_poly();
        let res_poly = op_poly.function.eval_poly::<f64>(&arg)?;
        assert_eq!(res_poly, arg);
        let res_bogus = op_poly.function.eval_poly::<i32>(&arg);
        assert_eq!(
            res_bogus.err().unwrap_test().variant,
            ErrorVariant::FailedCast
        );
        Ok(())
    }
}
