use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use num::Float;

use opendp::dom::{AllDomain, VectorDomain};
use opendp::err;
use opendp::meas::{GeometricDomain, make_base_geometric};
use opendp::traits::DistanceCast;

use crate::any::{AnyMeasurement, AnyObject, Downcast};
use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::util::Type;

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_geometric(
    scale: *const c_void,
    bounds: *const AnyObject,
    D: *const c_char, QO: *const c_char
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<D, QO>(
        scale: *const c_void, bounds: *const AnyObject
    ) -> FfiResult<*mut AnyMeasurement>
        where D: 'static + GeometricDomain,
              D::Atom: 'static + DistanceCast + PartialOrd,
              QO: 'static + Float + DistanceCast,
              f64: From<QO> {
        let scale = try_as_ref!(scale as *const QO).clone();
        let bounds = try_!(try_as_ref!(bounds).downcast_ref::<Option<(D::Atom, D::Atom)>>()).clone();
        make_base_geometric::<D, QO>(scale, bounds).into_any()
    }
    let D = try_!(Type::try_from(D));
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (D, [
            AllDomain<u8>, AllDomain<u16>, AllDomain<u32>, AllDomain<u64>, AllDomain<u128>,
            AllDomain<i8>, AllDomain<i16>, AllDomain<i32>, AllDomain<i64>, AllDomain<i128>,
            VectorDomain<AllDomain<u8>>, VectorDomain<AllDomain<u16>>, VectorDomain<AllDomain<u32>>,
            VectorDomain<AllDomain<u64>>, VectorDomain<AllDomain<u128>>, VectorDomain<AllDomain<i8>>,
            VectorDomain<AllDomain<i16>>, VectorDomain<AllDomain<i32>>, VectorDomain<AllDomain<i64>>,
            VectorDomain<AllDomain<i128>>
        ]),
        (QO, @floats)
    ], (scale, bounds))
}


#[cfg(test)]
mod tests {
    use opendp::error::Fallible;

    use crate::any::{AnyObject, Downcast};
    use crate::core;
    use crate::util;
    use crate::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_base_simple_geometric() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_geometric(
            util::into_raw(0.0) as *const c_void,
            // AnyObject::new_raw(None::<(i32, i32)>),
            "AllDomain<i32>".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }

    #[test]
    fn test_make_base_simple_constant_time_geometric() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_geometric(
            util::into_raw(0.0) as *const c_void,
            // util::into_raw(AnyObject::new(Some((0, 100)))),
            "AllDomain<i32>".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }
}
