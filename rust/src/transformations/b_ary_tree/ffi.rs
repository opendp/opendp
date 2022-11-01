use std::{
    convert::TryFrom,
    os::raw::{c_char, c_uint},
};

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    ffi::{any::AnyTransformation, util::Type},
    metrics::{L1Distance, L2Distance},
    traits::{Integer, Number},
    transformations::{make_b_ary_tree, BAryTreeMetric},
};

use super::choose_branching_factor;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_b_ary_tree(
    leaf_count: c_uint,
    branching_factor: c_uint,
    M: *const c_char,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<Q>(
        leaf_count: usize,
        branching_factor: usize,
        M: Type,
        TA: Type,
    ) -> FfiResult<*mut AnyTransformation>
    where
        Q: Number,
    {
        fn monomorphize2<M, TA>(
            leaf_count: usize,
            branching_factor: usize,
        ) -> FfiResult<*mut AnyTransformation>
        where
            TA: Integer,
            (VectorDomain<AtomDomain<TA>>, M): MetricSpace,
            M: 'static + BAryTreeMetric,
            M::Distance: Number,
        {
            make_b_ary_tree::<M, TA>(leaf_count, branching_factor).into_any()
        }

        dispatch!(monomorphize2, [
            (M, [L1Distance<Q>, L2Distance<Q>]),
            (TA, @integers)
        ], (leaf_count, branching_factor))
    }

    let leaf_count = leaf_count as usize;
    let branching_factor = branching_factor as usize;
    let M = try_!(Type::try_from(M));
    let TA = try_!(Type::try_from(TA));
    let Q = try_!(M.get_atom());
    dispatch!(monomorphize, [
        (Q, @integers)
    ], (leaf_count, branching_factor, M, TA))
}

#[no_mangle]
pub extern "C" fn opendp_transformations__choose_branching_factor(size_guess: c_uint) -> c_uint {
    let size_guess = size_guess as usize;
    choose_branching_factor(size_guess) as c_uint
}
