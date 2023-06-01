#[cfg(feature = "ffi")]
mod ffi;

use opendp_derive::bootstrap;

use crate::core::{MetricSpace, Transformation};
use crate::domains::{AtomDomain, Bounds, VectorDomain};
use crate::error::Fallible;
use crate::traits::{CheckAtom, TotalOrd};
use crate::transformations::make_row_by_row_fallible;

use super::DatasetMetric;

#[bootstrap(
    features("contrib"),
    arguments(
        input_domain(c_type = "AnyDomain *", hint = "Domain"),
        input_metric(c_type = "AnyMetric *", hint = "Metric", rust_type = b"null")
    ),
    generics(TA(suppress), M(suppress)),
    derived_types(TA = "$get_atom(get_type(input_domain))")
)]
/// Make a Transformation that clamps numeric data in `Vec<TA>` to `bounds`.
///
/// If datum is less than lower, let datum be lower.
/// If datum is greater than upper, let datum be upper.
///
/// # Arguments
/// * `input_domain` - Domain of input data.
/// * `input_metric` - Metric on input domain.
/// * `bounds` - Tuple of inclusive lower and upper bounds.
///
/// # Generics
/// * `TA` - Atomic Type
pub fn make_clamp<TA: 'static + Clone + TotalOrd + CheckAtom, M: DatasetMetric>(
    input_domain: VectorDomain<AtomDomain<TA>>,
    input_metric: M,
    bounds: (TA, TA),
) -> Fallible<Transformation<VectorDomain<AtomDomain<TA>>, VectorDomain<AtomDomain<TA>>, M, M>>
where
    (VectorDomain<AtomDomain<TA>>, M): MetricSpace,
{
    input_domain.element_domain.assert_non_null()?;

    let mut output_row_domain = input_domain.element_domain.clone();
    output_row_domain.bounds = Some(Bounds::<TA>::new_closed(bounds.clone())?);

    make_row_by_row_fallible(
        input_domain,
        input_metric,
        output_row_domain,
        move |arg: &TA| arg.clone().total_clamp(bounds.0.clone(), bounds.1.clone()),
    )
}

#[cfg(all(test, feature = "partials"))]
mod tests {
    use crate::{error::Fallible, metrics::SymmetricDistance, transformations::part_clamp};

    use super::*;

    #[test]
    fn test_make_clamp() -> Fallible<()> {
        let input_space = (
            VectorDomain::new(AtomDomain::default()),
            SymmetricDistance::default(),
        );
        let transformation = (input_space >> part_clamp((0, 10)))?;
        let arg = vec![-10, -5, 0, 5, 10, 20];
        let ret = transformation.invoke(&arg)?;
        let expected = vec![0, 0, 0, 5, 10, 10];
        assert_eq!(ret, expected);
        Ok(())
    }
}
