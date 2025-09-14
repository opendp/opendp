use std::ffi::c_char;

use chrono::{NaiveDate, NaiveTime};
use opendp_derive::bootstrap;
use polars::prelude::DataType;

use crate::{
    core::{Domain, FfiResult, MetricSpace},
    domains::{
        ArrayDomain, AtomDomain, CategoricalDomain, DatetimeDomain, EnumDomain, OptionDomain,
        PrimitiveDataType,
    },
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, Downcast},
        util::{self, Type},
    },
    metrics::EventLevelMetric,
    traits::CheckAtom,
};

use super::{SeriesDomain, SeriesElementDomain};

#[bootstrap(
    rust_path = "domains/struct.SeriesDomain",
    arguments(element_domain(c_type = "AnyDomain *", rust_type = b"null")),
    generics(DI(suppress)),
    returns(c_type = "FfiResult<AnyDomain *>", hint = "SeriesDomain")
)]
/// Construct an instance of `SeriesDomain`.
///
/// # Arguments
/// * `name` - The name of the series.
/// * `element_domain` - The domain of elements in the series.
fn series_domain<DI: 'static + SeriesElementDomain>(
    name: &str,
    element_domain: DI,
) -> SeriesDomain {
    SeriesDomain::new(name, element_domain)
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains__series_domain(
    name: *mut c_char,
    element_domain: *const AnyDomain,
) -> FfiResult<*mut AnyDomain> {
    let name = try_!(util::to_str(name));
    let element_domain = try_as_ref!(element_domain);
    let DA = element_domain.type_.clone();
    let T = try_!(DA.get_atom());

    macro_rules! handle_type {
        ($type:ty) => {
            if DA == Type::of::<$type>() {
                let element_domain = try_!(element_domain.downcast_ref::<$type>()).clone();
                return Ok(AnyDomain::new(series_domain(name, element_domain))).into();
            }
        };
    }

    if DA.descriptor.starts_with("OptionDomain") {
        handle_type!(OptionDomain<CategoricalDomain>);
        handle_type!(OptionDomain<EnumDomain>);
        handle_type!(OptionDomain<ArrayDomain>);
        handle_type!(OptionDomain<DatetimeDomain>);

        fn monomorphize_option<T: 'static + CheckAtom + PrimitiveDataType>(
            name: &str,
            element_domain: &AnyDomain,
        ) -> Fallible<AnyDomain> {
            let element_domain = element_domain
                .downcast_ref::<OptionDomain<AtomDomain<T>>>()?
                .clone();
            Ok(AnyDomain::new(series_domain(name, element_domain)))
        }

        dispatch!(
            monomorphize_option,
            // These types are the Polars primitive datatypes.
            // Don't forget to update the corresponding list below.
            [(
                T,
                [
                    u32, u64, i32, i64, f32, f64, bool, String, NaiveDate, NaiveTime
                ]
            )],
            (name, element_domain)
        )
        .into()
    } else {
        handle_type!(CategoricalDomain);
        handle_type!(EnumDomain);
        handle_type!(ArrayDomain);
        handle_type!(DatetimeDomain);

        fn monomorphize_atom<T: 'static + CheckAtom + PrimitiveDataType>(
            name: &str,
            element_domain: &AnyDomain,
        ) -> Fallible<AnyDomain> {
            let element_domain = element_domain.downcast_ref::<AtomDomain<T>>()?.clone();
            Ok(AnyDomain::new(series_domain(name, element_domain)))
        }
        dispatch!(
            monomorphize_atom,
            [(
                T,
                [
                    u32, u64, i32, i64, f32, f64, bool, String, NaiveDate, NaiveTime
                ]
            )],
            (name, element_domain)
        )
        .into()
    }
}

#[bootstrap(
    name = "_series_domain_get_name",
    arguments(series_domain(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// # Arguments
/// * `series_domain` - The series domain from which to retrieve the name of elements
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___series_domain_get_name(
    series_domain: *const AnyDomain,
) -> FfiResult<*mut AnyObject> {
    let series_domain = try_!(try_as_ref!(series_domain).downcast_ref::<SeriesDomain>());
    Ok(AnyObject::new(series_domain.name.to_string())).into()
}

#[bootstrap(
    arguments(series_domain(rust_type = b"null")),
    generics(D(suppress)),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Retrieve the element domain of the series domain.
///
/// # Arguments
/// * `series_domain` - The series domain from which to retrieve the element domain
fn _series_domain_get_element_domain<D: 'static + Domain>(
    series_domain: &SeriesDomain,
) -> Fallible<AnyDomain> {
    let element_domain = series_domain.element_domain::<D>()?;
    Ok(AnyDomain::new(element_domain.clone()))
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___series_domain_get_element_domain(
    series_domain: *const AnyDomain,
) -> FfiResult<*mut AnyDomain> {
    let series_domain = try_!(try_as_ref!(series_domain).downcast_ref::<SeriesDomain>());
    match series_domain.dtype() {
        DataType::Array(_, _) => _series_domain_get_element_domain::<ArrayDomain>(series_domain),
        DataType::Categorical(_, _) => {
            _series_domain_get_element_domain::<CategoricalDomain>(series_domain)
        }
        DataType::Datetime(_, _) => {
            _series_domain_get_element_domain::<DatetimeDomain>(series_domain)
        }
        DataType::Boolean => _series_domain_get_element_domain::<AtomDomain<bool>>(series_domain),
        DataType::UInt8 => _series_domain_get_element_domain::<AtomDomain<u8>>(series_domain),
        DataType::UInt16 => _series_domain_get_element_domain::<AtomDomain<u16>>(series_domain),
        DataType::UInt32 => _series_domain_get_element_domain::<AtomDomain<u32>>(series_domain),
        DataType::UInt64 => _series_domain_get_element_domain::<AtomDomain<u64>>(series_domain),
        DataType::Int8 => _series_domain_get_element_domain::<AtomDomain<i8>>(series_domain),
        DataType::Int16 => _series_domain_get_element_domain::<AtomDomain<i16>>(series_domain),
        DataType::Int32 => _series_domain_get_element_domain::<AtomDomain<i32>>(series_domain),
        DataType::Int64 => _series_domain_get_element_domain::<AtomDomain<i64>>(series_domain),
        DataType::Float32 => _series_domain_get_element_domain::<AtomDomain<f32>>(series_domain),
        DataType::Float64 => _series_domain_get_element_domain::<AtomDomain<f64>>(series_domain),
        DataType::String => _series_domain_get_element_domain::<AtomDomain<String>>(series_domain),
        DataType::Date => _series_domain_get_element_domain::<AtomDomain<NaiveDate>>(series_domain),
        DataType::Time => _series_domain_get_element_domain::<AtomDomain<NaiveTime>>(series_domain),
        dt => fallible!(FFI, "Unrecognized series domain element type: {}", dt),
    }
    .into()
}

#[bootstrap(
    name = "_series_domain_get_nullable",
    arguments(series_domain(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Retrieve whether elements in members of the domain may be null.
///
/// # Arguments
/// * `series_domain` - The series domain from which to check nullability.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_domains___series_domain_get_nullable(
    series_domain: *const AnyDomain,
) -> FfiResult<*mut AnyObject> {
    let series_domain = try_!(try_as_ref!(series_domain).downcast_ref::<SeriesDomain>());
    Ok(AnyObject::new(series_domain.nullable)).into()
}

impl MetricSpace for (SeriesDomain, AnyMetric) {
    fn check_space(&self) -> Fallible<()> {
        let (domain, metric) = self;

        fn monomorphize_dataset<M: 'static + EventLevelMetric>(
            domain: &SeriesDomain,
            metric: &AnyMetric,
        ) -> Fallible<()>
        where
            (SeriesDomain, M): MetricSpace,
        {
            let metric = metric.downcast_ref::<M>()?;
            (domain.clone(), metric.clone()).check_space()
        }
        let M = metric.type_.clone();

        fn in_set<T>() -> Option<()> {
            Some(())
        }

        if let Some(_) = dispatch!(in_set, [(M, @dataset_metrics)]) {
            dispatch!(monomorphize_dataset, [
                (M, @dataset_metrics)
            ], (domain, metric))
        } else {
            fallible!(MetricSpace, "Unsupported metric: {}", M.descriptor)
        }
    }
}
