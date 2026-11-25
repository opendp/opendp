use std::{any::TypeId, collections::HashSet, ffi::c_char};

use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Metric, MetricSpace},
    domains::{Margin, SeriesDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, Downcast},
        util::{self, AnyDomainPtr, Type},
    },
    metrics::{
        ChangeOneDistance, ChangeOneIdDistance, FrameDistance, SymmetricDistance,
        SymmetricIdDistance,
    },
};

use super::{DataFrameDomain, Frame, FrameDomain, LazyFrameDomain};
use polars::prelude::*;

#[bootstrap(
    name = "lazyframe_domain",
    arguments(series_domains(rust_type = "Vec<SeriesDomain>")),
    returns(c_type = "FfiResult<AnyDomain *>", hint = "LazyFrameDomain")
)]
/// Construct an instance of `LazyFrameDomain`.
///
/// # Arguments
/// * `series_domains` - Domain of each series in the lazyframe.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__lazyframe_domain(
    series_domains: *const AnyObject,
) -> FfiResult<*mut AnyDomain> {
    Ok(AnyDomain::new(try_!(LazyFrameDomain::new(try_!(
        unpack_series_domains(series_domains)
    )))))
    .into()
}

#[bootstrap(
    name = "dataframe_domain",
    arguments(series_domains(rust_type = "Vec<SeriesDomain>")),
    returns(c_type = "FfiResult<AnyDomain *>", hint = "DataFrameDomain")
)]
/// Construct an instance of `DataFrameDomain`.
///
/// # Arguments
/// * `series_domains` - Domain of each series in the dataframe.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__dataframe_domain(
    series_domains: *const AnyObject,
) -> FfiResult<*mut AnyDomain> {
    Ok(AnyDomain::new(try_!(DataFrameDomain::new(try_!(
        unpack_series_domains(series_domains)
    )))))
    .into()
}

#[bootstrap(
    name = "_frame_domain_get_columns",
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Retrieve the column names from an eager or lazy Polars frame domain.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___frame_domain_get_columns(
    frame_domain: *const AnyDomain,
) -> FfiResult<*mut AnyObject> {
    let domain = try_as_ref!(frame_domain);
    let names = if let Ok(frame) = domain.downcast_ref::<DataFrameDomain>() {
        &frame.series_domains
    } else {
        &try_!(domain.downcast_ref::<LazyFrameDomain>()).series_domains
    };
    Ok(AnyObject::new(
        names.iter().map(|s| s.name.to_string()).collect::<Vec<_>>(),
    ))
    .into()
}

#[bootstrap(
    name = "_frame_domain_get_series_domain",
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Retrieve a named series domain from an eager or lazy Polars frame domain.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___frame_domain_get_series_domain(
    frame_domain: *const AnyDomain,
    name: *const c_char,
) -> FfiResult<*mut AnyDomain> {
    let domain = try_as_ref!(frame_domain);
    let name = try_!(util::to_str(name));
    let series = if let Ok(frame) = domain.downcast_ref::<DataFrameDomain>() {
        frame.series_domain(name.into())
    } else {
        try_!(domain.downcast_ref::<LazyFrameDomain>()).series_domain(name.into())
    };
    Ok(AnyDomain::new(try_!(series))).into()
}

#[bootstrap(
    name = "_frame_domain_with_series",
    arguments(
        output_series_domains(rust_type = "Vec<SeriesDomain>"),
        output_carrier(rust_type = b"null")
    ),
    returns(c_type = "FfiResult<AnyDomain *>", hint = "Domain")
)]
/// Build a frame domain with a new carrier and column descriptors.
///
/// Callers must guarantee unchanged columns retain their values.
/// Margins referring to changed columns are conservatively discarded.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___frame_domain_with_series(
    frame_domain: *const AnyDomain,
    output_series_domains: *const AnyObject,
    output_carrier: *const c_char,
) -> FfiResult<*mut AnyDomain> {
    let domain = try_as_ref!(frame_domain);
    let columns = try_!(unpack_series_domains(output_series_domains));
    let output_carrier = try_!(util::to_str(output_carrier));
    let input_carrier = if domain.type_.id == TypeId::of::<DataFrameDomain>() {
        Type::of::<DataFrame>()
    } else if domain.type_.id == TypeId::of::<LazyFrameDomain>() {
        Type::of::<LazyFrame>()
    } else {
        return err!(FFI, "Expected DataFrameDomain or LazyFrameDomain").into();
    };
    let output_carrier = match output_carrier {
        "DataFrame" => Type::of::<DataFrame>(),
        "LazyFrame" => Type::of::<LazyFrame>(),
        _ => return err!(FFI, "Expected output carrier DataFrame or LazyFrame").into(),
    };

    fn monomorphize<FI: 'static + Frame, FO: 'static + Frame>(
        domain: &AnyDomain,
        columns: Vec<SeriesDomain>,
    ) -> Fallible<AnyDomain> {
        let input = domain.downcast_ref::<FrameDomain<FI>>()?;
        if input.series_domains.len() != columns.len()
            || input
                .series_domains
                .iter()
                .zip(columns.iter())
                .any(|(before, after)| before.name != after.name)
        {
            return fallible!(MakeDomain, "Frame domain column names and order must match");
        }

        let changed = input
            .series_domains
            .iter()
            .zip(columns.iter())
            .filter(|(before, after)| before != after)
            .map(|(before, _)| before.name.clone())
            .collect::<HashSet<_>>();
        let margins = input
            .margins
            .iter()
            .filter(|margin| {
                margin.by.iter().all(|expr| {
                    let meta = expr.clone().meta();
                    !meta.has_multiple_outputs()
                        && meta.root_names().iter().all(|name| !changed.contains(name))
                })
            })
            .cloned()
            .collect();

        Ok(AnyDomain::new(FrameDomain::<FO>::new_with_margins(
            columns, margins,
        )?))
    }

    dispatch!(
        monomorphize,
        [
            (input_carrier, [DataFrame, LazyFrame]),
            (output_carrier, [DataFrame, LazyFrame])
        ],
        (domain, columns)
    )
    .into()
}

#[bootstrap(
    name = "_lazyframe_domain_get_margin",
    arguments(by(rust_type = "Vec<Expr>")),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Retrieve the series domain at index 'column`.
///
/// # Arguments
/// * `lazyframe_domain` - Domain to retrieve the SeriesDomain from
/// * `by` - grouping columns
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___lazyframe_domain_get_margin(
    lazyframe_domain: *const AnyDomain,
    by: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let lazyframe_domain = try_!(try_as_ref!(lazyframe_domain).downcast_ref::<LazyFrameDomain>());
    let by = try_!(try_as_ref!(by).downcast_ref::<Vec<Expr>>());
    let margin = lazyframe_domain.get_margin(&HashSet::from_iter(by.iter().cloned()));
    Ok(AnyObject::new(margin)).into()
}

#[bootstrap()]
/// Construct an empty LazyFrame with the same schema as in the LazyFrameDomain.
///
/// This is useful for creating a dummy lazyframe used to write a query plan.
///
/// # Arguments
/// * `domain` - A LazyFrameDomain.
fn _lazyframe_from_domain(domain: LazyFrameDomain) -> Fallible<LazyFrame> {
    Ok(DataFrame::from_rows_and_schema(&[], &domain.schema())?.lazy())
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___lazyframe_from_domain(
    domain: *mut AnyDomain,
) -> FfiResult<*mut AnyObject> {
    let domain = try_!(try_as_ref!(domain).downcast_ref::<LazyFrameDomain>()).clone();
    _lazyframe_from_domain(domain).map(AnyObject::new).into()
}

pub(crate) fn unpack_series_domains(
    series_domains: *const AnyObject,
) -> Fallible<Vec<SeriesDomain>> {
    let vec_any = try_as_ref!(series_domains).downcast_ref::<Vec<AnyDomainPtr>>()?;

    vec_any
        .iter()
        .map(|x| {
            util::as_ref(x.clone())
                .and_then(|ad: &AnyDomain| ad.downcast_ref::<SeriesDomain>().ok())
                .cloned()
        })
        .collect::<Option<Vec<SeriesDomain>>>()
        .ok_or_else(|| err!(FailedCast, "domain downcast failed"))
}

#[bootstrap(
    name = "with_margin",
    arguments(frame_domain(rust_type = b"null"), margin(rust_type = "Margin")),
    returns(c_type = "FfiResult<AnyDomain *>", hint = "Domain")
)]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__with_margin(
    frame_domain: *mut AnyDomain,
    margin: *mut AnyObject,
) -> FfiResult<*mut AnyDomain> {
    let domain = try_as_ref!(frame_domain);
    let margin = try_!(try_as_ref!(margin).downcast_ref::<Margin>()).clone();
    let F_ = match domain.type_.id {
        x if x == TypeId::of::<DataFrameDomain>() => Type::of::<DataFrame>(),
        x if x == TypeId::of::<LazyFrameDomain>() => Type::of::<LazyFrame>(),
        _ => return err!(FFI, "Expected DataFrameDomain or LazyFrameDomain").into(),
    };

    fn monomorphize<F: 'static + Frame>(domain: &AnyDomain, margin: Margin) -> Fallible<AnyDomain> {
        let domain = domain.downcast_ref::<FrameDomain<F>>()?.clone();
        Ok(AnyDomain::new(domain.with_margin(margin)?))
    }

    dispatch!(
        monomorphize,
        [(F_, [DataFrame, LazyFrame])],
        (domain, margin)
    )
    .into()
}

impl<F: 'static + Frame> MetricSpace for (FrameDomain<F>, AnyMetric) {
    fn check_space(&self) -> Fallible<()> {
        let (domain, metric) = self;

        fn monomorphize_dataset<F: Frame, M: 'static + Metric>(
            domain: &FrameDomain<F>,
            metric: &AnyMetric,
        ) -> Fallible<()>
        where
            (FrameDomain<F>, M): MetricSpace,
        {
            let metric = metric.downcast_ref::<M>()?;
            (domain.clone(), metric.clone()).check_space()
        }

        let F = Type::of::<F>();
        let M = metric.type_.clone();

        dispatch!(
            monomorphize_dataset,
            [
                (F, [F]),
                (
                    M,
                    [
                        SymmetricDistance,
                        SymmetricIdDistance,
                        FrameDistance<SymmetricDistance>,
                        FrameDistance<SymmetricIdDistance>,
                        ChangeOneDistance,
                        ChangeOneIdDistance
                    ]
                )
            ],
            (domain, metric)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::{AtomDomain, OptionDomain};

    #[test]
    fn test_frame_ffi_carriers_and_margins() -> Fallible<()> {
        let column = SeriesDomain::new("value", AtomDomain::<i32>::default());
        let series = AnyDomain::new(column.clone());
        let columns = AnyObject::new(vec![&series as *const AnyDomain as AnyDomainPtr]);
        let eager: Fallible<AnyDomain> = opendp_domains__dataframe_domain(&columns).into();
        let eager = eager?;
        let eager = eager.downcast_ref::<DataFrameDomain>()?.clone();
        let eager = eager.with_margin(Margin::by(["value"]).with_max_groups(2))?;
        let eager = AnyDomain::new(eager);

        let converted: Fallible<AnyDomain> =
            opendp_domains___frame_domain_with_series(&eager, &columns, c"LazyFrame".as_ptr())
                .into();
        let lazy = converted?;
        assert_eq!(lazy.downcast_ref::<LazyFrameDomain>()?.margins.len(), 1);
        for domain in [&eager, &lazy] {
            let names: Fallible<AnyObject> =
                opendp_domains___frame_domain_get_columns(domain).into();
            assert_eq!(names?.downcast_ref::<Vec<String>>()?, &["value"]);
            let found: Fallible<AnyDomain> =
                opendp_domains___frame_domain_get_series_domain(domain, c"value".as_ptr()).into();
            assert_eq!(found?.downcast_ref::<SeriesDomain>()?, &column);
        }

        let changed = AnyDomain::new(SeriesDomain::new(
            "value",
            OptionDomain::new(AtomDomain::<i32>::default()),
        ));
        let changed_columns = AnyObject::new(vec![&changed as *const AnyDomain as AnyDomainPtr]);
        let converted: Fallible<AnyDomain> = opendp_domains___frame_domain_with_series(
            &lazy,
            &changed_columns,
            c"DataFrame".as_ptr(),
        )
        .into();
        assert!(
            converted?
                .downcast_ref::<DataFrameDomain>()?
                .margins
                .is_empty()
        );
        Ok(())
    }

    #[test]
    fn test_frame_ffi_rejects_invalid_columns_and_carriers() -> Fallible<()> {
        let domain = AnyDomain::new(DataFrameDomain::new(vec![])?);
        let columns = AnyObject::new(Vec::<AnyDomainPtr>::new());
        let invalid: Fallible<AnyDomain> =
            opendp_domains___frame_domain_with_series(&domain, &columns, c"Series".as_ptr()).into();
        assert!(invalid.is_err());
        let column = AnyDomain::new(SeriesDomain::new("new", AtomDomain::<i32>::default()));
        let columns = AnyObject::new(vec![&column as *const AnyDomain as AnyDomainPtr]);
        let invalid: Fallible<AnyDomain> =
            opendp_domains___frame_domain_with_series(&domain, &columns, c"DataFrame".as_ptr())
                .into();
        assert!(invalid.is_err());
        Ok(())
    }
}
