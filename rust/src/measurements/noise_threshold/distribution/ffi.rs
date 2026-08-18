use std::ffi::{c_char, c_void};

use crate::{
    core::FfiResult,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement},
        util::{Type, into_c_char_p, to_option_str},
    },
    measurements::noise_threshold::distribution::{
        gaussian::ffi::opendp_measurements__make_gaussian_threshold,
        laplace::ffi::opendp_measurements__make_laplace_threshold,
    },
    measurements::{NoiseDistribution, NoiseMetric},
    measures::{Approximate, MultiDP, PureDP, zCDP},
    metrics::{AbsoluteDistance, L01InfDistance, L02InfDistance},
    traits::Number,
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_noise_threshold(
    input_domain: *const AnyDomain,
    input_metric: *const crate::ffi::any::AnyMetric,
    output_measure: *const AnyMeasure,
    scale: f64,
    threshold: *const c_void,
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
                [(metric_type, [
                    L01InfDistance<AbsoluteDistance<Q>>,
                    L02InfDistance<AbsoluteDistance<Q>>
                ])]
            )
        }

        let Q = metric_type.get_atom().ok()?;
        dispatch!(monomorphize, [(Q, @numbers)], (metric_type.clone()))
    }

    let run_laplace = || {
        opendp_measurements__make_laplace_threshold(
            input_domain,
            input_metric,
            scale,
            threshold,
            k,
            try_!(into_c_char_p(output_type.descriptor.clone())),
        )
    };
    let run_gaussian = || {
        opendp_measurements__make_gaussian_threshold(
            input_domain,
            input_metric,
            scale,
            threshold,
            k,
            try_!(into_c_char_p(output_type.descriptor.clone())),
        )
    };

    match (output_type.id, distribution) {
        (id, Some(NoiseDistribution::Laplace))
            if id == Type::of::<Approximate<PureDP>>().id
                || id == Type::of::<Approximate<MultiDP>>().id =>
        {
            run_laplace()
        }
        (id, Some(NoiseDistribution::Gaussian))
            if id == Type::of::<Approximate<zCDP>>().id
                || id == Type::of::<Approximate<MultiDP>>().id =>
        {
            run_gaussian()
        }
        (id, Some(NoiseDistribution::Laplace)) if id == Type::of::<Approximate<zCDP>>().id => err!(
            FFI,
            "laplace distribution is incompatible with Approximate<zCDP>"
        )
        .into(),
        (id, Some(NoiseDistribution::Gaussian)) if id == Type::of::<Approximate<PureDP>>().id => {
            err!(
                FFI,
                "gaussian distribution is incompatible with Approximate<PureDP>"
            )
            .into()
        }
        (id, Some(_)) if id == Type::of::<Approximate<MultiDP>>().id => {
            err!(FFI, "distribution must be \"laplace\" or \"gaussian\"").into()
        }
        (id, None) if id == Type::of::<Approximate<PureDP>>().id => run_laplace(),
        (id, None) if id == Type::of::<Approximate<zCDP>>().id => run_gaussian(),
        (id, None) if id == Type::of::<Approximate<MultiDP>>().id => {
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
        _ => err!(
            FFI,
            "output_measure must be Approximate<PureDP>, Approximate<zCDP>, or Approximate<MultiDP>"
        )
        .into(),
    }
}
