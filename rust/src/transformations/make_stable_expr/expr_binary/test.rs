use crate::domains::{LazyFrameDomain, OptionDomain, SeriesDomain};
use crate::metrics::SymmetricDistance;

use super::*;

fn get_f64_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("L", AtomDomain::<f64>::default()),
        SeriesDomain::new("R", OptionDomain::new(AtomDomain::<f64>::default())),
    ])?;

    let lf = df!(
        "L" => [
            Some(0.), Some(1.),
            Some(f64::NAN), Some(0.),
            None, Some(0.),
            Some(f64::INFINITY), Some(0.)
        ],
        "R" => [
            Some(0.), Some(0.),
            Some(f64::NAN), Some(f64::NAN),
            None, None,
            Some(f64::INFINITY), Some(f64::INFINITY)
        ],
    )?
    .lazy();

    Ok((lf_domain, lf))
}

fn get_bool_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("L", AtomDomain::<bool>::default()),
        SeriesDomain::new("R", OptionDomain::new(AtomDomain::<bool>::default())),
    ])?;

    let lf = df!(
        "L" => [true, true, false, false, true],
        "R" => [
            Some(true), Some(false),
            Some(true), Some(false),
            None,
        ],
    )?
    .lazy();

    Ok((lf_domain, lf))
}

macro_rules! test_binary {
    ($get:ident, $op:ident, $expected:expr) => {
        let (lf_domain, lf) = $get()?;
        let expr_domain = lf_domain.row_by_row();
        assert_eq!(
            lf.with_column(col("L").$op(col("R")))
                .collect()?
                .column("L")?,
            &Series::new("L", $expected)
        );
        let t_op = col("L")
            .$op(col("R"))
            .make_stable(expr_domain.clone(), SymmetricDistance)?;
        let output_series = t_op.output_domain.active_series()?;
        assert_eq!(&*output_series.field.name, "L");
        assert!(output_series.nullable);
    };
}

#[test]
fn test_binary_ops() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        eq,
        [
            Some(true),
            Some(false),
            // NaN is equal to NaN?
            Some(true),
            Some(false),
            None,
            None,
            Some(true),
            Some(false),
        ]
    );

    test_binary!(
        get_f64_data,
        neq,
        [
            Some(false),
            Some(true),
            Some(false),
            Some(true),
            None,
            None,
            Some(false),
            Some(true),
        ]
    );

    test_binary!(
        get_f64_data,
        lt,
        [
            Some(false),
            Some(false),
            // zero is less than NaN?!
            Some(false),
            Some(true),
            None,
            None,
            Some(false),
            Some(true),
        ]
    );

    test_binary!(
        get_f64_data,
        lt_eq,
        [
            Some(true),
            Some(false),
            // nan is equal to NaN? zero is lte NaN?
            Some(true),
            Some(true),
            None,
            None,
            Some(true),
            Some(true),
        ]
    );

    test_binary!(
        get_f64_data,
        gt,
        [
            Some(false),
            Some(true),
            Some(false),
            Some(false),
            None,
            None,
            Some(false),
            Some(false),
        ]
    );

    test_binary!(
        get_f64_data,
        gt_eq,
        [
            Some(true),
            Some(true),
            // nan is equal to NaN?
            Some(true),
            Some(false),
            None,
            None,
            Some(true),
            Some(false),
        ]
    );

    test_binary!(
        get_bool_data,
        and,
        [Some(true), Some(false), Some(false), Some(false), None,]
    );

    test_binary!(
        get_bool_data,
        or,
        [Some(true), Some(true), Some(true), Some(false), Some(true),]
    );

    test_binary!(
        get_bool_data,
        xor,
        [
            Some(false),
            Some(true),
            Some(true),
            Some(false),
            // why not same behavior as or?
            None,
        ]
    );

    Ok(())
}
