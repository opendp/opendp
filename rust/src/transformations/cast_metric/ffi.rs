use std::convert::TryFrom;
use std::os::raw::{c_char, c_uint};

use opendp_derive::bootstrap;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt, Transformation};
use crate::domains::{AllDomain, BoundedDomain, SizedDomain, VectorDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::metrics::{
    ChangeOneDistance, HammingDistance, InsertDeleteDistance, IntDistance, SymmetricDistance,
};
use crate::traits::{CheckNull, TotalOrd};
use crate::transformations::cast_metric::traits::{
    BoundedMetric, OrderedMetric, UnboundedMetric, UnorderedMetric,
};

#[bootstrap(
    name = "make_ordered_random", 
    module = "transformations", 
    features("contrib")
)]
/// Make a Transformation that converts the unordered dataset metric `SymmetricDistance`
/// to the respective ordered dataset metric InsertDeleteDistance by assigning a random permutatation.
/// Operates exclusively on VectorDomain<AllDomain<`TA`>>.
///
/// The dataset metric is not generic over ChangeOneDistance because the dataset size is unknown.
///
/// # Generics
/// * `TA` - Atomic Type.
pub fn make_ordered_random_wrapper<TA: Clone + CheckNull>() -> Fallible<
    Transformation<
        VectorDomain<AllDomain<TA>>,
        VectorDomain<AllDomain<TA>>,
        SymmetricDistance,
        InsertDeleteDistance,
    >,
> {
    super::make_ordered_random(Default::default())
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_ordered_random(
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<TA: 'static + Clone + CheckNull>() -> FfiResult<*mut AnyTransformation> {
        make_ordered_random_wrapper::<TA>().into_any()
    }
    dispatch!(monomorphize, [
        (TA, @primitives)
    ], ())
}

#[bootstrap(
    module = "transformations", 
    features("contrib"),
    generics(MI(hint = "DatasetMetric", default = "SymmetricDistance"))
)]
/// Make a Transformation that converts the unordered dataset metric `MI`
/// to the respective ordered dataset metric by assigning a random permutatation.
/// Operates exclusively on SizedDomain<VectorDomain<AllDomain<`TA`>>>.
///
/// If `MI` is "SymmetricDistance", then output metric is "InsertDeleteDistance",
/// and respectively "ChangeOneDistance" maps to "HammingDistance".
///
/// # Arguments
/// * `size` - Number of records in input data.
/// 
/// # Generics
/// * `MI` - Input Metric. One of "SymmetricDistance" or "ChangeOneDistance"
/// * `TA` - Atomic Type.
fn make_sized_ordered_random<MI, TA>(
    size: usize
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<AllDomain<TA>>>,
        SizedDomain<VectorDomain<AllDomain<TA>>>,
        MI,
        MI::OrderedMetric,
    >,
>
where
    MI: 'static + UnorderedMetric<Distance = IntDistance>,
    TA: 'static + Clone + CheckNull,
{
    let domain = SizedDomain::new(VectorDomain::new(AllDomain::<TA>::new()), size);
    super::make_ordered_random(domain)
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_sized_ordered_random(
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
        make_sized_ordered_random::<MI, TA>(size).into_any()
    }
    dispatch!(monomorphize, [
        (MI, [SymmetricDistance, ChangeOneDistance]),
        (TA, @primitives)
    ], (size))
}

#[bootstrap(
    module = "transformations", 
    features("contrib"),
    generics(MI(hint = "DatasetMetric", default = "SymmetricDistance"))
)]
/// Make a Transformation that converts the unordered dataset metric `MI`
/// to the respective ordered dataset metric by assigning a random permutatation.
/// Operates exclusively on SizedDomain<VectorDomain<BoundedDomain<`TA`>>>.
///
/// If `MI` is "SymmetricDistance", then output metric is "InsertDeleteDistance",
/// and respectively "ChangeOneDistance" maps to "HammingDistance".
/// 
/// # Arguments
/// * `size` - Number of records in input data.
/// * `bounds` - Tuple of inclusive lower and upper bounds.
/// 
/// # Generics
/// * `MI` - Input Metric. One of "SymmetricDistance" or "ChangeOneDistance"
/// * `TA` - Atomic Type.
fn make_sized_bounded_ordered_random<MI, TA>(
    size: usize,
    bounds: (TA, TA)
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<TA>>>,
        SizedDomain<VectorDomain<BoundedDomain<TA>>>,
        MI,
        MI::OrderedMetric,
    >,
>
where
    MI: 'static + UnorderedMetric<Distance = IntDistance>,
    TA: 'static + Clone + CheckNull + TotalOrd,
{
    let domain = SizedDomain::new(
        VectorDomain::new(BoundedDomain::<TA>::new_closed(bounds)?),
        size,
    );
    super::make_ordered_random(domain)
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_sized_bounded_ordered_random(
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
        make_sized_bounded_ordered_random::<MI, TA>(size, bounds).into_any()
    }
    dispatch!(monomorphize, [
        (MI, [SymmetricDistance, ChangeOneDistance]),
        (TA, @numbers)
    ], (size, bounds))
}


#[bootstrap(module = "transformations", features("contrib"))]
/// Make a Transformation that converts the ordered dataset metric `InsertDeleteDistance`
/// to the respective ordered dataset metric SymmetricDistance with a no-op.
/// Operates exclusively on VectorDomain<AllDomain<`TA`>>.
///
/// The dataset metric is not generic over HammingDistance because the dataset size is unknown.
/// 
/// # Generics
/// * `TA` - Atomic Type.
fn make_unordered<TA: Clone + CheckNull>() -> Fallible<
    Transformation<
        VectorDomain<AllDomain<TA>>,
        VectorDomain<AllDomain<TA>>,
        InsertDeleteDistance,
        SymmetricDistance,
    >,
> {
    super::make_unordered(Default::default())
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_unordered(
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let TA = try_!(Type::try_from(TA));

    fn monomorphize<TA: 'static + Clone + CheckNull>() -> FfiResult<*mut AnyTransformation> {
        make_unordered::<TA>().into_any()
    }
    dispatch!(monomorphize, [
        (TA, @primitives)
    ], ())
}

#[bootstrap(
    module = "transformations", 
    features("contrib"),
    generics(MI(hint = "DatasetMetric", default = "InsertDeleteDistance"))
)]
/// Make a Transformation that converts the ordered dataset metric `MI`
/// to the respective unordered dataset metric via a no-op.
/// Operates exclusively on SizedDomain<VectorDomain<AllDomain<`TA`>>>.
///
/// If `MI` is "InsertDeleteDistance", then output metric is "SymmetricDistance",
/// and respectively "HammingDistance" maps to "ChangeOneDistance".
///
/// # Arguments
/// * `size` - Number of records in input data.
/// 
/// # Generics
/// * `MI` - Input Metric. One of "InsertDeleteDistance" or "HammingDistance"
/// * `TA` - Atomic Type.
fn make_sized_unordered<MI, TA>(
    size: usize
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<AllDomain<TA>>>,
        SizedDomain<VectorDomain<AllDomain<TA>>>,
        MI,
        MI::UnorderedMetric,
    >,
>
where
    MI: 'static + OrderedMetric<Distance = IntDistance>,
    TA: 'static + Clone + CheckNull,
{
    let domain = SizedDomain::new(VectorDomain::new(AllDomain::<TA>::new()), size);
    super::make_unordered(domain)
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_sized_unordered(
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
        make_sized_unordered::<MI, TA>(size).into_any()
    }
    dispatch!(monomorphize, [
        (MI, [InsertDeleteDistance, HammingDistance]),
        (TA, @primitives)
    ], (size))
}

#[bootstrap(
    module = "transformations", 
    features("contrib"),
    generics(MI(hint = "DatasetMetric", default = "InsertDeleteDistance"))
)]
/// Make a Transformation that converts the ordered dataset metric `MI`
/// to the respective unordered dataset metric via a no-op.
/// Operates exclusively on SizedDomain<VectorDomain<BoundedDomain<`TA`>>>.
///
/// If `MI` is "InsertDeleteDistance", then output metric is "SymmetricDistance",
/// and respectively "HammingDistance" maps to "ChangeOneDistance".
///
/// # Arguments
/// * `size` - Number of records in input data.
/// * `bounds` - Tuple of inclusive lower and upper bounds.
/// 
/// # Generics
/// * `MI` - Input Metric. One of "InsertDeleteDistance" or "HammingDistance"
/// * `TA` - Atomic Type.
fn make_sized_bounded_unordered<MI, TA>(
    size: usize,
    bounds: (TA, TA)
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<TA>>>,
        SizedDomain<VectorDomain<BoundedDomain<TA>>>,
        MI,
        MI::UnorderedMetric,
    >,
>
where
    MI: 'static + OrderedMetric<Distance = IntDistance>,
    TA: 'static + Clone + CheckNull + TotalOrd,
{
    let domain = SizedDomain::new(VectorDomain::new(BoundedDomain::<TA>::new_closed(bounds)?), size);
    super::make_unordered(domain)
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_sized_bounded_unordered(
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
        make_sized_bounded_unordered::<MI, TA>(size, bounds).into_any()
    }
    dispatch!(monomorphize, [
        (MI, [InsertDeleteDistance, HammingDistance]),
        (TA, @numbers)
    ], (size, bounds))
}

#[bootstrap(
    module = "transformations", 
    features("contrib"),
    generics(MI(hint = "DatasetMetric", default = "SymmetricDistance"))
)]
/// Make a Transformation that converts the unbounded dataset metric `MI` 
/// to the respective bounded dataset metric with a no-op. 
/// 
/// Operates exclusively on SizedDomain<VectorDomain<AllDomain<`TA`>>>.
/// The constructor enforces that the input domain has known size, 
/// because it must have known size to be valid under a bounded dataset metric.
/// 
/// While it is valid to operate with bounded data, there is no constructor for it in Python.
/// 
/// If MI is "SymmetricDistance", then output metric is "ChangeOneDistance", 
/// and respectively "InsertDeleteDistance" maps to "HammingDistance".
///
/// # Arguments
/// * `size` - Number of records in input data.
/// 
/// # Generics
/// * `MI` - Input Metric. One of "SymmetricDistance" or "InsertDeleteDistance"
/// * `TA` - Atomic Type.
fn make_metric_bounded<MI, TA>(
    size: usize
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<AllDomain<TA>>>,
        SizedDomain<VectorDomain<AllDomain<TA>>>,
        MI,
        MI::BoundedMetric,
    >,
>
where
    MI: 'static + UnboundedMetric<Distance = IntDistance>,
    TA: 'static + Clone + CheckNull,
{
    let domain = SizedDomain::new(VectorDomain::new_all(), size);
    super::make_metric_bounded(domain)
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_metric_bounded(
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
        make_metric_bounded::<MI, TA>(size).into_any()
    }
    let size = size as usize;
    dispatch!(monomorphize, [
        (MI, [SymmetricDistance, InsertDeleteDistance]),
        (TA, @primitives)
    ], (size))
}


#[bootstrap(
    module = "transformations", 
    features("contrib"),
    generics(MI(hint = "DatasetMetric", default = "ChangeOneDistance"))
)]
/// Make a Transformation that converts the bounded dataset metric `MI` 
/// to the respective unbounded dataset metric with a no-op. 
/// Operates exclusively on SizedDomain<VectorDomain<AllDomain<`TA`>>>.
/// 
/// If "ChangeOneDistance", then output metric is "SymmetricDistance", 
/// and respectively "HammingDistance" maps to "InsertDeleteDistance".
///
/// # Arguments
/// * `size` - Number of records in input data.
/// 
/// # Generics
/// * `MI` - Input Metric. One of "ChangeOneDistance" or "HammingDistance"
/// * `TA` - Atomic Type.
fn make_metric_unbounded<MI, TA>(
    size: usize
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<AllDomain<TA>>>,
        SizedDomain<VectorDomain<AllDomain<TA>>>,
        MI,
        MI::UnboundedMetric,
    >,
>
where
    MI: 'static + BoundedMetric<Distance = IntDistance>,
    TA: 'static + Clone + CheckNull,
{
    let domain = SizedDomain::new(VectorDomain::new_all(), size);
    super::make_metric_unbounded(domain)
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_metric_unbounded(
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
        make_metric_unbounded::<MI, TA>(size).into_any()
    }
    let size = size as usize;
    dispatch!(monomorphize, [
        (MI, [ChangeOneDistance, HammingDistance]),
        (TA, @primitives)
    ], (size))
}
