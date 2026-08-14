use std::ffi::c_char;

use crate::core::FfiResult;
use crate::ffi::any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric};
use crate::ffi::util::{Type, into_c_char_p, to_option_str};
use crate::measurements::noise::distribution::{NoiseDistribution, NoiseMetric};
use crate::measurements::noise::distribution::{
    gaussian::ffi::opendp_measurements__make_gaussian,
    laplace::ffi::opendp_measurements__make_laplace,
};
use crate::measures::{MultiDP, PureDP, zCDP};
use crate::metrics::{AbsoluteDistance, L1Distance, L2Distance};
use crate::traits::Number;

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_noise(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    scale: f64,
    k: *const i32,
    distribution: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    let output_type = &try_as_ref!(output_measure).type_;
    let metric_type = &try_as_ref!(input_metric).type_;
    let distribution = try_!(to_option_str(distribution))
        .map(NoiseDistribution::try_from)
        .transpose();
    let distribution = try_!(distribution);

    fn infer_multidp_distribution(metric_type: &Type) -> Option<NoiseDistribution> {
        fn monomorphize<Q: Number>(metric_type: Type) -> Option<NoiseDistribution> {
            fn monomorphize_metric<MI: NoiseMetric>() -> Option<NoiseDistribution> {
                MI::multidp_distribution()
            }

            dispatch!(
                monomorphize_metric,
                [(metric_type, [AbsoluteDistance<Q>, L1Distance<Q>, L2Distance<Q>])]
            )
        }

        let Q = metric_type.get_atom().ok()?;
        dispatch!(monomorphize, [(Q, @numbers)], (metric_type.clone()))
    }

    let run_laplace = || {
        opendp_measurements__make_laplace(
            input_domain,
            input_metric,
            scale,
            k,
            try_!(into_c_char_p(output_type.descriptor.clone())),
        )
    };
    let run_gaussian = || {
        opendp_measurements__make_gaussian(
            input_domain,
            input_metric,
            scale,
            k,
            try_!(into_c_char_p(output_type.descriptor.clone())),
        )
    };

    match (output_type.id, distribution) {
        (id, Some(NoiseDistribution::Laplace))
            if id == Type::of::<PureDP>().id || id == Type::of::<MultiDP>().id =>
        {
            run_laplace()
        }
        (id, Some(NoiseDistribution::Gaussian))
            if id == Type::of::<zCDP>().id || id == Type::of::<MultiDP>().id =>
        {
            run_gaussian()
        }
        (id, Some(NoiseDistribution::Laplace)) if id == Type::of::<zCDP>().id => {
            err!(FFI, "laplace distribution is incompatible with zCDP").into()
        }
        (id, Some(NoiseDistribution::Gaussian)) if id == Type::of::<PureDP>().id => {
            err!(FFI, "gaussian distribution is incompatible with PureDP").into()
        }
        (id, Some(_)) if id == Type::of::<MultiDP>().id => {
            err!(FFI, "distribution must be \"laplace\" or \"gaussian\"").into()
        }
        (id, None) if id == Type::of::<PureDP>().id => run_laplace(),
        (id, None) if id == Type::of::<zCDP>().id => run_gaussian(),
        (id, None) if id == Type::of::<MultiDP>().id => {
            match infer_multidp_distribution(metric_type) {
                Some(NoiseDistribution::Laplace) => run_laplace(),
                Some(NoiseDistribution::Gaussian) => run_gaussian(),
                None => err!(
                    FFI,
                    "distribution is required for MultiDP with an ambiguous metric"
                )
                .into(),
            }
        }
        _ => err!(FFI, "output_measure must be PureDP, zCDP, or MultiDP").into(),
    }
}
