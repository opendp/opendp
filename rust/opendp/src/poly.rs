use std::any;
use std::any::Any;

use crate::core::{Domain, Function, Measure, Measurement, Metric, Transformation};
use crate::error::*;
use std::fmt::{Formatter, Debug};

/// A polymorphic Domain. This admits any value of any type (represented as a Box<dyn Any>).
#[derive(PartialEq, Clone)]
pub struct PolyDomain {}

impl PolyDomain {
    pub fn new() -> Self { PolyDomain {} }
}
impl Debug for PolyDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "PolyDomain()")
    }
}
impl Domain for PolyDomain {
    type Carrier = Box<dyn Any>;
    fn member(&self, _val: &Self::Carrier) -> Fallible<bool> { Ok(true) }
}

impl<DI, DO> Function<DI, DO>
    where DI: 'static + Domain,
          DI::Carrier: 'static,
          DO: 'static + Domain,
          DO::Carrier: 'static {
    /// Converts this Function into one with polymorphic output.
    pub fn into_poly(self) -> Function<DI, PolyDomain> {
        let function = move |arg: &DI::Carrier| -> Fallible<<PolyDomain as Domain>::Carrier> {
            let res = self.eval(arg);
            res.map(|o| Box::new(o) as Box<dyn Any>)
        };
        Function::new_fallible(function)
    }
}

impl<DI: Domain> Function<DI, PolyDomain> {
    pub fn eval_poly<T: 'static>(&self, arg: &DI::Carrier) -> Fallible<T> {
        self.eval(arg)?.downcast().map_err(|_| err!(FailedCast, "Failed downcast of eval_poly result to {}", any::type_name::<T>())).map(|res| *res)
    }
}

impl<DI, DO, MI, MO> Measurement<DI, DO, MI, MO>
    where DI: 'static + Domain,
          DI::Carrier: 'static,
          DO: 'static + Domain,
          DO::Carrier: 'static,
          MI: 'static + Metric,
          MO: 'static + Measure {
    /// Converts this Measurement into one with polymorphic output. This is useful for composition
    /// of heterogeneous Measurements.
    pub fn into_poly(self) -> Measurement<DI, PolyDomain, MI, MO> {
        Measurement::new(
            self.input_domain,
            PolyDomain::new(),
            self.function.into_poly(),
            self.input_metric,
            self.output_measure,
            self.privacy_relation,
        )
    }
}

impl<DI, DO, MI, MO> Transformation<DI, DO, MI, MO>
    where DI: 'static + Domain,
          DI::Carrier: 'static,
          DO: 'static + Domain,
          DO::Carrier: 'static,
          MI: 'static + Metric,
          MO: 'static + Metric {
    /// Converts this Transformation into one with polymorphic output. It's not clear if we'll need this,
    /// but it's provided for symmetry with Measurement.
    pub fn into_poly(self) -> Transformation<DI, PolyDomain, MI, MO> {
        Transformation::new(
            self.input_domain,
            PolyDomain::new(),
            self.function.into_poly(),
            self.input_metric,
            self.output_metric,
            self.stability_relation,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::dist::SubstituteDistance;
    use crate::dom::AllDomain;
    use crate::error::*;
    use crate::meas;
    use crate::trans;

    #[test]
    fn test_poly_measurement() -> Fallible<()> {
        let op_plain = meas::make_base_laplace::<AllDomain<_>>(0.0)?;
        let arg = 99.9;
        let res_plain = op_plain.invoke(&arg)?;
        assert_eq!(res_plain, arg);
        let op_poly = op_plain.into_poly();
        let res_poly = op_poly.function.eval_poly::<f64>(&arg)?;
        assert_eq!(res_poly, arg);
        let res_bogus = op_poly.function.eval_poly::<i32>(&arg);
        assert_eq!(res_bogus.err().unwrap_test().variant, ErrorVariant::FailedCast);
        Ok(())
    }

    #[test]
    fn test_poly_transformation() -> Fallible<()> {
        let op_plain = trans::make_identity(AllDomain::new(), SubstituteDistance::default())?;
        let arg = 99.9;
        let res_plain = op_plain.invoke(&arg)?;
        assert_eq!(res_plain, arg);
        let op_poly = op_plain.into_poly();
        let res_poly = op_poly.function.eval_poly::<f64>(&arg)?;
        assert_eq!(res_poly, arg);
        let res_bogus = op_poly.function.eval_poly::<i32>(&arg);
        assert_eq!(res_bogus.err().unwrap_test().variant, ErrorVariant::FailedCast);
        Ok(())
    }
}
