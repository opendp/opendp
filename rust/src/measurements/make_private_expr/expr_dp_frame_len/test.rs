use polars::prelude::*;

use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain},
    error::Fallible,
    measurements::make_private_lazyframe,
    measures::MaxDivergence,
    metrics::{FrameDistance, SymmetricDistance},
    polars::dp_len,
};


#[test]
fn test_dp_frame_len_allow_negative() -> Fallible<()>{
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?
        .with_margin(Margin::select().with_max_length(10))?;

    let lf = df!("A" => &[1i32, 2, 3])?.lazy();

    let m = make_private_lazyframe(
        lf_domain,
        FrameDistance(SymmetricDistance),
        MaxDivergence,
        lf.clone().select(&[dp_len(Some(0.), Some(true))]),
        Some(1.),
        None,
    )?;

    let result = m.invoke(&lf)?.collect()?;

    let col = result.column("len")?.i64()?;
    assert_eq!(col.get(0), Some(3i64));
    Ok(())
}