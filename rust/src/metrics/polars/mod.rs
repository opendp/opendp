use std::{cmp::Ordering, collections::HashSet};

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
    pub identifier: Expr,
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
    pub identifier: Expr,
}

impl Metric for ChangeOneIdDistance {
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
        Some(self.identifier.clone())
    }
    type EventMetric = SymmetricDistance;
}
impl MicrodataMetric for ChangeOneIdDistance {
    const SIZED: bool = true;
    const ORDERED: bool = false;
    fn identifier(&self) -> Option<Expr> {
        Some(self.identifier.clone())
    }
    type EventMetric = ChangeOneDistance;
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
