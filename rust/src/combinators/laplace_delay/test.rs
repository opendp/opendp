use std::{
    fs::{remove_file, File},
    io::{BufWriter, Write},
    time::Instant,
};

use crate::{
    domains::{AtomDomain, VectorDomain},
    measurements::then_laplace,
    metrics::ChangeOneDistance,
    transformations::{then_count, then_metric_unbounded, then_sum},
};

use super::*;

fn make_test_timing_measurement(
    size: usize,
) -> Fallible<
    Measurement<
        VectorDomain<AtomDomain<u64>>,
        u64,
        ChangeOneDistance,
        FixedSmoothedMaxDivergence<f64>,
    >,
> {
    let space = (
        VectorDomain::new(AtomDomain::<u64>::new_closed((0, 1))?).with_size(size),
        ChangeOneDistance,
    );

    let m_count =
        (space >> then_metric_unbounded() >> then_sum::<_, u64>() >> then_laplace(1.0f64))?;

    let m_count_ts = Measurement::new(
        m_count.input_domain.clone(),
        m_count.function.clone(),
        m_count.input_metric.clone(),
        m_count.output_measure.clone(),
        m_count
            .privacy_map
            .clone()
            .with_timing(|d_in| Ok(*d_in as u64 * 2_000)),
    )?;

    make_laplace_delay(&m_count_ts, 50_000, 4_000.0)
}

#[test]
fn test_laplace_timings() -> Fallible<()> {
    let n_trials = 100_000usize;

    let time_it = |m: Measurement<_, _, _, _>, x: Vec<u64>| {
        (0..n_trials)
            .map(|_| {
                let start = Instant::now();
                m.invoke(&x).expect("invoke failed");
                start.elapsed().as_nanos()
            })
            .collect::<Vec<u128>>()
    };

    let m_x1 = make_test_timing_measurement(1000)?;
    let m_x2 = make_test_timing_measurement(1001)?;

    let x1 = vec![1; 1000];
    let x2 = vec![1; 1001];

    let (t1, t2) = (time_it(m_x1, x1), time_it(m_x2, x2));

    // Create a file
    let file_name = "timings.csv";
    remove_file(file_name).ok();
    let mut f = BufWriter::new(File::create(file_name).unwrap());
    t1.iter().zip(t2.iter()).for_each(|(t1, t2)| {
        write!(f, "{},{}\n", t1, t2).unwrap();
    });

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
