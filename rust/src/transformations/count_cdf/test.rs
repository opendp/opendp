use super::*;
#[test]
fn test_cdf() -> Fallible<()> {
    let cdf_func = make_cdf::<f64>()?;
    let cdf = cdf_func.eval(&vec![2.23, 3.4, 5., 2.7])?;
    assert_eq!(
        cdf,
        vec![
            0.16729182295573897,
            0.42235558889722435,
            0.7974493623405852,
            1.0
        ]
    );
    Ok(())
}

#[test]
fn test_quantile() -> Fallible<()> {
    let edges = vec![0, 25, 50, 75, 100];
    let alphas = vec![0., 0.1, 0.24, 0.51, 0.74, 0.75, 0.76, 0.99, 1.];
    let quantile_func =
        make_quantiles_from_counts(edges.clone(), alphas.clone(), Interpolation::Nearest)?;
    let quantiles = quantile_func.eval(&vec![100, 100, 100, 100])?;
    assert_eq!(quantiles, vec![0, 0, 25, 50, 75, 75, 75, 100, 100]);

    let quantile_func = make_quantiles_from_counts(edges, alphas, Interpolation::Linear)?;
    let quantiles = quantile_func.eval(&vec![100, 100, 100, 100])?;
    assert_eq!(quantiles, vec![0, 10, 24, 51, 74, 75, 76, 99, 100]);
    Ok(())
}

#[test]
fn test_quantile_with_edge_buckets() -> Fallible<()> {
    let edges = vec![0, 25, 50, 75, 100];
    let alphas = vec![0., 0.1, 0.24, 0.51, 0.74, 0.75, 0.76, 0.99, 1.];
    let quantile_func = make_quantiles_from_counts(edges, alphas, Interpolation::Nearest)?;
    let quantiles = quantile_func.eval(&vec![210, 100, 100, 100, 100, 234])?;
    println!("{:?}", quantiles);
    assert_eq!(quantiles, vec![0, 0, 25, 50, 75, 75, 75, 100, 100]);
    Ok(())
}

#[test]
fn test_quantile_float() -> Fallible<()> {
    let edges = vec![0., 10., 20., 30.];
    let alphas = vec![0.2, 0.4, 0.7];
    let quantile_trans =
        make_quantiles_from_counts(edges.clone(), alphas.clone(), Interpolation::Nearest)?;
    let quantiles = quantile_trans.eval(&vec![2.23, 3.4, 5.])?;
    assert_eq!(quantiles, vec![10., 20., 20.]);

    let quantile_trans = make_quantiles_from_counts(edges, alphas, Interpolation::Linear)?;
    let quantiles = quantile_trans.eval(&vec![2.23, 3.4, 5.])?;
    assert_eq!(
        quantiles,
        vec![9.533632286995514, 15.94705882352941, 23.622]
    );
    println!("{:?}", quantiles);
    Ok(())
}

#[test]
fn test_quantile_int() -> Fallible<()> {
    let edges = vec![0, 10, 50, 100];
    let alphas = vec![0.2, 0.4, 0.7];
    let quantile_func = make_quantiles_from_counts(edges, alphas, Interpolation::Nearest)?;
    let quantiles = quantile_func.eval(&vec![2, 3, 5])?;
    assert_eq!(quantiles, vec![10, 50, 50]);
    Ok(())
}
