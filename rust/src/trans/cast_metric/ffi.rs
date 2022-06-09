use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::dist::{
    ChangeOneDistance, HammingDistance, InsertDeleteDistance, IntDistance, SymmetricDistance,
};
use crate::dom::{AllDomain, SizedDomain, VectorDomain};
use crate::err;
use crate::ffi::any::AnyTransformation;
use crate::ffi::util::Type;
use crate::traits::CheckNull;
use crate::trans::cast_metric::traits::{
    BoundedMetric, OrderedMetric, UnboundedMetric, UnorderedMetric,
};
use crate::trans::{
    make_metric_bounded, make_metric_unbounded, make_random_ordering, make_unordered,
};

#[no_mangle]
pub extern "C" fn opendp_trans__make_random_ordering(
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<TA>() -> FfiResult<*mut AnyTransformation>
    where
        TA: 'static + Clone + CheckNull,
    {
        make_random_ordering::<VectorDomain<AllDomain<TA>>, SymmetricDistance>(
            VectorDomain::new_all(),
        )
        .into_any()
    }
    dispatch!(monomorphize, [
        (TA, @primitives)
    ], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_random_ordering(
    size: c_uint,
    MI: *const c_char,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let size = size as usize;
    let MI = try_!(Type::try_from(MI));
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<MI, TA>(size: usize) -> FfiResult<*mut AnyTransformation>
    where
        MI: 'static + UnorderedMetric<Distance = IntDistance>,
        TA: 'static + Clone + CheckNull,
    {
        let domain = SizedDomain::new(VectorDomain::new(AllDomain::<TA>::new()), size);
        make_random_ordering::<_, MI>(domain).into_any()
    }
    dispatch!(monomorphize, [
        (MI, [SymmetricDistance, ChangeOneDistance]),
        (TA, @primitives)
    ], (size))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_unordered(
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<TA>() -> FfiResult<*mut AnyTransformation>
    where
        TA: 'static + Clone + CheckNull,
    {
        make_unordered::<VectorDomain<AllDomain<TA>>, InsertDeleteDistance>(VectorDomain::new_all()).into_any()
    }
    dispatch!(monomorphize, [
        (TA, @primitives)
    ], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_unordered(
    size: c_uint,
    MI: *const c_char,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let size = size as usize;
    let MI = try_!(Type::try_from(MI));
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<MI, TA>(size: usize) -> FfiResult<*mut AnyTransformation>
    where
        MI: 'static + OrderedMetric<Distance = IntDistance>,
        TA: 'static + Clone + CheckNull,
    {
        let domain = SizedDomain::new(VectorDomain::new(AllDomain::<TA>::new()), size);
        make_unordered::<_, MI>(domain).into_any()
    }
    dispatch!(monomorphize, [
        (MI, [InsertDeleteDistance, HammingDistance]),
        (TA, @primitives)
    ], (size))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_metric_bounded(
    size: c_uint,
    MI: *const c_char,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let MI = try_!(Type::try_from(MI));
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<MI, TA>(size: usize) -> FfiResult<*mut AnyTransformation>
    where
        MI: 'static + UnboundedMetric<Distance = IntDistance>,
        TA: 'static + Clone + CheckNull,
    {
        let domain = SizedDomain::new(VectorDomain::new_all(), size);
        make_metric_bounded::<VectorDomain<AllDomain<TA>>, MI>(domain).into_any()
    }
    let size = size as usize;
    dispatch!(monomorphize, [
        (MI, [SymmetricDistance, InsertDeleteDistance]),
        (TA, @primitives)
    ], (size))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_metric_unbounded(
    size: c_uint,
    MI: *const c_char,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let MI = try_!(Type::try_from(MI));
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<MI, TA>(size: usize) -> FfiResult<*mut AnyTransformation>
    where
        MI: 'static + BoundedMetric<Distance = IntDistance>,
        TA: 'static + Clone + CheckNull,
    {
        let domain = SizedDomain::new(VectorDomain::new_all(), size);
        make_metric_unbounded::<VectorDomain<AllDomain<TA>>, MI>(domain).into_any()
    }
    let size = size as usize;
    dispatch!(monomorphize, [
        (MI, [ChangeOneDistance, HammingDistance]),
        (TA, @primitives)
    ], (size))
}
