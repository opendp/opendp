use super::*;

fn plugin(input: Vec<Expr>, flags: FunctionOptions) -> Expr {
    let mut expr = apply_anonymous_function(input, PermutePlugin);
    if let Expr::AnonymousFunction { options, .. } = &mut expr {
        *options = flags;
    }
    expr
}

#[test]
fn test_match_permute() {
    let expr = plugin(vec![col("x")], FunctionOptions::length_preserving());
    assert_eq!(match_permute(&expr).unwrap(), Some(&col("x")));
    for (input, options) in [
        (vec![], FunctionOptions::length_preserving()),
        (vec![col("x"), lit(1)], FunctionOptions::length_preserving()),
        (vec![col("x")], FunctionOptions::elementwise()),
        (vec![col("x")], FunctionOptions::groupwise()),
    ] {
        assert!(match_permute(&plugin(input, options)).unwrap().is_none());
    }
}

#[test]
fn test_permute_invalid_arity() {
    let data = Series::new("x".into(), [1i64]);
    assert!(permute(&[]).is_err());
    assert!(permute(&[data.clone(), data]).is_err());
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
