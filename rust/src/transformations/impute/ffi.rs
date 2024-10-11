use rand::distributions::uniform::SampleUniform;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace};
use crate::domains::{AtomDomain, OptionDomain, VectorDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::{Type, TypeContents};
use crate::traits::{CheckAtom, Float, InherentNull};
use crate::transformations::{
    make_drop_null, make_impute_constant, make_impute_uniform_float, DatasetMetric,
    ImputeConstantDomain,
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_impute_uniform_float(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    bounds: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let bounds = try_as_ref!(bounds);
    let M_ = input_metric.type_.clone();
    let TA_ = try_!(input_domain.type_.get_atom());

    fn monomorphize<M, TA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        bounds: &AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        TA: Float + SampleUniform,
        M: 'static + DatasetMetric,
        (VectorDomain<AtomDomain<TA>>, M): MetricSpace,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<M>()?.clone();
        let bounds = *try_as_ref!(bounds).downcast_ref::<(TA, TA)>()?;
        make_impute_uniform_float(input_domain, input_metric, bounds).into_any()
    }
    dispatch!(monomorphize, [(M_, @dataset_metrics), (TA_, @floats)], (input_domain, input_metric, bounds)).into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_impute_constant(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    constant: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let constant = try_as_ref!(constant);

    let DI = input_domain.type_.clone();
    let DIA = if let TypeContents::GENERIC {
        name: "VectorDomain",
        args,
    } = DI.contents
    {
        try_!(Type::of_id(try_!(args.get(0).ok_or_else(|| err!(
            FFI,
            "Vec must have one type argument."
        )))))
    } else {
        return err!(
            FFI,
            "Invalid type name. Expected VectorDomain, found {}",
            DI.to_string()
        )
        .into();
    };

    let TA = try_!(DIA.get_atom());
    let M = input_metric.type_.clone();

    match &DIA.contents {
        TypeContents::GENERIC { name, .. } if name == &"OptionDomain" => {
            fn monomorphize<M, TA>(
                input_domain: &AnyDomain,
                input_metric: &AnyMetric,
                constant: &AnyObject,
            ) -> Fallible<AnyTransformation>
            where
                OptionDomain<AtomDomain<TA>>: ImputeConstantDomain<Imputed = TA>,
                TA: 'static + Clone + CheckAtom,
                M: 'static + DatasetMetric,
                (VectorDomain<OptionDomain<AtomDomain<TA>>>, M): MetricSpace,
                (VectorDomain<AtomDomain<TA>>, M): MetricSpace,
            {
                let input_domain = input_domain
                    .downcast_ref::<VectorDomain<OptionDomain<AtomDomain<TA>>>>()?
                    .clone();
                let input_metric = input_metric.downcast_ref::<M>()?.clone();
                let constant: TA = constant.downcast_ref::<TA>()?.clone();

                make_impute_constant(input_domain, input_metric, constant).into_any()
            }
            dispatch!(monomorphize, [(M, @dataset_metrics), (TA, @primitives)], (input_domain, input_metric, constant)).into()
        }
        TypeContents::GENERIC { name, .. } if name == &"AtomDomain" => {
            fn monomorphize<M, TA>(
                input_domain: &AnyDomain,
                input_metric: &AnyMetric,
                constant: &AnyObject,
            ) -> Fallible<AnyTransformation>
            where
                AtomDomain<TA>: ImputeConstantDomain<Imputed = TA>,
                TA: 'static + InherentNull + Clone + CheckAtom,
                M: 'static + DatasetMetric,
                (VectorDomain<AtomDomain<TA>>, M): MetricSpace,
            {
                let input_domain = input_domain
                    .downcast_ref::<VectorDomain<AtomDomain<TA>>>()?
                    .clone();
                let input_metric = input_metric.downcast_ref::<M>()?.clone();
                let constant: TA = constant.downcast_ref::<TA>()?.clone();
                make_impute_constant(input_domain, input_metric, constant).into_any()
            }
            dispatch!(monomorphize,
                [(M, @dataset_metrics), (TA, [f64, f32])],
                (input_domain, input_metric, constant)
            )
            .into()
        }
        _ => err!(
            TypeParse,
            "DA must be an OptionDomain<AtomDomain<T>> or an AtomDomain<T>"
        )
        .into(),
    }
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_drop_null(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);

    let DI = input_domain.type_.clone();
    let DIA = if let TypeContents::GENERIC {
        name: "VectorDomain",
        args,
    } = DI.contents
    {
        try_!(Type::of_id(try_!(args.get(0).ok_or_else(|| err!(
            FFI,
            "Vec must have one type argument."
        )))))
    } else {
        return err!(
            FFI,
            "Invalid input domain. Expected VectorDomain, found {}",
            DI.to_string()
        )
        .into();
    };

    let TA = try_!(DIA.get_atom());
    let M = input_metric.type_.clone();

    match &DIA.contents {
        TypeContents::GENERIC { name, .. } if name == &"OptionDomain" => {
            fn monomorphize<M, TA>(
                input_domain: &AnyDomain,
                input_metric: &AnyMetric,
            ) -> Fallible<AnyTransformation>
            where
                OptionDomain<AtomDomain<TA>>: ImputeConstantDomain<Imputed = TA>,
                TA: 'static + Clone + CheckAtom,
                M: 'static + DatasetMetric,
                (VectorDomain<OptionDomain<AtomDomain<TA>>>, M): MetricSpace,
                (VectorDomain<AtomDomain<TA>>, M): MetricSpace,
            {
                let input_domain = input_domain
                    .downcast_ref::<VectorDomain<OptionDomain<AtomDomain<TA>>>>()?
                    .clone();
                let input_metric = input_metric.downcast_ref::<M>()?.clone();

                make_drop_null(input_domain, input_metric).into_any()
            }
            dispatch!(monomorphize, [(M, @dataset_metrics), (TA, @primitives)], (input_domain, input_metric)).into()
        }
        TypeContents::GENERIC { name, .. } if name == &"AtomDomain" => {
            fn monomorphize<M, TA>(
                input_domain: &AnyDomain,
                input_metric: &AnyMetric,
            ) -> Fallible<AnyTransformation>
            where
                AtomDomain<TA>: ImputeConstantDomain<Imputed = TA>,
                TA: 'static + InherentNull + Clone + CheckAtom,
                M: 'static + DatasetMetric,
                (VectorDomain<AtomDomain<TA>>, M): MetricSpace,
            {
                let input_domain = input_domain
                    .downcast_ref::<VectorDomain<AtomDomain<TA>>>()?
                    .clone();
                let input_metric = input_metric.downcast_ref::<M>()?.clone();
                make_drop_null(input_domain, input_metric).into_any()
            }
            dispatch!(monomorphize,
                [(M, @dataset_metrics), (TA, [f64, f32])],
                (input_domain, input_metric)
            )
            .into()
        }
        _ => err!(
            TypeParse,
            "DA must be an OptionDomain<AtomDomain<T>> or an AtomDomain<T>"
        )
        .into(),
    }
}
