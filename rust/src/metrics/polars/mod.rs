use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use polars::prelude::Expr;

use crate::{
    core::Metric,
    domains::find_min_covering,
    error::Fallible,
    traits::{InfMul, ProductOrd, option_min},
    transformations::traits::UnboundedMetric,
};

use super::{ChangeOneDistance, IntDistance, MicrodataMetric, SymmetricDistance};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

pub type OwnerClaim = Vec<Expr>;

#[derive(Clone, PartialEq, Debug)]
pub struct Binding {
    pub exprs: Vec<Expr>,
    pub space: String,
}

impl Binding {
    pub fn root_names(&self) -> HashSet<polars::prelude::PlSmallStr> {
        self.exprs
            .iter()
            .flat_map(|expr| expr.clone().meta().root_names())
            .collect()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct UniqueKey {
    pub table: String,
    pub key: Expr,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ForeignKey {
    pub from_table: String,
    pub from: Expr,
    pub to_table: String,
    pub to: Expr,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FunctionalDependency {
    pub table: String,
    pub from: Expr,
    pub to: Expr,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Ownership {
    pub table: String,
    pub claims: Vec<OwnerClaim>,
}

fn canonical_expr(expr: &Expr) -> Expr {
    expr.clone().meta().undo_aliases()
}

fn canonical_binding(binding: &Binding) -> Binding {
    let mut exprs = binding
        .exprs
        .iter()
        .map(canonical_expr)
        .fold(Vec::new(), |mut acc, expr| {
            if !acc.contains(&expr) {
                acc.push(expr);
            }
            acc
        });
    exprs.sort_by_key(|expr| format!("{expr:?}"));
    Binding {
        exprs,
        space: binding.space.clone(),
    }
}

fn normalize_bindings(bindings: &[Binding]) -> Vec<Binding> {
    bindings.iter().map(canonical_binding).fold(Vec::new(), |mut acc, binding| {
        if !acc.contains(&binding) {
            acc.push(binding);
        }
        acc
    })
}

pub fn normalize_claim(claim: &OwnerClaim) -> OwnerClaim {
    let mut out = claim.iter().map(canonical_expr).fold(Vec::new(), |mut acc, expr| {
        if !acc.contains(&expr) {
            acc.push(expr);
        }
        acc
    });
    out.sort_by_key(|expr| format!("{expr:?}"));
    out
}

pub fn normalize_claims(claims: &[OwnerClaim]) -> Vec<OwnerClaim> {
    claims.iter().fold(Vec::new(), |mut acc, claim| {
        let claim = normalize_claim(claim);
        if !acc.contains(&claim) {
            acc.push(claim);
        }
        acc
    })
}

pub fn claims_root_names(claims: &[OwnerClaim]) -> HashSet<polars::prelude::PlSmallStr> {
    claims
        .iter()
        .flatten()
        .flat_map(|expr| canonical_expr(expr).meta().root_names())
        .collect::<HashSet<_>>()
}

pub fn bindings_root_names(bindings: &[Binding]) -> HashSet<polars::prelude::PlSmallStr> {
    bindings
        .iter()
        .flat_map(Binding::root_names)
        .collect::<HashSet<_>>()
}

pub fn unique_id_expr(bindings: &[Binding]) -> Fallible<Option<Expr>> {
    let sites = bindings.iter().filter(|site| !site.exprs.is_empty()).fold(
        Vec::<Binding>::new(),
        |mut acc, site| {
            if !acc.contains(site) {
                acc.push(site.clone());
            }
            acc
        },
    );

    match sites.as_slice() {
        [] => Ok(None),
        [site] => Ok(site.exprs.first().cloned()),
        _ => fallible!(
            MakeTransformation,
            "this operation currently supports at most one identifier site, but found {}",
            sites.len()
        ),
    }
}

pub fn filter_bindings(bindings: &[Binding], label: &str) -> Vec<Binding> {
    bindings
        .iter()
        .filter(|site| site.space == label)
        .cloned()
        .collect()
}

pub fn default_owner_claims(bindings: &[Binding], protect: &str) -> Vec<OwnerClaim> {
    normalize_claims(
        &filter_bindings(&normalize_bindings(bindings), protect)
            .into_iter()
            .flat_map(|binding| binding.exprs.into_iter().map(|expr| vec![expr]))
            .collect::<Vec<_>>(),
    )
}

pub fn expr_identifies_protected_id(bindings: &[Binding], protect: &str, expr: &Expr) -> bool {
    let expr = canonical_expr(expr);
    filter_bindings(&normalize_bindings(bindings), protect).into_iter().any(|binding| {
        binding
            .exprs
            .into_iter()
            .map(|candidate| canonical_expr(&candidate))
            .any(|candidate| candidate == expr)
    })
}

pub fn compile_database_id_distance(
    protect: String,
    bindings: HashMap<String, Vec<Binding>>,
    unique_keys: Vec<UniqueKey>,
    foreign_keys: Vec<ForeignKey>,
    functional_dependencies: Vec<FunctionalDependency>,
    ownerships: Vec<Ownership>,
) -> Fallible<DatabaseIdDistance> {
    let mut compiled = bindings
        .iter()
        .map(|(table, bindings)| (table.clone(), normalize_bindings(bindings)))
        .collect::<HashMap<_, _>>();

    let all_tables = bindings
        .keys()
        .cloned()
        .chain(unique_keys.iter().map(|u| u.table.clone()))
        .chain(foreign_keys.iter().map(|fk| fk.from_table.clone()))
        .chain(foreign_keys.iter().map(|fk| fk.to_table.clone()))
        .chain(functional_dependencies.iter().map(|fd| fd.table.clone()))
        .chain(ownerships.iter().map(|o| o.table.clone()))
        .collect::<HashSet<_>>();

    for table in all_tables {
        compiled.entry(table).or_default();
    }

    let unique_keys = unique_keys
        .into_iter()
        .map(|unique| (unique.table, canonical_expr(&unique.key)))
        .collect::<HashSet<_>>();
    let foreign_keys = foreign_keys
        .into_iter()
        .map(|fk| ForeignKey {
            from_table: fk.from_table,
            from: canonical_expr(&fk.from),
            to_table: fk.to_table,
            to: canonical_expr(&fk.to),
        })
        .collect::<Vec<_>>();
    let functional_dependencies = functional_dependencies
        .into_iter()
        .map(|fd| FunctionalDependency {
            table: fd.table,
            from: canonical_expr(&fd.from),
            to: canonical_expr(&fd.to),
        })
        .collect::<Vec<_>>();

    let mut changed = true;
    while changed {
        changed = false;
        let snapshot = compiled.clone();

        for fk in &foreign_keys {
            if !unique_keys.contains(&(fk.to_table.clone(), fk.to.clone())) {
                continue;
            }
            let Some(target_bindings) = snapshot.get(&fk.to_table) else {
                continue;
            };
            let Some(source_bindings) = compiled.get_mut(&fk.from_table) else {
                continue;
            };
            for binding in target_bindings
                .iter()
                .filter(|binding| binding.exprs.contains(&fk.to))
            {
                let candidate = canonical_binding(&Binding {
                    exprs: vec![fk.from.clone()],
                    space: binding.space.clone(),
                });
                if !source_bindings.contains(&candidate) {
                    source_bindings.push(candidate);
                    changed = true;
                }
            }
        }

        for fd in &functional_dependencies {
            let Some(table_bindings) = snapshot.get(&fd.table) else {
                continue;
            };
            let Some(output_bindings) = compiled.get_mut(&fd.table) else {
                continue;
            };
            for binding in table_bindings
                .iter()
                .filter(|binding| binding.exprs.contains(&fd.to))
            {
                let candidate = canonical_binding(&Binding {
                    exprs: vec![fd.from.clone()],
                    space: binding.space.clone(),
                });
                if !output_bindings.contains(&candidate) {
                    output_bindings.push(candidate);
                    changed = true;
                }
            }
        }
    }

    for bindings in compiled.values_mut() {
        *bindings = normalize_bindings(bindings);
    }

    let ownerships = ownerships
        .into_iter()
        .map(|ownership| (ownership.table, normalize_claims(&ownership.claims)))
        .collect::<HashMap<_, _>>();

    let mut base_owner_claims = HashMap::new();
    for (table, bindings) in &compiled {
        let claims = ownerships
            .get(table)
            .cloned()
            .unwrap_or_else(|| default_owner_claims(bindings, &protect));

        if claims.iter().flatten().any(|expr| !expr_identifies_protected_id(bindings, &protect, expr))
        {
            return fallible!(
                MakeMeasurement,
                "ownership declaration for table {} contains a factor that does not identify exactly one protected identifier",
                table
            );
        }
        base_owner_claims.insert(table.clone(), claims);
    }

    Ok(DatabaseIdDistance {
        protect,
        bindings: compiled,
        base_owner_claims,
    })
}

pub fn choose_owner_claim(claims: &[OwnerClaim]) -> Option<OwnerClaim> {
    normalize_claims(claims)
        .into_iter()
        .filter(|claim| !claim.is_empty())
        .min_by(|left, right| {
            left.len()
                .cmp(&right.len())
                .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
        })
}

pub trait PolarsMetric: MicrodataMetric {
    fn bindings(&self) -> Vec<Binding>;
    fn owner_claims(&self) -> Vec<OwnerClaim> {
        vec![]
    }
    fn protected_label(&self) -> Option<&str> {
        None
    }
    fn active_bindings(&self) -> Vec<Binding> {
        self.protected_label()
            .map(|label| filter_bindings(&self.bindings(), label))
            .unwrap_or_default()
    }
    fn active_id_sites(&self) -> Vec<Binding> {
        self.active_bindings()
    }
    fn rebuild_group_by_metric(&self, _keys: &[Expr]) -> Fallible<Self>
    where
        Self: Sized + Clone,
    {
        Ok(self.clone())
    }
}

/// Distance betweeen datasets in terms of the number of added or removed identifiers.
///
/// # Proof Definition
///
/// ## `d`-closeness
/// For any two datasets $x, x'$ and any $d$ of type [`IntDistance`],
/// we say that $x, x'$ are $d$-close under the identifier distance metric whenever
///
/// ```math
/// d_{SymId}(x, x') = |\mathrm{identifier}(x) \Delta \mathrm{identifier}(x')| \leq d
/// ```
///
/// In addition, for each identifier in both $x$ and $x'$,
/// the corresponding data must be equivalent for the datasets to be close.
#[derive(Clone, PartialEq, Debug)]
pub struct SymmetricIdDistance {
    pub protect: String,
    pub bindings: Vec<Binding>,
    pub owner_claims: Vec<OwnerClaim>,
}

impl Metric for SymmetricIdDistance {
    type Distance = IntDistance;
}

/// Distance betweeen datasets in terms of the number of changed identifiers.
///
/// # Proof Definition
///
/// ## `d`-closeness
/// For any two datasets $x, x'$ and any $d$ of type [`IntDistance`],
/// we say that $x, x'$ are $d$-close under the identifier distance metric whenever
///
/// ```math
/// d_{COId}(x, x') = d_{SymId}(ids(u), ids(v)) / 2 \leq d
/// ```
/// $d_{COId}$ is in reference to the [`ChangeOneIdDistance`].
///
/// In addition, for each identifier in both $x$ and $x'$,
/// the corresponding data must be equivalent for the datasets to be close.
#[derive(Clone, PartialEq, Debug)]
pub struct ChangeOneIdDistance {
    pub protect: String,
    pub bindings: Vec<Binding>,
    pub owner_claims: Vec<OwnerClaim>,
}

impl Metric for ChangeOneIdDistance {
    type Distance = IntDistance;
}

#[derive(Clone, PartialEq, Debug)]
pub struct DatabaseIdDistance {
    pub protect: String,
    pub bindings: HashMap<String, Vec<Binding>>,
    pub base_owner_claims: HashMap<String, Vec<OwnerClaim>>,
}

impl Metric for DatabaseIdDistance {
    type Distance = IntDistance;
}

/// Frame-distance betweeen datasets.
///
/// # Proof Definition
///
/// ## `d`-closeness
/// For any two datasets $x, x' \in \texttt{D}$,
/// and for each distance bound $d_i$ of type [`Bounds`],
/// let $\pi_{d_i}(x)$ and $\pi_{d_i}(x')$ be groupings of $x, x'$ with respect to $d_i$,
/// and let $\pi_{d_i}(x)_j$ denote the data in group j of $\pi_{d_i}(x)$.
///
/// Define a vector of per-group distances $s_i$ with respect to grouping $d_i$ as follows:
///
/// ```math
/// s_i = [d_M(\pi_{d_i}(x)_0, \pi_{d_i}(x')_0), \ldots, d_M(\pi_{d_i}(x)_r, \pi_{d_i}(x')_r)],
/// ```
///
/// Then $x, x'$ are $d_i$-close under the ith frame distance,
/// where $d_{Fi0}$ is ``num_groups`` and $d_{Fi\infty}$ is ``per_group``,
/// whenever,
///
/// ```math
/// |s_i|_0 \leq d_{Fi0} \land |s_i|_\infty \leq d_{Fi\infty}.
/// ```
///
/// Finally, for any two datasets $x, x'$ and any $d$ of type [`Bounds`],
/// we say that $x, x'$ are $d$-close under the frame distance metric whenever
/// $x, x'$ are $d_i$-close under the ith frame distance for all $d_i$ in $d$.
#[derive(Clone, PartialEq, Debug)]
pub struct FrameDistance<M: UnboundedMetric>(pub M);

impl MicrodataMetric for SymmetricIdDistance {
    const SIZED: bool = false;
    const ORDERED: bool = false;
    fn identifier(&self) -> Option<Expr> {
        unique_id_expr(&self.active_bindings()).ok().flatten()
    }
    type EventMetric = SymmetricDistance;
}
impl MicrodataMetric for ChangeOneIdDistance {
    const SIZED: bool = true;
    const ORDERED: bool = false;
    fn identifier(&self) -> Option<Expr> {
        unique_id_expr(&self.active_bindings()).ok().flatten()
    }
    type EventMetric = ChangeOneDistance;
}

impl PolarsMetric for SymmetricDistance {
    fn bindings(&self) -> Vec<Binding> {
        vec![]
    }
}
impl PolarsMetric for ChangeOneDistance {
    fn bindings(&self) -> Vec<Binding> {
        vec![]
    }
}
impl PolarsMetric for super::InsertDeleteDistance {
    fn bindings(&self) -> Vec<Binding> {
        vec![]
    }
}
impl PolarsMetric for super::HammingDistance {
    fn bindings(&self) -> Vec<Binding> {
        vec![]
    }
}
impl PolarsMetric for SymmetricIdDistance {
    fn bindings(&self) -> Vec<Binding> {
        self.bindings.clone()
    }
    fn owner_claims(&self) -> Vec<OwnerClaim> {
        self.owner_claims.clone()
    }
    fn protected_label(&self) -> Option<&str> {
        Some(&self.protect)
    }
    fn rebuild_group_by_metric(&self, keys: &[Expr]) -> Fallible<Self> {
        let keys = keys.iter().map(canonical_expr).collect::<Vec<_>>();
        let surviving_bindings = self
            .bindings
            .iter()
            .filter_map(|binding| {
                let exprs = binding
                    .exprs
                    .iter()
                    .map(canonical_expr)
                    .filter(|expr| keys.contains(expr))
                    .collect::<Vec<_>>();
                if exprs.is_empty() {
                    None
                } else {
                    Some(Binding {
                        space: binding.space.clone(),
                        exprs,
                    })
                }
            })
            .collect::<Vec<_>>();

        let has_singleton_owner_claim = self
            .owner_claims
            .iter()
            .any(|claim| normalize_claim(claim).len() == 1);

        let mut rebuilt_claims = self
            .owner_claims
            .iter()
            .filter_map(|claim| {
                let claim = normalize_claim(claim);
                if claim.is_empty() {
                    return Some(claim);
                }
                if claim.iter().all(|expr| keys.contains(expr)) {
                    Some(claim)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if has_singleton_owner_claim {
            rebuilt_claims.extend(
                keys.iter()
                    .filter(|expr| expr_identifies_protected_id(&self.bindings, &self.protect, expr))
                    .cloned()
                    .map(|expr| vec![expr]),
            );
        }

        let surviving_claims = normalize_claims(&rebuilt_claims);

        if !self.owner_claims.is_empty() && choose_owner_claim(&surviving_claims).is_none() {
            return fallible!(
                MakeTransformation,
                "group_by keys must preserve at least one complete protected owner claim"
            );
        }

        Ok(Self {
            protect: self.protect.clone(),
            bindings: surviving_bindings,
            owner_claims: surviving_claims,
        })
    }
}
impl PolarsMetric for ChangeOneIdDistance {
    fn bindings(&self) -> Vec<Binding> {
        self.bindings.clone()
    }
    fn owner_claims(&self) -> Vec<OwnerClaim> {
        self.owner_claims.clone()
    }
    fn protected_label(&self) -> Option<&str> {
        Some(&self.protect)
    }
    fn rebuild_group_by_metric(&self, keys: &[Expr]) -> Fallible<Self> {
        let rebuilt = SymmetricIdDistance {
            protect: self.protect.clone(),
            bindings: self.bindings.clone(),
            owner_claims: self.owner_claims.clone(),
        }
        .rebuild_group_by_metric(keys)?;
        Ok(Self {
            protect: rebuilt.protect,
            bindings: rebuilt.bindings,
            owner_claims: rebuilt.owner_claims,
        })
    }
}

impl<M: UnboundedMetric> Metric for FrameDistance<M> {
    type Distance = Bounds;
}

#[derive(Clone, PartialEq, Debug)]
pub struct Bounds(pub Vec<Bound>);

impl From<u32> for Bounds {
    fn from(v: u32) -> Self {
        Self(vec![Bound::by::<[Expr; 0], Expr>([]).with_per_group(v)])
    }
}

impl Bounds {
    /// # Proof Definition
    /// Return a `Bound` with given `by` implied by `self`.
    pub fn get_bound(&self, by: &HashSet<Expr>) -> Bound {
        let mut bound = (self.0.iter())
            .find(|b| &b.by == by)
            .cloned()
            .unwrap_or_else(|| Bound {
                by: by.clone(),
                ..Default::default()
            });

        let subset_bounds = (self.0.iter())
            .filter(|m| m.by.is_subset(by))
            .collect::<Vec<&Bound>>();

        // max per-group contributions is the fewest per-group contributions
        // of any grouping as coarse or coarser than the current grouping
        bound.per_group = (subset_bounds.iter()).filter_map(|m| m.per_group).min();

        let all_mips = (self.0.iter())
            .filter_map(|b| Some((&b.by, b.num_groups?)))
            .collect();

        // in the worst case, the max num-group contributions is the product of the max num-group contributions of the cover
        bound.num_groups = find_min_covering(by.clone(), all_mips)
            .map(|cover| {
                cover
                    .iter()
                    .try_fold(1u32, |acc, (_, v)| acc.inf_mul(v).ok())
            })
            .flatten();

        if by.is_empty() {
            bound.num_groups = Some(1);
        }

        bound
    }

    pub fn with_bound(mut self, bound: Bound) -> Self {
        if let Some(b) = self.0.iter_mut().find(|m| m.by == bound.by) {
            b.num_groups = option_min(b.num_groups, bound.num_groups);
            b.per_group = option_min(b.per_group, bound.per_group);
        } else {
            self.0.push(bound);
        }
        self
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Bound {
    /// The columns to group by.
    pub by: HashSet<Expr>,

    /// The greatest number of contributions that can be made by one unit to any one group.
    pub per_group: Option<u32>,

    /// The greatest number of groups that can be contributed.
    pub num_groups: Option<u32>,
}

impl Bound {
    pub fn by<E: AsRef<[IE]>, IE: Into<Expr> + Clone>(by: E) -> Self {
        Self {
            by: by.as_ref().iter().cloned().map(Into::into).collect(),
            per_group: None,
            num_groups: None,
        }
    }

    pub fn with_per_group(mut self, value: u32) -> Self {
        self.per_group = Some(value);
        self
    }
    pub fn with_num_groups(mut self, value: u32) -> Self {
        self.num_groups = Some(value);
        self
    }
}

impl ProductOrd for Bounds {
    fn total_cmp(&self, other: &Self) -> Fallible<Ordering> {
        if self != other {
            return fallible!(
                MakeTransformation,
                "cannot compare bounds with different by columns"
            );
        }
        Ok(Ordering::Equal)
    }
}
