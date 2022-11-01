use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use az::SaturatingCast;
use rug::{Integer, Rational};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, MetricSpace};
use crate::domains::{AllDomain, VectorDomain};
use crate::ffi::any::AnyMeasurement;
use crate::ffi::util::Type;
use crate::measurements::{
    make_base_discrete_gaussian, DiscreteGaussianDomain, DiscreteGaussianMeasure,
};
use crate::measures::ZeroConcentratedDivergence;
use crate::traits::{CheckNull, Float, InfCast, Number};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_discrete_gaussian(
    scale: *const c_void,
    D: *const c_char,
    MO: *const c_char,
    QI: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T, QI, QO>(
        scale: *const c_void,
        D: Type,
        MO: Type,
        QI: Type,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        T: 'static + Clone + CheckNull,
        Integer: From<T> + SaturatingCast<T>,

        QI: Number,
        QO: Float + InfCast<QI>,
        Rational: TryFrom<QO>,
    {
        fn monomorphize2<D, MO, QI>(scale: MO::Atom) -> FfiResult<*mut AnyMeasurement>
        where
            D: 'static + DiscreteGaussianDomain<QI>,
            (D, D::InputMetric): MetricSpace,
            Integer: From<D::Atom> + SaturatingCast<D::Atom>,

            MO: 'static + DiscreteGaussianMeasure<D, QI>,
            Rational: TryFrom<MO::Atom>,

            QI: Number,
        {
            make_base_discrete_gaussian::<D, MO, QI>(scale).into_any()
        }
        let scale = *try_as_ref!(scale as *const QO);
        dispatch!(monomorphize2, [
            (D, [VectorDomain<AllDomain<T>>, AllDomain<T>]),
            (MO, [ZeroConcentratedDivergence<QO>]),
            (QI, [QI])
        ], (scale))
    }

    let D = try_!(Type::try_from(D));
    let MO = try_!(Type::try_from(MO));
    let QI = try_!(Type::try_from(QI));
    let T = try_!(D.get_atom());
    let QO = try_!(MO.get_atom());

    dispatch!(monomorphize, [
        (T, @integers),
        (QI, @numbers),
        (QO, @floats)
    ], (scale, D, MO, QI))
}

#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_base_discrete_gaussian() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_discrete_gaussian(
            util::into_raw(0.0) as *const c_void,
            "AllDomain<i32>".to_char_p(),
            "ZeroConcentratedDivergence<f64>".to_char_p(),
            "i32".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }
}
