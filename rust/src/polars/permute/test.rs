use super::*;

#[cfg(feature = "ffi")]
fn plugin(input: Vec<Expr>, flags: FunctionOptions) -> Expr {
    Expr::Function {
        input,
        function: FunctionExpr::FfiPlugin {
            lib: "opendp".into(),
            symbol: "permute_plugin".into(),
            kwargs: Default::default(),
            flags,
        },
    }
}

#[cfg(feature = "ffi")]
#[test]
fn test_match_permute() {
    let expr = plugin(vec![col("x")], FunctionOptions::length_preserving());
    assert_eq!(match_permute(&expr).unwrap(), Some(&col("x")));

    // Wrong arity and execution flags cannot establish uniform permutation.
    assert!(
        match_permute(&plugin(vec![], FunctionOptions::length_preserving()))
            .unwrap()
            .is_none()
    );
    assert!(
        match_permute(&plugin(
            vec![col("x"), lit(1)],
            FunctionOptions::length_preserving()
        ))
        .unwrap()
        .is_none()
    );
    assert!(
        match_permute(&plugin(vec![col("x")], FunctionOptions::elementwise()))
            .unwrap()
            .is_none()
    );
    assert!(
        match_permute(&plugin(vec![col("x")], FunctionOptions::groupwise()))
            .unwrap()
            .is_none()
    );

    let mut expr = expr;
    if let Expr::Function {
        function: FunctionExpr::FfiPlugin { symbol, .. },
        ..
    } = &mut expr
    {
        *symbol = "other_plugin".into();
    }
    assert!(match_permute(&expr).unwrap().is_none());
}

#[test]
fn test_permute_length_and_dtype() -> PolarsResult<()> {
    let data = Series::new("x".into(), [Some(1i64), None, Some(3)]);
    let result = permute(&[data.clone()])?;
    assert_eq!(result.len(), data.len());
    assert_eq!(result.dtype(), data.dtype());
    assert_eq!(result.name(), data.name());
    assert_eq!(result.null_count(), 1);

    let empty = Series::new_empty("x".into(), &DataType::Int64);
    assert!(permute(&[empty])?.is_empty());

    let singleton = Series::new("x".into(), [7i64]);
    let output = permute(&[singleton])?;
    assert_eq!(output.len(), 1);
    assert!(matches!(output.get(0)?, AnyValue::Int64(7)));

    assert!(permute(&[]).is_err());
    assert!(permute(&[data.clone(), data]).is_err());
    Ok(())
}

#[test]
fn test_match_native_permute() -> PolarsResult<()> {
    use crate::polars::PrivacyNamespace;
    let expr = col("x").dp().permute();
    assert_eq!(match_permute(&expr).unwrap(), Some(&col("x")));

    let input = df!("x" => [0i32, 1, 2, 3, 4, 5, 6, 7, 8, 9])?.lazy();
    let output = input.select([expr]).collect()?;
    let values = output.column("x")?.i32()?;
    let mut sorted: Vec<_> = values.into_no_null_iter().collect();
    sorted.sort();
    assert_eq!(sorted, (0..10).collect::<Vec<_>>());
    Ok(())
}
