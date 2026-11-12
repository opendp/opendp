//! Shared type checking and column-domain inference for verified query plans.
//! Independent of the concrete frame carrier and of the Polars frontend.

use polars::prelude::{DataType, Field, PlSmallStr};

use crate::{
    domains::ColumnDomain,
    error::*,
    verified_query::{self as verified, VExpr, VField, VDType},
};

/// Translate logical OpenDP column constraints into the verified kernel's
/// types. Shared by native membership and Polars normalization.
pub(crate) fn kernel_schema<C>(columns: &[ColumnDomain<C>]) -> Fallible<Vec<VField>> {
    columns.iter().map(|column| {
        let dtype = match column.dtype() {
            DataType::Int64 => verified::VDType::Int64,
            DataType::Boolean => verified::VDType::Boolean,
            dtype => return fallible!(MakeTransformation,
                "unsupported verified column dtype: {:?}", dtype),
        };
        Ok(verified::VField { dtype, nullable: column.nullable })
    }).collect()
}

/// Infer the same verified projection columns for any frame carrier.
/// Only a column reference (possibly wrapped in aliases) retains its
/// original value-domain constraints; derived expressions get broad domains.
pub(crate) fn project_columns<C: Clone>(
    input: &[ColumnDomain<C>], exprs: &[VExpr],
) -> Fallible<Vec<ColumnDomain<C>>> {
    let schema = kernel_schema(input)?;
    exprs.iter().map(|expr| {
        let field = verified::check_expr(expr, &schema)
            .ok_or_else(|| err!(MakeTransformation, "ill-typed verified projection expression"))?;
        let name = match expr {
            VExpr::Alias { name, .. } => PlSmallStr::from(name.as_str()),
            VExpr::Column(index) => input.get(*index)
                .ok_or_else(|| err!(MakeTransformation, "column outside verified schema"))?
                .name.clone(),
            _ => return fallible!(MakeTransformation,
                "verified projection expressions must be named"),
        };
        let mut column = match verified::project::identity_column(expr) {
            Some(index) => input.get(index)
                .ok_or_else(|| err!(MakeTransformation, "column outside verified schema"))?
                .clone(),
            None => {
                let dtype = match field.dtype {
                    VDType::Int64 => DataType::Int64,
                    VDType::Boolean => DataType::Boolean,
                };
                let mut column = ColumnDomain::<C>::new_from_field(Field::new(name.clone(), dtype))?;
                column.nullable = field.nullable;
                column
            },
        };
        column.name = name;
        Ok(column)
    }).collect()
}

/// The shared filter typing rule for native and verified plan frontends.
pub(crate) fn check_filter_predicate<C>(
    input: &[ColumnDomain<C>], predicate: &VExpr,
) -> Fallible<()> {
    let field = verified::check_expr(predicate, &kernel_schema(input)?)
        .ok_or_else(|| err!(MakeTransformation, "ill-typed verified filter predicate"))?;
    if field.dtype != VDType::Boolean {
        return fallible!(MakeTransformation, "verified filter predicate must be boolean");
    }
    Ok(())
}
