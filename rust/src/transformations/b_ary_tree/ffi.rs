use std::{
    convert::TryFrom,
    os::raw::{c_char, c_uint},
};

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    ffi::{any::AnyTransformation, util::Type},
    metrics::{L1Distance, L2Distance},
    traits::{Integer, Number},
    trans::{make_b_ary_tree, BAryTreeMetric},
};

#[no_mangle]
pub extern "C" fn opendp_trans__make_b_ary_tree(
    num_bins: c_uint,
    b: c_uint,
    T: *const c_char,
    M: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<Q>(
        num_bins: usize,
        b: usize,
        M: Type,
        T: Type,
    ) -> FfiResult<*mut AnyTransformation>
    where
        Q: Number,
    {
        fn monomorphize2<M, T>(num_bins: usize, b: usize) -> FfiResult<*mut AnyTransformation>
        where
            T: Integer,
            M: 'static + BAryTreeMetric,
            M::Distance: Number,
        {
            make_b_ary_tree::<M, T>(num_bins, b).into_any()
        }

        dispatch!(monomorphize2, [
            (M, [L1Distance<Q>, L2Distance<Q>]),
            (T, @integers)
        ], (num_bins, b))
    }

    let num_bins = num_bins as usize;
    let b = b as usize;
    let M = try_!(Type::try_from(M));
    let T = try_!(Type::try_from(T));
    let Q = try_!(M.get_atom());
    dispatch!(monomorphize, [
        (Q, @integers)
    ], (num_bins, b, M, T))
}
