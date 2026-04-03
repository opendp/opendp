use opendp_derive::bootstrap;
use polars::prelude::{Expr, col};

use crate::{
    core::FfiResult,
    error::Fallible,
    ffi::{
        any::{AnyMetric, AnyObject, Downcast},
        util,
    },
    metrics::{
        Bounds, ChangeOneIdDistance, DatabaseIdDistance, FrameDistance, IdSite,
        InsertDeleteDistance, PolarsMetric, SymmetricDistance, SymmetricIdDistance,
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
    identifier: *const AnyObject,
) -> FfiResult<*mut AnyMetric> {
    let metric = SymmetricIdDistance {
        protected_label: "identifier".to_string(),
        id_sites: vec![IdSite {
            label: "identifier".to_string(),
            exprs: vec![try_!(try_as_ref!(identifier).downcast_ref::<Expr>()).clone()],
        }],
    };
    FfiResult::Ok(util::into_raw(AnyMetric::new(metric)))
}

#[bootstrap(name = "_symmetric_id_distance_get_identifier")]
/// Retrieve the identifier of a `SymmetricIdDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___symmetric_id_distance_get_identifier(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyObject> {
    let metric = try_!(try_as_ref!(metric).downcast_ref::<SymmetricIdDistance>());
    let identifier = try_!(try_!(super::unique_id_expr(&metric.active_id_sites()))
        .ok_or_else(|| err!(FFI, "metric does not contain exactly one identifier expression")));
    FfiResult::Ok(AnyObject::new_raw(identifier))
}

#[bootstrap(
    name = "change_one_id_distance",
    arguments(identifier(rust_type = "Expr")),
    returns(c_type = "FfiResult<AnyMetric *>", hint = "ChangeOneIdDistance")
)]
/// Construct an instance of the `ChangeOneIdDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__change_one_id_distance(
    identifier: *const AnyObject,
) -> FfiResult<*mut AnyMetric> {
    let metric = ChangeOneIdDistance {
        protected_label: "identifier".to_string(),
        id_sites: vec![IdSite {
            label: "identifier".to_string(),
            exprs: vec![try_!(try_as_ref!(identifier).downcast_ref::<Expr>()).clone()],
        }],
    };
    FfiResult::Ok(util::into_raw(AnyMetric::new(metric)))
}

#[bootstrap(
    name = "id_site",
    arguments(label(rust_type = "String"), exprs(rust_type = "Vec<Expr>")),
    returns(c_type = "FfiResult<AnyObject *>")
)]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__id_site(
    label: *const AnyObject,
    exprs: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let label = try_!(try_as_ref!(label).downcast_ref::<String>()).clone();
    let exprs = try_!(try_as_ref!(exprs).downcast_ref::<Vec<Expr>>()).clone();
    FfiResult::Ok(AnyObject::new_raw(IdSite { label, exprs }))
}

#[bootstrap(name = "_id_site_get_label")]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___id_site_get_label(
    id_site: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let id_site = try_!(try_as_ref!(id_site).downcast_ref::<IdSite>());
    FfiResult::Ok(AnyObject::new_raw(id_site.label.clone()))
}

#[bootstrap(name = "_id_site_get_exprs")]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___id_site_get_exprs(
    id_site: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let id_site = try_!(try_as_ref!(id_site).downcast_ref::<IdSite>());
    FfiResult::Ok(AnyObject::new_raw(id_site.exprs.clone()))
}

#[bootstrap(
    name = "database_id_distance",
    arguments(
        protected_label(rust_type = "String"),
        table_to_id_sites(rust_type = "HashMap<String, Vec<IdSite>>")
    ),
    returns(c_type = "FfiResult<AnyMetric *>", hint = "DatabaseIdDistance")
)]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__database_id_distance(
    protected_label: *const AnyObject,
    table_to_id_sites: *const AnyObject,
) -> FfiResult<*mut AnyMetric> {
    let protected_label = try_!(try_as_ref!(protected_label).downcast_ref::<String>()).clone();
    let table_to_id_sites = try_!(try_as_ref!(table_to_id_sites)
        .downcast_ref::<std::collections::HashMap<String, Vec<IdSite>>>())
    .clone();
    FfiResult::Ok(util::into_raw(AnyMetric::new(DatabaseIdDistance {
        protected_label,
        id_sites: table_to_id_sites,
    })))
}

#[bootstrap(
    name = "database_id_distance_from_exprs",
    arguments(
        protected_label(rust_type = "String"),
        table_to_exprs(rust_type = "HashMap<String, Vec<Expr>>")
    ),
    returns(c_type = "FfiResult<AnyMetric *>", hint = "DatabaseIdDistance")
)]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__database_id_distance_from_exprs(
    protected_label: *const AnyObject,
    table_to_exprs: *const AnyObject,
) -> FfiResult<*mut AnyMetric> {
    let protected_label = try_!(try_as_ref!(protected_label).downcast_ref::<String>()).clone();
    let table_to_exprs = try_!(try_as_ref!(table_to_exprs)
        .downcast_ref::<std::collections::HashMap<String, Vec<Expr>>>())
    .clone();

    let id_sites = table_to_exprs
        .into_iter()
        .map(|(table_name, exprs)| {
            (
                table_name,
                vec![IdSite {
                    label: protected_label.clone(),
                    exprs,
                }],
            )
        })
        .collect();

    FfiResult::Ok(util::into_raw(AnyMetric::new(DatabaseIdDistance {
        protected_label,
        id_sites,
    })))
}

#[bootstrap(
    name = "database_id_distance_from_identifiers",
    arguments(
        protected_label(rust_type = "String"),
        table_to_identifier(rust_type = "HashMap<String, Expr>")
    ),
    returns(c_type = "FfiResult<AnyMetric *>", hint = "DatabaseIdDistance")
)]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__database_id_distance_from_identifiers(
    protected_label: *const AnyObject,
    table_to_identifier: *const AnyObject,
) -> FfiResult<*mut AnyMetric> {
    let protected_label = try_!(try_as_ref!(protected_label).downcast_ref::<String>()).clone();
    let table_to_identifier = try_!(try_as_ref!(table_to_identifier)
        .downcast_ref::<std::collections::HashMap<String, Expr>>())
    .clone();

    let id_sites = table_to_identifier
        .into_iter()
        .map(|(table_name, expr)| {
            (
                table_name,
                vec![IdSite {
                    label: protected_label.clone(),
                    exprs: vec![expr],
                }],
            )
        })
        .collect();

    FfiResult::Ok(util::into_raw(AnyMetric::new(DatabaseIdDistance {
        protected_label,
        id_sites,
    })))
}

#[bootstrap(
    name = "database_id_distance_from_columns",
    arguments(
        protected_label(rust_type = "String"),
        table_to_column(rust_type = "HashMap<String, String>")
    ),
    returns(c_type = "FfiResult<AnyMetric *>", hint = "DatabaseIdDistance")
)]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics__database_id_distance_from_columns(
    protected_label: *const AnyObject,
    table_to_column: *const AnyObject,
) -> FfiResult<*mut AnyMetric> {
    let protected_label = try_!(try_as_ref!(protected_label).downcast_ref::<String>()).clone();
    let table_to_column = try_!(try_as_ref!(table_to_column)
        .downcast_ref::<std::collections::HashMap<String, String>>())
    .clone();

    let id_sites = table_to_column
        .into_iter()
        .map(|(table_name, column_name)| {
            (
                table_name,
                vec![IdSite {
                    label: protected_label.clone(),
                    exprs: vec![col(column_name)],
                }],
            )
        })
        .collect();

    FfiResult::Ok(util::into_raw(AnyMetric::new(DatabaseIdDistance {
        protected_label,
        id_sites,
    })))
}

#[bootstrap(name = "_database_id_distance_get_id_sites")]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___database_id_distance_get_id_sites(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyObject> {
    let metric = try_!(try_as_ref!(metric).downcast_ref::<DatabaseIdDistance>());
    FfiResult::Ok(AnyObject::new_raw(metric.id_sites.clone()))
}

#[bootstrap(name = "_database_id_distance_get_protected_label")]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___database_id_distance_get_protected_label(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyObject> {
    let metric = try_!(try_as_ref!(metric).downcast_ref::<DatabaseIdDistance>());
    FfiResult::Ok(AnyObject::new_raw(metric.protected_label.clone()))
}

#[bootstrap(name = "_database_id_distance_get_identifiers")]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___database_id_distance_get_identifiers(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyObject> {
    let metric = try_!(try_as_ref!(metric).downcast_ref::<DatabaseIdDistance>());
    let identifiers = metric
        .id_sites
        .iter()
        .map(|(table_name, id_sites)| {
            (
                table_name.clone(),
                super::filter_id_sites(id_sites, &metric.protected_label)
                    .into_iter()
                    .flat_map(|site| site.exprs)
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<std::collections::HashMap<_, _>>();
    FfiResult::Ok(AnyObject::new_raw(identifiers))
}

#[bootstrap(name = "_database_id_distance_get_columns")]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___database_id_distance_get_columns(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyObject> {
    let metric = try_!(try_as_ref!(metric).downcast_ref::<DatabaseIdDistance>());
    let columns = metric
        .id_sites
        .iter()
        .map(|(table_name, id_sites)| {
            let identifier = try_!(super::unique_id_expr(&super::filter_id_sites(
                id_sites,
                &metric.protected_label,
            )))
            .ok_or_else(|| {
                err!(
                    FFI,
                    "table {} does not contain exactly one protected identifier expression",
                    table_name
                )
            })?;
            let root_names = identifier.meta().root_names();
            let [root_name] = root_names.as_slice() else {
                return fallible!(
                    FFI,
                    "table {} protected identifier must be a single root column",
                    table_name
                );
            };
            Ok((table_name.clone(), root_name.to_string()))
        })
        .collect::<Fallible<std::collections::HashMap<_, _>>>();
    FfiResult::Ok(AnyObject::new_raw(try_!(columns)))
}

#[bootstrap(name = "_change_one_id_distance_get_identifier")]
/// Retrieve the identifier of a `ChangeOneIdDistance` metric.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_metrics___change_one_id_distance_get_identifier(
    metric: *const AnyMetric,
) -> FfiResult<*mut AnyObject> {
    let metric = try_!(try_as_ref!(metric).downcast_ref::<ChangeOneIdDistance>());
    let identifier = try_!(try_!(super::unique_id_expr(&metric.active_id_sites()))
        .ok_or_else(|| err!(FFI, "metric does not contain exactly one identifier expression")));
    FfiResult::Ok(AnyObject::new_raw(identifier))
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
