//! A marker for population variance in a private group-by workload.
//!
//! The group-by workload planner lowers this marker into one noisy count,
//! one noisy sum, and one noisy bounded centered second moment. The marker
//! itself is never executed on private data.

use std::sync::Arc;

#[cfg(feature = "ffi")]
use polars::series::Series;
use polars::{
    error::{PolarsResult, polars_bail, polars_err},
    prelude::{AnonymousColumnsUdf, Column, ColumnsUdf, DataType, Expr, Field},
};
use polars_plan::prelude::FunctionOptions;
use serde::{Deserialize, Serialize};

use crate::polars::OpenDPPlugin;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct DPVarShim;

impl ColumnsUdf for DPVarShim {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Column> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}

impl AnonymousColumnsUdf for DPVarShim {
    fn as_column_udf(self: Arc<Self>) -> Arc<dyn ColumnsUdf> {
        self
    }

    fn deep_clone(self: Arc<Self>) -> Arc<dyn AnonymousColumnsUdf> {
        Arc::new(Arc::unwrap_or_clone(self))
    }

    fn get_field(&self, _: &polars::prelude::Schema, fields: &[Field]) -> PolarsResult<Field> {
        let [input, ..] = fields else {
            return Err(polars_err!(InvalidOperation: "{} expects a column", Self::NAME));
        };
        Ok(Field::new(input.name().clone(), DataType::Float64))
    }
}

impl OpenDPPlugin for DPVarShim {
    const NAME: &'static str = "dp_var";

    #[cfg(feature = "ffi")]
    const SHIM: bool = true;

    fn function_options() -> FunctionOptions {
        FunctionOptions::aggregation()
    }
}

#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type=Null)]
fn dp_var(_: &[Series]) -> PolarsResult<Series> {
    polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
}
