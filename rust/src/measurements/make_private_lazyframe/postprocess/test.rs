use polars::{df, lazy::frame::IntoLazy};
use polars_plan::dsl::len;

use crate::{
    core::PrivacyNamespaceHelper,
    domains::{AtomDomain, SeriesDomain},
    measurements::make_private_lazyframe,
    measures::MaxDivergence,
    metrics::SymmetricDistance,
};

use super::*;

#[test]
fn test_make_private_lazyframe_sort() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "A",
        AtomDomain::new_closed((0, 1))?,
    )])?;
    let lf = df!("A" => [0, 1, 1])?.lazy();

    let query = lf
        .select([len().dp().laplace(Some(1.))])
        .sort("len", Default::default());

    make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        MaxDivergence::default(),
        query,
        None,
    )?;

    Ok(())
}
