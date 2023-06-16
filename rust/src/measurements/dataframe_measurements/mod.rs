use num::Float as _;
use opendp_derive::bootstrap;

use rug::{Rational};
use polars::prelude::*;
//use polars::datatypes::DataType::Utf8;

use crate::{
    core::{Function, Measurement, Metric, MetricSpace, PrivacyMap},
    domains::{AtomDomain, LazyFrameDomain},
    measurements::{make_base_laplace, get_discretization_consts},
    error::Fallible,
    measures::MaxDivergence,
    metrics::{AbsoluteDistance, L1Distance},
    traits::{samplers::{SampleDiscreteLaplaceZ2k,CastInternalRational}, CheckAtom, ExactIntCast, Float, FloatBits, InfAdd, InfDiv},
};

use super::{MappableDomain};


//#[cfg(feature = "ffi")]
//mod ffi;

/*#[bootstrap(
    features("contrib"),
    arguments(
        scale(rust_type = "T", c_type = "void *"),
        k(default = -1074, rust_type = "i32", c_type = "uint32_t")),
    generics(
        D(default = "AtomDomain<T>", generics = "T"),
        MO(default = "ZeroConcentratedDivergence<T>", generics = "T")),
    derived_types(T = "$get_atom_or_infer(D, scale)")
)]*/

/// Make a Measurement that adds noise from the Laplace(`scale`) distribution to the last column of a polars dataframe.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | input type   | `input_metric`         |
/// | ------------------------------- | ------------ | ---------------------- |
/// | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
/// | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == sqrt(2) * standard_deviation.
/// * `k` - The noise granularity in terms of 2^k.
pub fn make_polarsDF_laplace(
    input_domain: LazyFrameDomain,
    input_metric: L1Distance<f64>, // TO DO !!!
    scale: f64,
    k: Option<i32>,
) -> Fallible<
    Measurement<
     LazyFrameDomain,
     LazyFrame,
     L1Distance<f64>,
     MaxDivergence<f64>
    >
> where
    i32: ExactIntCast<<f64 as FloatBits>::Bits>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    // Create Laplace measurement
    let scalar_laplace_measurement = make_base_laplace(
        AtomDomain::default(),
        AbsoluteDistance::default(),
        scale.clone(),
        k.clone())?;

    let (_k, relaxation) = get_discretization_consts(k.clone())?;

    Measurement::new(
        input_domain.clone(),
        Function::new_fallible(move |data: &LazyFrame| {

            // Get last column name and position
            let number_columns = input_domain.series_domains.len();
            let last_column_id = number_columns.clone() - 1;
            let last_column_name = data.clone().collect()?;
            let last_column_name = last_column_name.get_column_names()[last_column_id].clone();


            // Retrieve AtomDomain
            //let sum_series_domain = input_domain.column(sum_name.clone().as_str()).unwrap().clone();
            //let sumAtomDomain = sum_series_domain.element_domain.atom_domain()
            //let series_type = data.clone().collect()?.column(last_column_name)?.dtype().clone();
            //let test = Rational::from_f64(scale.clone().to_f64().unwrap()).unwrap() ;
            //let scale_test = CastInternalRational::from_rational(test);

            // Retreive series of last column
            let s = data
                                .clone()
                                .collect()?
                                .column(last_column_name)?
                                .clone();
            
            // Add noise to series
            let mut s_with_noise = Series::from_iter(
                                        s.unpack::<Float64Type>()?
                                        .into_iter()
                                        .map(|v|
                                            v.map(|v| scalar_laplace_measurement.invoke(&v).unwrap())),
            );
            s_with_noise.rename(s.name());

            // Add noised series to dataframe output 
            let result = data.clone().collect()?.with_column(s_with_noise)?.clone().lazy();
            
            Ok(result.clone())
        }),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &f64| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if scale == 0.0 {
                return Ok(f64::infinity());
            }

            // increase d_in by the worst-case rounding of the discretization
            let d_in = d_in.inf_add(&relaxation)?;

            // d_in / scale
            d_in.inf_div(&scale)
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
     fn test_dataFrame_noiseAddition_known_categories() -> Fallible<()> {
        use crate::domains::SeriesDomain;
        use crate::transformations::make_sized_partitioned_sum;

        let data_frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("Country", AtomDomain::<String>::default()),
            SeriesDomain::new("Age", AtomDomain::<i32>::default()),
        ])?
        .with_counts(df!["Country" => ["CH", "US"], "count" => [3, 2]]?.lazy())?;

        let data = df!("Country" => ["CH", "CH", "US", "CH", "US"], "Age" => [1, 4, 2, 8, 10])?.lazy();


        let bounded_partitioned_sum = make_sized_partitioned_sum(
                                                                            data_frame_domain.clone(),
                                                                            "Country",
                                                                            "Age",
                                                                            (0.,99.),
                                                                            false).unwrap();

        let laplace_mechanism = make_polarsDF_laplace(
            bounded_partitioned_sum.output_domain.clone(),
            L1Distance::<f64>::default(),
            1.0,
            None
        ).unwrap();

        let partioned_sums = bounded_partitioned_sum.invoke(&data).unwrap(); 

        println!("{} days", partioned_sums.clone().collect()?);

        let noised_result = laplace_mechanism.invoke(&partioned_sums).unwrap();

        println!("{} days", noised_result.clone().collect()?);

        let pipeline = (bounded_partitioned_sum >> laplace_mechanism)?;

        let noised_result_pipeline = pipeline.invoke(&data).unwrap();

        println!("{} days", noised_result_pipeline.clone().collect()?);
        //assert!(result.clone().collect()?.frame_equal(&df_check));

        Ok(())
    }

}
