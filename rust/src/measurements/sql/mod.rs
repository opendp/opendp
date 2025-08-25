use std::{collections::HashMap, sync::Arc};

use opendp_derive::bootstrap;
use polars::{
    error::PolarsResult,
    prelude::{LazyFrame, LazySerde, PlSmallStr, SpecialEq},
    sql::{FunctionRegistry, SQLContext},
};
use polars_plan::dsl::UserDefinedFunction;

use crate::{
    error::Fallible,
    measurements::{
        expr_dp_counting_query::{DPCountShim, DPLenShim, DPNUniqueShim, DPNullCountShim},
        expr_dp_frame_len::DPFrameLenShim,
        expr_dp_mean::DPMeanShim,
        expr_dp_median::DPMedianShim,
        expr_dp_quantile::DPQuantileShim,
        expr_dp_sum::DPSumShim,
        expr_index_candidates::IndexCandidatesShim,
        expr_noise::NoiseShim,
        expr_noisy_max::NoisyMaxShim,
    },
    polars::OpenDPPlugin,
    transformations::expr_discrete_quantile_score::DiscreteQuantileScoreShim,
};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

/// Translate a SQL query into a lazyframe plan.
///
/// # Arguments
/// * `query` - The sql query.
/// * `tables` - Hashmap of tables involved in query.
#[bootstrap(arguments(tables(c_type = "AnyObject *")))]
pub fn sql_to_plan(query: String, tables: HashMap<String, LazyFrame>) -> Fallible<LazyFrame> {
    let registry = ODPFunctionRegistry::new()?;
    let mut context = SQLContext::new().with_function_registry(Arc::new(registry));
    tables.into_iter().for_each(|(name, table)| {
        context.register(name.as_str(), table);
    });
    context.execute(query.as_str()).map_err(|e| e.into())
}

#[derive(Default)]
struct ODPFunctionRegistry {
    registry: HashMap<PlSmallStr, UserDefinedFunction>,
}

fn register<P: OpenDPPlugin>(plugin: P) -> Fallible<(PlSmallStr, UserDefinedFunction)> {
    let function = UserDefinedFunction {
        name: P::NAME.into(),
        return_type: plugin
            .get_output()
            .ok_or_else(|| err!(MakeMeasurement, "output must be known"))?,
        fun: LazySerde::Deserialized(SpecialEq::new(Arc::new(plugin))),
        options: P::function_options(),
    };
    Ok((P::NAME.into(), function))
}

impl ODPFunctionRegistry {
    fn new() -> Fallible<Self> {
        macro_rules! map_shims {
            ($($expr:expr),+) => ([$(register($expr)?),+])
        }
        Ok(ODPFunctionRegistry {
            registry: HashMap::from(map_shims![
                IndexCandidatesShim,
                NoiseShim,
                NoisyMaxShim,
                DiscreteQuantileScoreShim,
                DPFrameLenShim,
                DPLenShim,
                DPCountShim,
                DPNullCountShim,
                DPNUniqueShim,
                DPSumShim,
                DPMeanShim,
                DPQuantileShim,
                DPMedianShim
            ]),
        })
    }
}

impl FunctionRegistry for ODPFunctionRegistry {
    fn register(&mut self, name: &str, fun: UserDefinedFunction) -> PolarsResult<()> {
        self.registry.insert(name.into(), fun);
        Ok(())
    }

    fn get_udf(&self, name: &str) -> PolarsResult<Option<UserDefinedFunction>> {
        Ok(self.registry.get(name).cloned())
    }

    fn contains(&self, name: &str) -> bool {
        self.registry.contains_key(name)
    }
}
