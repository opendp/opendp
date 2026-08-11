use opendp_derive::bootstrap;
use polars::{
    lazy::frame::LazyFrame,
    prelude::{Expr, NamedFrom, Null, Series, lit},
};
use std::{
    ffi::{c_char, c_int, c_void},
    slice,
};

use crate::{
    core::{FfiResult, FfiSlice},
    domains::{Context, LazyFrameDomain, Margin, SeriesDomain, WildExprDomain},
    ffi::any::{AnyDomain, AnyObject, AnyQueryable, Downcast},
    ffi::util,
    measurements::{
        expr_dp_counting_query::{DPCountShim, DPNUniqueShim, DPNullCountShim},
        expr_dp_frame_len::DPFrameLenShim,
        expr_dp_mean::DPMeanShim,
        expr_dp_median::DPMedianShim,
        expr_dp_quantile::DPQuantileShim,
        expr_dp_sum::DPSumShim,
        expr_noise::NoiseShim,
    },
    polars::{ExtractLazyFrame, OnceFrameAnswer, OnceFrameQuery, OpenDPPlugin},
};

/// Construct a LazyFrameDomain from a C slice of SeriesDomain pointers.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__lazyframe_domain_from_slice(
    series_domains: *const FfiSlice,
) -> FfiResult<*mut AnyDomain> {
    let series_domains = try_as_ref!(series_domains);
    let pointers = unsafe {
        slice::from_raw_parts(
            series_domains.ptr as *const *const AnyDomain,
            series_domains.len,
        )
    };
    let series_domains = pointers
        .iter()
        .map(|domain| try_as_ref!(*domain).downcast_ref::<SeriesDomain>().cloned())
        .collect::<crate::error::Fallible<Vec<_>>>();
    Ok(AnyDomain::new(try_!(LazyFrameDomain::new(try_!(
        series_domains
    )))))
    .into()
}

/// Attach a grouping margin to a LazyFrameDomain using column names supplied
/// by R.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__lazyframe_domain_with_margin_by(
    domain: *const AnyDomain,
    by: *const FfiSlice,
    max_length: *const u32,
    max_groups: *const u32,
    invariant: *const c_char,
) -> FfiResult<*mut AnyDomain> {
    let domain = try_!(try_as_ref!(domain).downcast_ref::<LazyFrameDomain>()).clone();
    let by = try_as_ref!(by);
    let names = unsafe { slice::from_raw_parts(by.ptr as *const *const c_char, by.len) };
    let mut margin = Margin::by(try_!(
        names
            .iter()
            .map(|name| util::to_str(*name).map(polars::prelude::col))
            .collect::<crate::error::Fallible<Vec<_>>>()
    ));
    if let Some(max_length) = util::as_ref(max_length) {
        margin = margin.with_max_length(*max_length);
    }
    if let Some(max_groups) = util::as_ref(max_groups) {
        margin = margin.with_max_groups(*max_groups);
    }
    if !invariant.is_null() {
        margin = match try_!(util::to_str(invariant)) {
            "keys" => margin.with_invariant_keys(),
            "lengths" => margin.with_invariant_lengths(),
            other => {
                return err!(FFI, "invariant must be \"keys\" or \"lengths\", found {:?}", other)
                    .into();
            }
        };
    }
    Ok(AnyDomain::new(try_!(domain.with_margin(margin)))).into()
}

/// Construct a WildExprDomain from a C slice of SeriesDomain pointers.
/// A null `by` gives a row-by-row domain; otherwise `by` (possibly empty)
/// defines an aggregation margin.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__wild_expr_domain_from_slice(
    series_domains: *const FfiSlice,
    by: *const FfiSlice,
    max_length: *const u32,
    max_groups: *const u32,
    invariant: *const c_char,
) -> FfiResult<*mut AnyDomain> {
    let series_domains = try_as_ref!(series_domains);
    let pointers = unsafe {
        slice::from_raw_parts(
            series_domains.ptr as *const *const AnyDomain,
            series_domains.len,
        )
    };
    let columns = try_!(
        pointers
            .iter()
            .map(|domain| try_as_ref!(*domain).downcast_ref::<SeriesDomain>().cloned())
            .collect::<crate::error::Fallible<Vec<_>>>()
    );

    let context = if let Some(by) = util::as_ref(by) {
        let names = unsafe { slice::from_raw_parts(by.ptr as *const *const c_char, by.len) };
        let mut margin = Margin::by(try_!(
            names
                .iter()
                .map(|name| util::to_str(*name).map(polars::prelude::col))
                .collect::<crate::error::Fallible<Vec<_>>>()
        ));
        if let Some(max_length) = util::as_ref(max_length) {
            margin = margin.with_max_length(*max_length);
        }
        if let Some(max_groups) = util::as_ref(max_groups) {
            margin = margin.with_max_groups(*max_groups);
        }
        if !invariant.is_null() {
            margin = match try_!(util::to_str(invariant)) {
                "keys" => margin.with_invariant_keys(),
                "lengths" => margin.with_invariant_lengths(),
                other => {
                    return err!(FFI, "invariant must be \"keys\" or \"lengths\", found {:?}", other)
                        .into();
                }
            };
        }
        Context::Aggregation { margin }
    } else {
        Context::RowByRow
    };

    Ok(AnyDomain::new(WildExprDomain { columns, context })).into()
}

fn deserialize_r_expr(raw: &FfiSlice) -> crate::error::Fallible<Expr> {
    let input = unsafe { slice::from_raw_parts(raw.ptr as *const u8, raw.len) };
    ciborium::from_reader(input)
        .map_err(|err| err!(FFI, "Error when deserializing r-polars 'Expr': {}", err))
}

fn serialize_r_expr(expr: &Expr) -> crate::error::Fallible<FfiSlice> {
    let mut buffer = Vec::new();
    ciborium::into_writer(expr, &mut buffer)
        .map_err(|err| err!(FFI, "failed to serialize r-polars Expr: {}", err))?;
    let output = FfiSlice {
        ptr: buffer.as_ptr() as *mut c_void,
        len: buffer.len(),
    };
    util::into_raw(buffer);
    Ok(output)
}

/// Round-trip an expression serialized by r-polars.
///
/// Unlike Python Polars, which uses Polars' `SerializeOptions`, r-polars
/// serializes expressions directly as CBOR with `ciborium`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__roundtrip_r_polars_expr(
    raw: *const FfiSlice,
) -> FfiResult<*mut FfiSlice> {
    let raw = try_as_ref!(raw);
    serialize_r_expr(&try_!(deserialize_r_expr(raw))).into()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__make_r_polars_dp_len(
    scale: *const f64,
    signed: c_int,
    lib: *const c_char,
) -> FfiResult<*mut FfiSlice> {
    let scale = util::as_ref(scale)
        .copied()
        .map(lit)
        .unwrap_or_else(|| lit(Null {}));
    let lib = try_!(util::to_str(lib));
    let expr =
        crate::polars::apply_ffi_plugin(vec![scale, lit(signed != 0)], DPFrameLenShim, lib);
    serialize_r_expr(&expr).into()
}

/// Build an `[input, scale]` OpenDP plugin expression.
fn unary_scale_plugin<KW: OpenDPPlugin>(
    raw: *const FfiSlice,
    scale: *const f64,
    lib: *const c_char,
    kwargs: KW,
) -> FfiResult<*mut FfiSlice> {
    let input = try_!(deserialize_r_expr(try_as_ref!(raw)));
    let scale = util::as_ref(scale)
        .copied()
        .map(lit)
        .unwrap_or_else(|| lit(Null {}));
    let lib = try_!(util::to_str(lib));
    let expr = crate::polars::apply_ffi_plugin(vec![input, scale], kwargs, lib);
    serialize_r_expr(&expr).into()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__make_r_polars_dp_noise(
    raw: *const FfiSlice,
    scale: *const f64,
    lib: *const c_char,
) -> FfiResult<*mut FfiSlice> {
    unary_scale_plugin(raw, scale, lib, NoiseShim)
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__make_r_polars_dp_count(
    raw: *const FfiSlice,
    scale: *const f64,
    lib: *const c_char,
) -> FfiResult<*mut FfiSlice> {
    unary_scale_plugin(raw, scale, lib, DPCountShim)
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__make_r_polars_dp_null_count(
    raw: *const FfiSlice,
    scale: *const f64,
    lib: *const c_char,
) -> FfiResult<*mut FfiSlice> {
    unary_scale_plugin(raw, scale, lib, DPNullCountShim)
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__make_r_polars_dp_n_unique(
    raw: *const FfiSlice,
    scale: *const f64,
    lib: *const c_char,
) -> FfiResult<*mut FfiSlice> {
    unary_scale_plugin(raw, scale, lib, DPNUniqueShim)
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__make_r_polars_dp_sum(
    raw: *const FfiSlice,
    lower: f64,
    upper: f64,
    scale: *const f64,
    lib: *const c_char,
) -> FfiResult<*mut FfiSlice> {
    if lower > upper {
        return err!(FFI, "lower bound must not exceed upper bound").into();
    }
    let input = try_!(deserialize_r_expr(try_as_ref!(raw)));
    let scale = util::as_ref(scale)
        .copied()
        .map(lit)
        .unwrap_or_else(|| lit(Null {}));
    let lib = try_!(util::to_str(lib));
    let expr =
        crate::polars::apply_ffi_plugin(vec![input, lit(lower), lit(upper), scale], DPSumShim, lib);
    serialize_r_expr(&expr).into()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__make_r_polars_dp_mean(
    raw: *const FfiSlice,
    lower: f64,
    upper: f64,
    scale: *const f64,
    lib: *const c_char,
) -> FfiResult<*mut FfiSlice> {
    if lower > upper {
        return err!(FFI, "lower bound must not exceed upper bound").into();
    }
    let input = try_!(deserialize_r_expr(try_as_ref!(raw)));
    let scale = util::as_ref(scale)
        .copied()
        .map(lit)
        .unwrap_or_else(|| lit(Null {}));
    let lib = try_!(util::to_str(lib));
    let expr = crate::polars::apply_ffi_plugin(
        vec![input, lit(lower), lit(upper), scale],
        DPMeanShim,
        lib,
    );
    serialize_r_expr(&expr).into()
}

/// Build a candidate `Series` literal from a slice of doubles supplied by R.
fn r_polars_candidates(
    candidates: *const f64,
    candidates_len: usize,
) -> crate::error::Fallible<Series> {
    if candidates.is_null() || candidates_len == 0 {
        return fallible!(FFI, "candidates must be non-empty");
    }
    let values = unsafe { slice::from_raw_parts(candidates, candidates_len) };
    Ok(Series::new("".into(), values))
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__make_r_polars_dp_median(
    raw: *const FfiSlice,
    candidates: *const f64,
    candidates_len: usize,
    scale: *const f64,
    lib: *const c_char,
) -> FfiResult<*mut FfiSlice> {
    let input = try_!(deserialize_r_expr(try_as_ref!(raw)));
    let candidates = try_!(r_polars_candidates(candidates, candidates_len));
    let scale = util::as_ref(scale)
        .copied()
        .map(lit)
        .unwrap_or_else(|| lit(Null {}));
    let lib = try_!(util::to_str(lib));
    let expr = crate::polars::apply_ffi_plugin(
        vec![input, lit(candidates), scale],
        DPMedianShim,
        lib,
    );
    serialize_r_expr(&expr).into()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__make_r_polars_dp_quantile(
    raw: *const FfiSlice,
    alpha: f64,
    candidates: *const f64,
    candidates_len: usize,
    scale: *const f64,
    lib: *const c_char,
) -> FfiResult<*mut FfiSlice> {
    if !(0.0..=1.0).contains(&alpha) {
        return err!(FFI, "alpha must be between 0 and 1").into();
    }
    let input = try_!(deserialize_r_expr(try_as_ref!(raw)));
    let candidates = try_!(r_polars_candidates(candidates, candidates_len));
    let scale = util::as_ref(scale)
        .copied()
        .map(lit)
        .unwrap_or_else(|| lit(Null {}));
    let lib = try_!(util::to_str(lib));
    let expr = crate::polars::apply_ffi_plugin(
        vec![input, lit(alpha), lit(candidates), scale],
        DPQuantileShim,
        lib,
    );
    serialize_r_expr(&expr).into()
}

#[bootstrap(
    name = "onceframe_collect",
    arguments(onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"))
)]
/// Internal function. Collects a DataFrame from a OnceFrame, exhausting the OnceFrame.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__onceframe_collect(
    onceframe: *mut AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let query = AnyObject::new(OnceFrameQuery::Collect);
    let answer: OnceFrameAnswer = try_!(try_!(queryable.eval(&query)).downcast());
    let OnceFrameAnswer::Collect(frame) = answer;

    Ok(AnyObject::new(frame)).into()
}

/// Collect a OnceFrame exactly once and serialize the released DataFrame in
/// r-polars' binary DataFrame format.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__onceframe_collect_r_polars(
    onceframe: *mut AnyObject,
) -> FfiResult<*mut FfiSlice> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let query = AnyObject::new(OnceFrameQuery::Collect);
    let answer: OnceFrameAnswer = try_!(try_!(queryable.eval(&query)).downcast());
    let OnceFrameAnswer::Collect(mut frame) = answer;
    let buffer = try_!(frame.serialize_to_bytes().map_err(|err| err!(
        FFI,
        "failed to serialize released DataFrame: {}",
        err
    )));
    let output = FfiSlice {
        ptr: buffer.as_ptr() as *mut c_void,
        len: buffer.len(),
    };
    util::into_raw(buffer);
    Ok(output).into()
}

#[bootstrap(
    features("honest-but-curious"),
    name = "onceframe_lazy",
    arguments(onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"))
)]
/// Internal function. Extracts a LazyFrame from a OnceFrame,
/// circumventing protections against multiple evaluations.
///
/// Each collection consumes the entire allocated privacy budget.
/// To remain DP at the advertised privacy level, only collect the LazyFrame once.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
///
/// # Why honest-but-curious?
/// The privacy guarantees only apply if:
///
/// 1. The LazyFrame (compute plan) is only ever executed once.
/// 2. The analyst does not observe ordering of rows in the output.
///    
/// To ensure that row ordering is not observed:
///
/// 1. Do not extend the compute plan with order-sensitive computations.
/// 2. Shuffle the output once collected ([in Polars sample all, with shuffling enabled](https://docs.pola.rs/api/python/stable/reference/dataframe/api/polars.DataFrame.sample.html)).
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__onceframe_lazy(
    onceframe: *mut AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let answer: LazyFrame = try_!(queryable.eval_internal(&ExtractLazyFrame));
    Ok(AnyObject::new(answer)).into()
}
