//! Membership of concrete verified tables in OpenDP frame domains.
//! Unbound VPlans are syntax; checking a plan on bound input is a test utility.

use polars::prelude::{DataType, Field, PlSmallStr};

use crate::verified_query as verified;
use crate::{
    core::Domain,
    domains::{ColumnDomain, Frame, FrameDomain, Margin},
    error::*,
};

use verified::{VExpr, VField, VTable, VValue, VDType};

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

#[derive(Clone)]
pub struct VColumn(pub Vec<Option<VValue>>);

impl Domain for ColumnDomain<VColumn> {
    type Carrier = VColumn;

    fn member(&self, column: &VColumn) -> Fallible<bool> {
        match self.dtype() {
            DataType::Int64 => {
                let atom = self.atom_domain::<i64>()?;
                for cell in &column.0 {
                    match cell {
                        None if self.nullable => {},
                        Some(VValue::Int(v)) if atom.member(v)? => {},
                        _ => return Ok(false),
                    }
                }
            },
            DataType::Boolean => {
                let atom = self.atom_domain::<bool>()?;
                for cell in &column.0 {
                    match cell {
                        None if self.nullable => {},
                        Some(VValue::Bool(v)) if atom.member(v)? => {},
                        _ => return Ok(false),
                    }
                }
            },
            dtype => return fallible!(MakeDomain, "unsupported verified dtype: {:?}", dtype),
        }
        Ok(true)
    }
}

fn roots(expr: &VExpr, columns: &[ColumnDomain<VColumn>]) -> Fallible<Vec<PlSmallStr>> {
    if verified::check_expr(expr, &kernel_schema(columns)?).is_none() {
        return fallible!(MakeDomain, "ill-typed verified grouping expression");
    }

    fn collect(
        expr: &VExpr,
        columns: &[ColumnDomain<VColumn>],
        names: &mut Vec<PlSmallStr>,
    ) -> Fallible<()> {
        match expr {
            VExpr::Column(index) => {
                let column = columns.get(*index)
                    .ok_or_else(|| err!(MakeDomain, "verified column index outside schema"))?;
                names.push(column.name.clone());
            },
            VExpr::Alias { expr, .. } => collect(expr, columns, names)?,
            VExpr::Literal(_) | VExpr::Null(_) => {},
            VExpr::Binary { left, right, .. } => {
                collect(left, columns, names)?;
                collect(right, columns, names)?;
            },
            VExpr::Clip { expr, lower, upper } => {
                collect(expr, columns, names)?;
                if let Some(lower) = lower { collect(lower, columns, names)?; }
                if let Some(upper) = upper { collect(upper, columns, names)?; }
            },
        }
        Ok(())
    }

    let mut names = Vec::new();
    collect(expr, columns, &mut names)?;
    Ok(names)
}

fn margin_member(margin: &Margin<VExpr>, table: &VTable) -> Fallible<bool> {
    // Key order is arbitrary but fixed within this invocation.
    // The verifier checks both expression evaluation and group counting.
    let keys = margin.by.iter().cloned().collect::<Vec<_>>();
    verified::margin::check_margin(
        &table.rows, &keys, margin.max_length, margin.max_groups,
    ).map_err(|_| err!(MakeDomain, "invalid verified grouping expression"))
}

fn table_member(table: &VTable, domain: &FrameDomain<VTable>) -> Fallible<bool> {
    if !verified::check_table(table) || table.schema.len() != domain.series_domains.len() {
        return Ok(false);
    }
    let expected_schema = kernel_schema(&domain.series_domains)?;
    // The Verus theorems quantify over tables with exactly the declared
    // source schema, including nullability (even for empty tables).
    if table.schema != expected_schema {
        return Ok(false);
    }
    for (index, column) in domain.series_domains.iter().enumerate() {
        let values = table.rows.iter().map(|row| row[index]).collect();
        if !column.member(&VColumn(values))? {
            return Ok(false);
        }
    }
    for margin in &domain.margins {
        for expr in &margin.by {
            if verified::check_expr(expr, &table.schema).is_none() {
                return Ok(false);
            }
        }
        if !margin_member(margin, table)? {
            return Ok(false);
        }
    }
    Ok(true)
}

impl Frame for VTable {
    type Column = VColumn;
    type Expr = VExpr;

    fn root_names(expr: &VExpr, columns: &[ColumnDomain<VColumn>]) -> Fallible<Vec<PlSmallStr>> {
        roots(expr, columns)
    }
    fn frame_member(&self, domain: &FrameDomain<Self>) -> Fallible<bool> {
        table_member(self, domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::{AtomDomain, OptionDomain};
    use verified::{VPlan, VDType};

    /// Concrete execution is useful for testing; this is not the carrier
    /// of a symbolic OpenDP DSL transformation.
    fn bound_plan_member(
        domain: &FrameDomain<VTable>, input: &VTable, plan: &VPlan,
    ) -> Fallible<bool> {
        if !verified::check_table(input) {
            return Ok(false);
        }
        let Some(schema) = verified::check_plan(plan, &input.schema) else {
            return Ok(false);
        };
        let Ok(rows) = verified::evaluate(plan, input) else {
            return Ok(false);
        };
        domain.member(&VTable { schema, rows })
    }

    fn table() -> VTable {
        VTable {
            schema: vec![
                VField { dtype: VDType::Int64, nullable: true },
                VField { dtype: VDType::Boolean, nullable: true },
            ],
            rows: vec![
                vec![Some(VValue::Int(3)), Some(VValue::Bool(true))],
                vec![Some(VValue::Int(7)), Some(VValue::Bool(false))],
                vec![None, None],
            ],
        }
    }

    fn domain() -> Fallible<FrameDomain<VTable>> {
        FrameDomain::new(vec![
            ColumnDomain::new("x", AtomDomain::<i64>::new_closed((0, 5))?),
            ColumnDomain::new("flag", OptionDomain::new(AtomDomain::<bool>::default())),
        ])
    }

    #[test]
    fn verified_table_checks_values_and_nulls() -> Fallible<()> {
        let domain = domain()?;
        assert!(!domain.member(&table())?);
        let mut value = table();
        value.rows[1][0] = Some(VValue::Int(5));
        assert!(!domain.member(&value)?); // x is non-nullable
        value.rows[2][0] = Some(VValue::Int(0));
        value.schema[0].nullable = false;
        assert!(domain.member(&value)?);
        value.rows[1][0] = Some(VValue::Bool(true));
        assert!(!domain.member(&value)?);
        Ok(())
    }

    #[test]
    fn verified_table_schema_requires_exact_nullability() -> Fallible<()> {
        let domain = FrameDomain::<VTable>::new(vec![
            ColumnDomain::new("x", AtomDomain::<i64>::default()),
        ])?;
        let mut table = VTable {
            schema: vec![VField { dtype: verified::VDType::Int64, nullable: false }],
            rows: vec![],
        };
        assert!(domain.member(&table)?);
        // A nullable *declaration* does not satisfy a non-null contract,
        // even if the data happens to contain no nulls.
        table.schema[0].nullable = true;
        assert!(!domain.member(&table)?);

        let nullable = FrameDomain::<VTable>::new(vec![
            ColumnDomain::new("x", OptionDomain::new(AtomDomain::<i64>::default())),
        ])?;
        assert!(nullable.member(&table)?);
        table.schema[0].nullable = false;
        assert!(!nullable.member(&table)?);
        Ok(())
    }

    #[test]
    fn verified_grouping_checks_keys_and_empty_groups() -> Fallible<()> {
        let domain = FrameDomain::<VTable>::new(vec![
            ColumnDomain::new("x", OptionDomain::new(AtomDomain::<i64>::default())),
            ColumnDomain::new("flag", OptionDomain::new(AtomDomain::<bool>::default())),
        ])?;
        let by_x = Margin::by([VExpr::Column(0)]).with_max_length(1);
        let grouped = domain.clone().with_margin(by_x)?;
        assert!(grouped.member(&table())?);
        let mut repeated = table();
        repeated.rows[1][0] = Some(VValue::Int(3));
        assert!(!grouped.member(&repeated)?);
        assert!(domain.clone().with_margin(Margin::select().with_max_length(3))?.member(&table())?);
        assert!(!domain.with_margin(Margin::select().with_max_groups(0))?.member(
            &VTable { schema: table().schema, rows: vec![] },
        )?);
        Ok(())
    }

    #[test]
    fn verified_frame_distance_uses_verified_grouping_keys() -> Fallible<()> {
        use crate::core::MetricSpace;
        use crate::metrics::{Bounds, FrameDistanceFor, SymmetricDistance};
        use std::collections::HashSet;

        let metric = FrameDistanceFor::<SymmetricDistance, VExpr>::new(SymmetricDistance);
        (domain()?, metric.clone()).check_space()?;
        let bounds: Bounds<VExpr> = Bounds::from(3);
        let by = HashSet::from([VExpr::Column(0)]);
        assert_eq!(bounds.get_bound(&by).per_group, Some(3));
        Ok(())
    }

    #[test]
    fn bound_plan_membership_uses_verified_evaluation() -> Fallible<()> {
        let mut domain = domain()?;
        domain.series_domains[0] = ColumnDomain::new(
            "x", OptionDomain::new(AtomDomain::<i64>::new_closed((0, 5))?),
        );
        let plan = VPlan::Project {
            input: Box::new(VPlan::Filter {
                input: Box::new(VPlan::Input),
                predicate: VExpr::Column(1),
            }),
            exprs: vec![
                VExpr::Clip {
                    expr: Box::new(VExpr::Column(0)),
                    lower: None,
                    upper: Some(Box::new(VExpr::Literal(VValue::Int(5)))),
                },
                VExpr::Column(1),
            ],
        };
        assert!(bound_plan_member(&domain, &table(), &plan)?);
        let empty = VTable { schema: table().schema, rows: vec![] };
        let ill_typed = VPlan::Filter {
            input: Box::new(VPlan::Input),
            predicate: VExpr::Column(0),
        };
        assert!(!bound_plan_member(&domain, &empty, &ill_typed)?); // ill-typed even on empty input
        assert!(domain.with_margin(Margin::by([VExpr::Column(99)])).is_err());
        Ok(())
    }
}
