use std::collections::HashMap;

use opendp_derive::bootstrap;
use polars::prelude::Expr;

use crate::{
    core::FfiResult,
    error::Fallible,
    ffi::{
        any::{AnyMetric, AnyObject, Downcast},
        util,
    },
    metrics::{
        Binding, Bounds, ChangeOneIdDistance, DatabaseIdDistance, FrameDistance,
        InsertDeleteDistance, SymmetricDistance, SymmetricIdDistance,
    },
    transformations::traits::UnboundedMetric,
};

#[bootstrap(
    name = "symmetric_id_distance",
    arguments(identifier(rust_type = "Expr")),
    returns(c_type = "FfiResult<AnyMetric *>", hint = "SymmetricIdDistance")
)]
/// Construct an instance of the `SymmetricIdDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__symmetric_id_distance(
    bindings: *const AnyObject,
    protect: *const AnyObject,
) -> FfiResult<*mut AnyMetric> {
    let metric = SymmetricIdDistance {
        protect: try_!(try_as_ref!(protect).downcast_ref::<String>()).clone(),
        bindings: try_!(try_as_ref!(bindings).downcast_ref::<Vec<Binding>>()).clone(),
    };
    FfiResult::Ok(util::into_raw(AnyMetric::new(metric)))
}

#[bootstrap(name = "_symmetric_id_distance_get_bindings")]
/// Retrieve the bindings of a `SymmetricIdDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___symmetric_id_distance_get_bindings(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyObject> {
    let metric = try_!(try_as_ref!(metric).downcast_ref::<SymmetricIdDistance>());
    FfiResult::Ok(AnyObject::new_raw(metric.bindings.clone()))
}

#[bootstrap(name = "_symmetric_id_distance_get_protect")]
/// Retrieve the protected space of a `SymmetricIdDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___symmetric_id_distance_get_protect(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyObject> {
    let metric = try_!(try_as_ref!(metric).downcast_ref::<SymmetricIdDistance>());
    FfiResult::Ok(AnyObject::new_raw(metric.protect.clone()))
}

#[bootstrap(
    name = "change_one_id_distance",
    arguments(identifier(rust_type = "Expr")),
    returns(c_type = "FfiResult<AnyMetric *>", hint = "ChangeOneIdDistance")
)]
/// Construct an instance of the `ChangeOneIdDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__change_one_id_distance(
    bindings: *const AnyObject,
    protect: *const AnyObject,
) -> FfiResult<*mut AnyMetric> {
    let metric = ChangeOneIdDistance {
        protect: try_!(try_as_ref!(protect).downcast_ref::<String>()).clone(),
        bindings: try_!(try_as_ref!(bindings).downcast_ref::<Vec<Binding>>()).clone(),
    };
    FfiResult::Ok(util::into_raw(AnyMetric::new(metric)))
}

#[bootstrap(name = "_change_one_id_distance_get_protect")]
/// Retrieve the protected space of a `ChangeOneIdDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___change_one_id_distance_get_protect(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyObject> {
    let metric = try_!(try_as_ref!(metric).downcast_ref::<ChangeOneIdDistance>());
    FfiResult::Ok(AnyObject::new_raw(metric.protect.clone()))
}

#[bootstrap(name = "_change_one_id_distance_get_bindings")]
/// Retrieve the bindings of a `ChangeOneIdDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___change_one_id_distance_get_bindings(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyObject> {
    let metric = try_!(try_as_ref!(metric).downcast_ref::<ChangeOneIdDistance>());
    FfiResult::Ok(AnyObject::new_raw(metric.bindings.clone()))
}

#[bootstrap(
    name = "database_id_distance",
    arguments(
        protect(rust_type = "String"),
        bindings(rust_type = "HashMap<String, Vec<Binding>>")
    ),
    returns(c_type = "FfiResult<AnyMetric *>", hint = "DatabaseIdDistance")
)]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__database_id_distance(
    protect: *const AnyObject,
    bindings: *const AnyObject,
) -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(AnyMetric::new(DatabaseIdDistance {
        protect: try_!(try_as_ref!(protect).downcast_ref::<String>()).clone(),
        bindings: try_!(try_as_ref!(bindings).downcast_ref::<HashMap<String, Vec<Binding>>>())
            .clone(),
    })))
}

#[bootstrap(name = "_database_id_distance_get_bindings")]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___database_id_distance_get_bindings(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyObject> {
    let metric = try_!(try_as_ref!(metric).downcast_ref::<DatabaseIdDistance>());
    FfiResult::Ok(AnyObject::new_raw(metric.bindings.clone()))
}

#[bootstrap(name = "_database_id_distance_get_protect")]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___database_id_distance_get_protect(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyObject> {
    let metric = try_!(try_as_ref!(metric).downcast_ref::<DatabaseIdDistance>());
    FfiResult::Ok(AnyObject::new_raw(metric.protect.clone()))
}

#[bootstrap(name = "_bind_get_to")]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___bind_get_to(
    bind: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let bind = try_!(try_as_ref!(bind).downcast_ref::<Binding>());
    FfiResult::Ok(AnyObject::new_raw(bind.space.clone()))
}

#[bootstrap(name = "_bind_get_exprs")]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___bind_get_exprs(
    bind: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let bind = try_!(try_as_ref!(bind).downcast_ref::<Binding>());
    FfiResult::Ok(AnyObject::new_raw(bind.exprs.clone()))
}

#[bootstrap(name = "frame_distance", returns(c_type = "FfiResult<AnyMetric *>"))]
/// `frame_distance` is a higher-order metric that contains multiple distance bounds for different groupings of data.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__frame_distance(
    inner_metric: *const AnyMetric,
) -> FfiResult<*mut AnyMetric> {
    fn monomorphize<M: 'static + UnboundedMetric>(inner_metric: &AnyMetric) -> Fallible<AnyMetric> {
        let inner_metric = inner_metric.downcast_ref::<M>()?.clone();
        Ok(AnyMetric::new(FrameDistance(inner_metric)))
    }
    let inner_metric = try_as_ref!(inner_metric);
    let M = inner_metric.type_.clone();
    dispatch!(
        monomorphize,
        [(
            M,
            [SymmetricDistance, InsertDeleteDistance, SymmetricIdDistance]
        )],
        (inner_metric)
    )
    .into()
}

#[bootstrap(
    name = "_frame_distance_get_inner_metric",
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Retrieve the inner metric of a `FrameDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___frame_distance_get_inner_metric(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyMetric> {
    fn monomorphize<M: UnboundedMetric>(metric: &AnyMetric) -> Fallible<AnyMetric> {
        Ok(AnyMetric::new(
            metric.downcast_ref::<FrameDistance<M>>()?.0.clone(),
        ))
    }
    let metric = try_as_ref!(metric);
    let M = try_!(metric.type_.get_atom());
    dispatch!(
        monomorphize,
        [(
            M,
            [SymmetricDistance, InsertDeleteDistance, SymmetricIdDistance]
        )],
        (metric)
    )
    .into()
}

#[bootstrap(
    name = "_get_bound",
    arguments(bounds(rust_type = "Bounds"), by(rust_type = "Vec<Expr>"),),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Infer a bound when grouping by `by`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___get_bound(
    bounds: *const AnyObject,
    by: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let bounds = try_!(try_as_ref!(bounds).downcast_ref::<Bounds>());
    let by = try_!(try_as_ref!(by).downcast_ref::<Vec<Expr>>());
    Ok(AnyObject::new(
        bounds.get_bound(&by.iter().cloned().collect()),
    ))
    .into()
}
