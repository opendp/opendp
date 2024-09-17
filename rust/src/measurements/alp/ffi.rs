use std::ffi::c_void;

use num::ToPrimitive;

use crate::{
    core::{FfiResult, MetricSpace},
    domains::{AtomDomain, MapDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast},
        util::{self, Type, TypeContents},
    },
    metrics::L1Distance,
    traits::{DistanceConstant, Hashable, InfCast, Integer},
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_alp_queryable(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: f64,
    total_limit: *const c_void,
    value_limit: *const c_void,
    size_factor: *const u32,
    alpha: *const u32,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<K, CI>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: f64,
        total_limit: *const c_void,
        value_limit: *const c_void,
        size_factor: Option<u32>,
        alpha: Option<u32>,
    ) -> Fallible<AnyMeasurement>
    where
        K: 'static + Hashable,
        CI: 'static + Integer + DistanceConstant<CI> + InfCast<f64> + ToPrimitive,
        f64: InfCast<CI>,
        (MapDomain<AtomDomain<K>, AtomDomain<CI>>, L1Distance<CI>): MetricSpace,
    {
        let input_domain =
            try_!(input_domain.downcast_ref::<MapDomain<AtomDomain<K>, AtomDomain<CI>>>());
        let input_metric = try_!(input_metric.downcast_ref::<L1Distance<CI>>());
        let total_limit = try_as_ref!(total_limit as *const CI).clone();
        let value_limit = util::as_ref(value_limit as *const CI).cloned();

        Ok(super::make_alp_queryable(
            input_domain.clone(),
            input_metric.clone(),
            scale,
            total_limit,
            value_limit,
            size_factor,
            alpha,
        )?
        .into_any_Q()
        .into_any_A()
        .into_any())
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let size_factor = util::as_ref(size_factor).cloned();
    let alpha = util::as_ref(alpha).cloned();

    let TypeContents::GENERIC { name, args } = &input_domain.carrier_type.contents else {
        return err!(
            FFI,
            "Expected input domain to be MapDomain, found {}",
            input_domain.type_.to_string()
        )
        .into();
    };

    if name != &"HashMap" {
        return err!(
            FFI,
            "Expected input domain to be MapDomain, found {}",
            input_domain.type_.to_string()
        )
        .into();
    }

    let K = try_!(try_!(Type::of_id(&args[0])).get_atom());
    let CI = try_!(try_!(Type::of_id(&args[1])).get_atom());

    dispatch!(monomorphize, [
        (K, @hashable),
        (CI, @integers)
    ], (input_domain, input_metric, scale, total_limit, value_limit, size_factor, alpha))
    .into()
}
