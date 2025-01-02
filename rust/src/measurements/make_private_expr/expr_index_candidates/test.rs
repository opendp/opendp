use crate::{
    measures::MaxDivergence,
    metrics::{PartitionDistance, SymmetricDistance},
    polars::PrivacyNamespace,
    transformations::expr_discrete_quantile_score::test::get_quantile_test_data,
};

use super::*;
use polars::prelude::*;

#[test]
fn test_index_candidates_udf() -> Fallible<()> {
    let candidates = Series::new(
        "".into(),
        &["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"],
    );
    let selection_indices = Column::new("".into(), &[0u32, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    let selections = index_candidates_udf(
        &[selection_indices],
        IndexCandidatesPlugin {
            candidates: candidates.clone(),
        },
    )?;

    assert_eq!(selections, candidates.into_column());
    Ok(())
}

#[test]
fn test_index_candidates_expr() -> Fallible<()> {
    let (lf_domain, lf) = get_quantile_test_data()?;
    let candidates = Series::new(
        "".into(),
        [0., 10., 20., 30., 40., 50., 60., 70., 80., 90., 100.],
    );
    let scale: f64 = 1e-8;

    let m_quant = col("cycle_(..101f64)")
        .dp()
        .quantile(0.80, candidates, Some(scale))
        .make_private(
            lf_domain.select(),
            PartitionDistance(SymmetricDistance),
            MaxDivergence::default(),
            None,
        )?;

    let dp_expr = m_quant.invoke(&lf.logical_plan)?.expr;
    let df = lf.select([dp_expr]).collect()?;
    let actual = df.column("cycle_(..101f64)")?.f64()?.get(0).unwrap();
    assert_eq!(actual, 80.);

    Ok(())
}

#[test]
#[cfg(feature = "ffi")]
fn test_index_candidates_serde() -> Fallible<()> {
    macro_rules! test_roundtrip {
        ($args:expr) => {{
            let ic_args = IndexCandidatesPlugin {
                candidates: $args.clone(),
            };
            let serialized = serde_pickle::to_vec(&ic_args, Default::default()).unwrap();
            let deserialized: IndexCandidatesPlugin =
                serde_pickle::from_slice(&serialized, Default::default()).unwrap();
            assert_eq!($args, deserialized.candidates);
        }};
    }

    test_roundtrip!(Series::new("".into(), &[true, false]));
    test_roundtrip!(Series::new("".into(), &[1i64, 2, 3]));
    test_roundtrip!(Series::new("".into(), &[1.0, 2.0, 3.0]));
    test_roundtrip!(Series::new("".into(), &["a", "b", "c"]));
    Ok(())
}
