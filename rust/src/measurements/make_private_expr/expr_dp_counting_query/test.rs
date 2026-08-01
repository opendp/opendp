use polars::prelude::*;

use crate::{
    domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::{L0PInfDistance, SymmetricDistance},
    polars::{PrivacyNamespace, match_shim},
};

use super::{
    DPLenShim, make_expr_dp_count, make_expr_dp_len, make_expr_dp_n_unique, make_expr_dp_null_count,
};

fn get_test_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "A",
        OptionDomain::new(AtomDomain::<i32>::default()),
    )])?;
    let lf = df!("A" => [Some(1), Some(2), None, Some(2)])?.lazy();
    Ok((lf_domain, lf))
}

macro_rules! test_counting_query_dtype {
    ($test_name:ident, $constructor:ident, $dp_method:ident, $expected:literal) => {
        #[test]
        fn $test_name() -> Fallible<()> {
            for (signed, dtype) in [(false, DataType::UInt32), (true, DataType::Int64)] {
                let (lf_domain, lf) = get_test_data()?;
                let expr = col("A").dp().$dp_method(Some(0.0), signed);
                let measurement = $constructor(
                    lf_domain.select(),
                    L0PInfDistance(SymmetricDistance),
                    MaxDivergence,
                    expr,
                    None,
                )?;
                let dp_expr = measurement.invoke(&lf.logical_plan)?.expr;
                let df = lf.select([dp_expr]).collect()?;

                assert_eq!(df.column("A")?.dtype(), &dtype, "signed={signed}");

                let value = df
                    .column("A")?
                    .cast(&DataType::Int64)?
                    .i64()?
                    .get(0)
                    .unwrap();
                assert_eq!(value, $expected, "signed={signed}");
            }
            Ok(())
        }
    };
}

// exact values for column A = [1, 2, None, 2]
test_counting_query_dtype!(test_dp_len_types, make_expr_dp_len, len, 4);
test_counting_query_dtype!(test_dp_count_types, make_expr_dp_count, count, 3);
test_counting_query_dtype!(
    test_dp_null_count_types,
    make_expr_dp_null_count,
    null_count,
    1
);
test_counting_query_dtype!(test_dp_n_unique_types, make_expr_dp_n_unique, n_unique, 3);

#[test]
fn test_counting_shim_schema_unsigned() -> Fallible<()> {
    // regardless of the source column's dtype, the unsigned shim infers UInt32
    let lf = df!("A" => [1.0f64, 2.0, 3.0])?.lazy();
    let expr = col("A").dp().count(Some(0.0), false);
    let schema = lf.select([expr]).collect_schema()?;
    assert_eq!(schema.get("A").unwrap(), &DataType::UInt32);
    Ok(())
}

#[test]
fn test_counting_shim_not_evaluable() -> Fallible<()> {
    let lf = df!("A" => [1, 2, 3])?.lazy();
    let expr = col("A").dp().count(Some(0.0), false);
    assert!(lf.select([expr]).collect().is_err());
    Ok(())
}

#[test]
fn test_counting_shim_signed_argument() -> Fallible<()> {
    for signed in [false, true] {
        let expr = col("A").dp().len(Some(0.0), signed);
        let [_, _, signed_arg] = match_shim::<DPLenShim, 3>(&expr)?.unwrap();
        assert_eq!(signed_arg, lit(signed));
    }
    Ok(())
}
