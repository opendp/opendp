use std::any::Any;
use std::any::{self, type_name};

use crate::core::{Domain, Function, Measure, Measurement, Metric};
use crate::error::*;
use crate::interactive::{PolyQueryable, Queryable, QueryableFunctor, QueryableMap};

impl<TI: 'static, TO: 'static + QueryableMap> Function<TI, TO> {
    /// Converts this Function into one with polymorphic output.
    pub fn into_poly(self) -> Function<TI, Box<dyn QueryableFunctor>> {
        let function = move |arg: &TI| -> Fallible<Box<dyn QueryableFunctor>> {
            let res = self.eval(arg);
            res.map(|o| Box::new(o) as Box<dyn QueryableFunctor>)
        };
        Function::new_fallible(function)
    }
}

impl<TI: 'static, Q: 'static, A: QueryableMap> Function<TI, Queryable<Q, A>> {
    /// Converts this Function into one with polymorphic output.
    pub fn into_poly_queryable(self) -> Function<TI, PolyQueryable> {
        let function = move |arg: &TI| -> Fallible<PolyQueryable> {
            let mut res = self.eval(arg)?;
            Ok(Queryable::new_external(move |query: &dyn Any| {
                let query = query.downcast_ref::<Q>().ok_or_else(|| {
                    err!(
                        FailedFunction,
                        "failed to downcast dyn query to {}",
                        type_name::<Q>()
                    )
                })?;
                res.eval_mappable(query)
                    .map(|o| Box::new(o) as Box<dyn QueryableFunctor>)
            }))
        };
        Function::new_fallible(function)
    }
}

impl<TI> Function<TI, Box<dyn QueryableFunctor>> {
    pub fn eval_poly<TO: 'static>(&self, arg: &TI) -> Fallible<TO> {
        self.eval(arg)?
            .into_any()
            .downcast()
            .map_err(|_| {
                err!(
                    FailedCast,
                    "Failed downcast of eval_poly result to {}",
                    any::type_name::<TO>()
                )
            })
            .map(|res| *res)
    }
}

impl<DI, Q, A, MI, MO> Measurement<DI, Queryable<Q, A>, MI, MO>
where
    DI: 'static + Domain,
    DI::Carrier: 'static,
    Q: 'static,
    A: QueryableMap,
    MI: 'static + Metric,
    MO: 'static + Measure,
{
    /// Converts this Measurement into one with polymorphic output. This is useful for composition
    /// of heterogeneous Measurements.
    pub fn into_poly_queryable(self) -> Measurement<DI, PolyQueryable, MI, MO> {
        Measurement::new(
            self.input_domain,
            self.function.into_poly_queryable(),
            self.input_metric,
            self.output_measure,
            self.privacy_map,
        )
    }
}


impl<DI, TO, MI, MO> Measurement<DI, TO, MI, MO>
where
    DI: 'static + Domain,
    DI::Carrier: 'static,
    TO: 'static + QueryableMap,
    MI: 'static + Metric,
    MO: 'static + Measure,
{
    /// Converts this Measurement into one with polymorphic output. This is useful for composition
    /// of heterogeneous Measurements.
    pub fn into_poly(self) -> Measurement<DI, Box<dyn QueryableFunctor>, MI, MO> {
        Measurement::new(
            self.input_domain,
            self.function.into_poly(),
            self.input_metric,
            self.output_measure,
            self.privacy_map,
        )
    }
}

#[cfg(all(test, feature = "untrusted"))]
mod tests {
    use crate::domains::AllDomain;
    use crate::error::*;
    use crate::interactive::Static;
    use crate::measurements;

    #[test]
    fn test_poly_measurement() -> Fallible<()> {
        let op_plain = measurements::make_base_laplace::<AllDomain<_>>(0.0, None)?;
        let arg = 100.;
        let res_plain = op_plain.invoke(&arg)?;
        assert_eq!(res_plain, arg);

        // invoke interactively and with type-erasure
        let op_poly = op_plain.clone().interactive().into_poly_queryable();
        let res_poly = op_poly.invoke_poly::<(), Static<f64>>(&arg)?.get()?;
        assert_eq!(res_poly, arg);

        // invoke interactively and with type-erasure, but expect the wrong type
        let op_poly = op_plain.clone().interactive().into_poly_queryable();
        let res_bogus = op_poly.invoke1_poly::<Static<i32>>(&arg);
        assert_eq!(
            res_bogus.err().unwrap_test().variant,
            ErrorVariant::FailedCast
        );
        Ok(())
    }
}
