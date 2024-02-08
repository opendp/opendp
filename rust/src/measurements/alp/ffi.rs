use std::ffi::{c_char, c_uint, c_void};

use dashu::float::FBig;
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
    traits::{DistanceConstant, Float, Hashable, InfCast, Integer},
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_alp_queryable(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: *const c_void,
    total_limit: *const c_void,
    value_limit: *const c_void,
    size_factor: *const c_uint,
    alpha: *const c_void,
    CO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<K, CI, CO>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: *const c_void,
        total_limit: *const c_void,
        value_limit: *const c_void,
        size_factor: Option<u32>,
        alpha: *const c_void,
    ) -> Fallible<AnyMeasurement>
    where
        K: 'static + Hashable,
        CI: 'static + Integer + DistanceConstant<CI> + InfCast<CO> + ToPrimitive,
        CO: 'static + Float + DistanceConstant<CO> + InfCast<FBig> + InfCast<CI>,
        FBig: InfCast<CO>,
        (MapDomain<AtomDomain<K>, AtomDomain<CI>>, L1Distance<CI>): MetricSpace,
    {
        let input_domain =
            try_!(input_domain.downcast_ref::<MapDomain<AtomDomain<K>, AtomDomain<CI>>>());
        let input_metric = try_!(input_metric.downcast_ref::<L1Distance<CI>>());
        let scale = try_as_ref!(scale as *const CO).clone();
        let total_limit = try_as_ref!(total_limit as *const CI).clone();
        let value_limit = util::as_ref(value_limit as *const CI).cloned();
        let alpha = util::as_ref(alpha as *const u32).cloned();

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
    let size_factor = util::as_ref(size_factor as *const u32).cloned();

    let TypeContents::GENERIC { name, args } = &input_domain.carrier_type.contents else {
        return err!(FFI, "Expected generic input domain").into();
    };

    if name != &"HashMap" {
        return err!(FFI, "Expected input domain to be MapDomain").into();
    }

    let K = try_!(try_!(Type::of_id(&args[0])).get_atom());
    let CI = try_!(try_!(Type::of_id(&args[1])).get_atom());
    let CO = try_!(Type::try_from(CO));

    dispatch!(monomorphize, [
        (K, @hashable),
        (CI, @integers),
        (CO, @floats)
    ], (input_domain, input_metric, scale, total_limit, value_limit, size_factor, alpha))
    .into()
}
