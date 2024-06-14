use std::collections::HashMap;

use crate::domains::{AtomDomain, Margin, SeriesDomain};

use super::*;
use polars::{prelude::*, sql::{FunctionOptions, FunctionRegistry, SQLContext}};
use polars_plan::{dsl::{GetOutput, UserDefinedFunction}, prelude::ApplyOptions};

struct ODPFunctionRegistry {
    registry: HashMap<String, UserDefinedFunction>,
}

impl FunctionRegistry for ODPFunctionRegistry {
    fn register(&mut self, name: &str, fun: UserDefinedFunction) -> PolarsResult<()> {
        self.registry.insert(name.to_string(), fun);
        Ok(())
    }

    fn get_udf(&self, name: &str) -> PolarsResult<Option<UserDefinedFunction>> {
        Ok(self.registry.get(name).cloned())
    }

    fn contains(&self, name: &str) -> bool {
        self.registry.contains_key(name)
    }
}

#[test]
fn test_sql() -> Fallible<()> {
    let mut ctx = SQLContext::new();
    let df = df! {
        "a" =>  [1, 2, 3],
    }?;

    ctx.register("df", df.clone().lazy());
    let fun = UserDefinedFunction {
        name: "laplace".to_string(),
        input_fields: vec![Field::new("a", DataType::Int32), Field::new("_", DataType::Float64)],
        return_type: GetOutput::from_type(DataType::Int32),
        fun: SpecialEq::new(Arc::new(|s: &mut [Series]| Ok(Some(s[0].clone())) )),
        options: FunctionOptions {
            collect_groups: ApplyOptions::ElementWise,
            ..Default::default()
        },
    };
    let mut registry = ODPFunctionRegistry { registry: HashMap::new() };
    registry.register("laplace", fun)?;
    let mut ctx2 = ctx.with_function_registry(Arc::new(registry ));

    let sql = "SELECT laplace(a, 1.0) FROM df";
    let sql_df = ctx2.execute(sql)?.collect()?;
    assert!(sql_df.equals(&df));
    Ok(())
}

#[test]
fn test_sql2() -> Fallible<()> {
    let mut ctx = SQLContext::new();
    let df = df! {
        "a" =>  [1, 2, 3],
        "b" =>  [1, 2, 3],
    }?;

    ctx.register("df", df.clone().lazy());

    let sql = "SELECT a FROM df GROUP BY b";
    let sql_df = ctx.execute(sql)?;
    let _ = make_stable_lazyframe(
        LazyFrameDomain::new(vec![
            SeriesDomain::new("a", AtomDomain::<i32>::default()),
            SeriesDomain::new("b", AtomDomain::<i32>::default()),
        ])?.with_margin(&["b"], Margin::new().with_public_keys())?,
        SymmetricDistance,
        sql_df,
    )?;
    Ok(())
}


