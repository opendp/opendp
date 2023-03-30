use std::any;
use std::any::Any;

use crate::core::{Domain, Function, Measure, Measurement, Metric};
use crate::error::*;

impl<TI: 'static, TO: 'static> Function<TI, TO> {
    /// Converts this Function into one with polymorphic output.
    pub fn into_poly(self) -> Function<TI, Box<dyn Any>> {
        let function = move |arg: &TI| -> Fallible<Box<dyn Any>> {
            let res = self.eval(arg);
            res.map(|o| Box::new(o) as Box<dyn Any>)
        };
        Function::new_fallible(function)
    }
}

impl<TI> Function<TI, Box<dyn Any>> {
    pub fn eval_poly<TO: 'static>(&self, arg: &TI) -> Fallible<TO> {
        self.eval(arg)?
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

impl<DI, TO, MI, MO> Measurement<DI, TO, MI, MO>
where
    DI: 'static + Domain,
    DI::Carrier: 'static,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + Measure,
{
    /// Converts this Measurement into one with polymorphic output. This is useful for composition
    /// of heterogeneous Measurements.
    pub fn into_poly(self) -> Measurement<DI, Box<dyn Any>, MI, MO> {
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
    use crate::measurements;

    #[test]
    fn test_poly_measurement() -> Fallible<()> {
        let op_plain = measurements::make_base_laplace::<AllDomain<_>>(0.0, None)?;
        let arg = 100.;
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
