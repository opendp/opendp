use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::core::{
    ChangeOneDistance, HammingDistance, InsertDeleteDistance, IntDistance, SymmetricDistance,
};
use crate::core::{AllDomain, SizedDomain, VectorDomain, BoundedDomain};
use crate::err;
use crate::ffi::any::{AnyTransformation, AnyObject, Downcast};
use crate::ffi::util::Type;
use crate::traits::{CheckNull, TotalOrd};
use crate::trans::cast_metric::traits::{
    BoundedMetric, OrderedMetric, UnboundedMetric, UnorderedMetric,
};
use crate::trans::{
    make_metric_bounded, make_metric_unbounded, make_ordered_random, make_unordered,
};

#[no_mangle]
pub extern "C" fn opendp_trans__make_ordered_random(
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<TA>() -> FfiResult<*mut AnyTransformation>
    where
        TA: 'static + Clone + CheckNull,
    {
        make_ordered_random::<VectorDomain<AllDomain<TA>>, SymmetricDistance>(
            VectorDomain::new_all(),
        )
        .into_any()
    }
    dispatch!(monomorphize, [
        (TA, @primitives)
    ], ())
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_ordered_random(
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
        make_ordered_random::<_, MI>(domain).into_any()
    }
    dispatch!(monomorphize, [
        (MI, [SymmetricDistance, ChangeOneDistance]),
        (TA, @primitives)
    ], (size))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_bounded_ordered_random(
    size: c_uint,
    bounds: *const AnyObject,
    MI: *const c_char,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let size = size as usize;
    let bounds = try_as_ref!(bounds);
    let MI = try_!(Type::try_from(MI));
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<MI, TA>(size: usize, bounds: &AnyObject) -> FfiResult<*mut AnyTransformation>
    where
        MI: 'static + UnorderedMetric<Distance = IntDistance>,
        TA: 'static + Clone + CheckNull + TotalOrd,
    {
        let bounds = try_!(bounds.downcast_ref::<(TA, TA)>()).clone();
        let domain = SizedDomain::new(VectorDomain::new(try_!(BoundedDomain::<TA>::new_closed(bounds))), size);
        make_ordered_random::<_, MI>(domain).into_any()
    }
    dispatch!(monomorphize, [
        (MI, [SymmetricDistance, ChangeOneDistance]),
        (TA, @numbers)
    ], (size, bounds))
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
pub extern "C" fn opendp_trans__make_sized_bounded_unordered(
    size: c_uint,
    bounds: *const AnyObject,
    MI: *const c_char,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let size = size as usize;
    let bounds = try_as_ref!(bounds);
    let MI = try_!(Type::try_from(MI));
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<MI, TA>(size: usize, bounds: &AnyObject) -> FfiResult<*mut AnyTransformation>
    where
        MI: 'static + OrderedMetric<Distance = IntDistance>,
        TA: 'static + Clone + CheckNull + TotalOrd,
    {
        let bounds = try_!(bounds.downcast_ref::<(TA, TA)>()).clone();
        let domain = SizedDomain::new(VectorDomain::new(try_!(BoundedDomain::<TA>::new_closed(bounds))), size);
        make_unordered::<_, MI>(domain).into_any()
    }
    dispatch!(monomorphize, [
        (MI, [InsertDeleteDistance, HammingDistance]),
        (TA, @numbers)
    ], (size, bounds))
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
