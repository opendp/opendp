use num::Zero;
use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    ffi::{
        any::{AnyMeasurement, AnyObject, IntoAnyMeasurementOutExt, Downcast, AnyMeasure},
        util::AnyMeasurementPtr,
    }, error::Fallible, traits::InfAdd, measures::{MaxDivergence, FixedSmoothedMaxDivergence, ZeroConcentratedDivergence},
};

use super::BasicCompositionMeasure;

#[bootstrap(
    features("contrib"),
    arguments(measurements(rust_type = "Vec<AnyMeasurementPtr>")),
    dependencies("$get_dependencies_iterable(measurements)")
)]
/// Construct the DP composition [`measurement0`, `measurement1`, ...]. 
/// Returns a Measurement that when invoked, computes `[measurement0(x), measurement1(x), ...]`
/// 
/// All metrics and domains must be equivalent, except for the output domain.
/// 
/// # Arguments
/// * `measurements` - A vector of Measurements to compose.
fn make_basic_composition(measurements: Vec<&AnyMeasurement>) -> Fallible<AnyMeasurement> {
    super::make_basic_composition(measurements).map(IntoAnyMeasurementOutExt::into_any_out)
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_basic_composition(
    measurements: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let meas_ptrs = try_!(try_as_ref!(measurements).downcast_ref::<Vec<AnyMeasurementPtr>>());

    let measurements: Vec<&AnyMeasurement> =
        try_!(meas_ptrs.iter().map(|ptr| Ok(try_as_ref!(*ptr))).collect());

    make_basic_composition(measurements)
        .into()
}


impl BasicCompositionMeasure for AnyMeasure {
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        fn monomorphize1<Q: 'static + Clone + InfAdd + Zero>(
            self_: &AnyMeasure, d_i: Vec<AnyObject>
        ) -> Fallible<AnyObject> {
            fn monomorphize2<M: 'static + BasicCompositionMeasure>(
                self_: &AnyMeasure, d_i: Vec<AnyObject>
            ) -> Fallible<AnyObject>
                where M::Distance: Clone {
                self_.downcast_ref::<M>()?.compose(d_i.iter()
                    .map(|d_i| d_i.downcast_ref::<M::Distance>().map(Clone::clone))
                    .collect::<Fallible<Vec<M::Distance>>>()?).map(AnyObject::new)
            }
            dispatch!(monomorphize2, [
                (self_.type_, [MaxDivergence<Q>, FixedSmoothedMaxDivergence<Q>, ZeroConcentratedDivergence<Q>])
            ], (self_, d_i))
        }

        let Q_Atom = try_!(self.type_.get_atom());
        dispatch!(monomorphize1, [(Q_Atom, @floats)], (self, d_i))
    }
}

#[cfg(test)]
mod tests {
    use crate::combinators::tests::make_test_measurement;
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast, IntoAnyMeasurementExt};
    use crate::ffi::util;

    use super::*;

    #[test]
    fn test_make_basic_composition_ffi() -> Fallible<()> {
        let measurement0 = util::into_raw(make_test_measurement::<i32>().into_any()) as AnyMeasurementPtr;
        let measurement1 = util::into_raw(make_test_measurement::<i32>().into_any()) as AnyMeasurementPtr;
        let measurements = vec![measurement0, measurement1];
        let basic_composition =
            Result::from(opendp_combinators__make_basic_composition(
                AnyObject::new_raw(measurements),
            ))?;
        let arg = AnyObject::new_raw(999);
        let res = core::opendp_core__measurement_invoke(&basic_composition, arg);
        let res: Vec<AnyObject> = Fallible::from(res)?.downcast()?;
        let res = (*res[0].downcast_ref::<i32>()?, *res[1].downcast_ref::<i32>()?);
        println!("{:?}", res);
        assert_eq!(res, (999, 999));
        Ok(())
    }
}
