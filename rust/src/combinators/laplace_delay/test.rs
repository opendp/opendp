use cpu_time::ProcessTime;

use crate::{
    domains::{AtomDomain, VectorDomain},
    measurements::then_laplace,
    metrics::ChangeOneDistance,
    transformations::then_count,
    transformations::then_metric_unbounded,
};

use super::*;

fn measure_nanos(size: usize) -> Fallible<u128> {
    let start = ProcessTime::now();
    let space = (
        VectorDomain::new(AtomDomain::<i64>::default()).with_size(size),
        ChangeOneDistance,
    );

    let m_count = (space >> then_metric_unbounded() >> then_count::<_, i64>() >> then_laplace(1.))?;

    let m_count_ts = Measurement::new(
        m_count.input_domain.clone(),
        m_count.function.clone(),
        m_count.input_metric.clone(),
        m_count.output_measure.clone(),
        m_count.privacy_map.clone().with_timing(|_| Ok(10)),
    )?;

    let m_count_delayed = make_laplace_delay(&m_count_ts, 1_000, 100.0)?;

    m_count_delayed.invoke(&vec![1; size])?;

    Ok(start.elapsed().as_nanos())
}

#[test]
fn test_laplace_timings() -> Fallible<()> {
    (0..10_000usize).for_each(|_| println!("{}", measure_nanos(5).unwrap()));
    Ok(())
}

#[test]
fn test_laplace_delay() -> Fallible<()> {
    let space = (
        VectorDomain::new(AtomDomain::<i64>::default()).with_size(5),
        ChangeOneDistance,
    );

    let m_count = (space >> then_metric_unbounded() >> then_count::<_, i64>() >> then_laplace(1.))?;

    let m_count_ts = Measurement::new(
        m_count.input_domain.clone(),
        m_count.function.clone(),
        m_count.input_metric.clone(),
        m_count.output_measure.clone(),
        m_count.privacy_map.clone().with_timing(|_| Ok(10)),
    )?;

    let m_count_delayed = make_laplace_delay(&m_count_ts, 1_000, 100.0)?;

    println!("release: {}", m_count_delayed.invoke(&vec![1, 2, 3, 4, 5])?);

    println!("combined loss: {:?}", m_count.map(&1)?);
    println!("combined loss: {:?}", m_count_delayed.map(&1)?);

    Ok(())
}
