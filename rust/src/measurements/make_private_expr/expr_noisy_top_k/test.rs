use super::*;
use polars::prelude::*;
use polars_arrow::array::{FixedSizeListArray, UInt32Array};

#[test]
fn test_noisy_top_k_udf_returns_selected_indices() -> Fallible<()> {
    let scores_slice = &[3, 1, 0, 0, 2, 1, 1, 3, 9];
    let dtype = ArrowDataType::FixedSizeList(
        Box::new(ArrowField::new("item".into(), ArrowDataType::UInt32, true)),
        3,
    );
    let scores = FixedSizeListArray::new(
        dtype,
        3,
        Box::new(UInt32Array::from_slice(scores_slice)),
        None,
    );
    let scores = Series::from(ArrayChunked::from(scores)).into_column();

    let actual = super::noisy_top_k_udf(
        &[scores],
        NoisyTopKPlugin {
            replacement: true,
            negate: false,
            scale: 0.0,
            k: 2,
        },
    )?;

    let expected: ListChunked = [vec![0u32, 1], vec![1, 2], vec![2, 1]]
        .into_iter()
        .map(|indices| Some(Series::new("".into(), indices)))
        .collect();
    assert_eq!(actual, expected.into_column());
    Ok(())
}

#[test]
fn test_noisy_top_k_udf_rejects_zero_k() {
    let dtype = ArrowDataType::FixedSizeList(
        Box::new(ArrowField::new("item".into(), ArrowDataType::UInt32, true)),
        3,
    );
    let scores = FixedSizeListArray::new(
        dtype,
        1,
        Box::new(UInt32Array::from_slice(&[1u32, 2, 3])),
        None,
    );
    let scores = Series::from(ArrayChunked::from(scores)).into_column();
    let result = super::noisy_top_k_udf(
        &[scores],
        NoisyTopKPlugin {
            replacement: true,
            negate: false,
            scale: 1.0,
            k: 0,
        },
    );
    assert!(result.is_err());
}
