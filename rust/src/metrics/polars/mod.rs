use std::{cmp::Ordering, collections::HashSet};

use polars::prelude::Expr;

use crate::{
    core::Metric,
    domains::{find_min_covering, option_min},
    error::Fallible,
    traits::{InfMul, ProductOrd},
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
/// d_{ID}(x, x') = |identifier(x) \Delta identifier(x')| \leq d
/// ```
///
/// In addition, if the data in $x$ and $x'$ corresponding to any one identifier differs,
/// then the datasets are not close.
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
/// $d_{SymId}$ is in reference to the [`ChangeOneIdDistance`].
///
/// In addition, if the data in $x$ and $x'$ corresponding to any one identifier differs,
/// then the datasets are not close.
#[derive(Clone, PartialEq, Debug)]
pub struct ChangeOneIdDistance {
    pub identifier: Expr,
}

impl Metric for ChangeOneIdDistance {
    type Distance = IntDistance;
}

/// Multi-distance betweeen datasets in terms of the number of added or removed identifiers.
///
/// # Proof Definition
///
/// ## `d`-closeness
/// For any two datasets $x, x'$ and any $d$ of type [`IntDistance`],
/// we say that $x, x'$ are $d$-close under the identifier distance metric whenever
///
/// ```math
/// d_{ID}(x, x') = |identifier(x) \Delta identifier(x')| \leq d
/// ```
///
/// In addition, if the data in $x$ and $x'$ corresponding to any one identifier differs,
/// then the datasets are not close.
#[derive(Clone, PartialEq, Debug)]
pub struct Multi<M: UnboundedMetric>(pub M);

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

impl<M: UnboundedMetric> Metric for Multi<M> {
    type Distance = GroupBounds;
}

#[derive(Clone, PartialEq, Debug)]
pub struct GroupBounds(pub Vec<GroupBound>);

impl From<u32> for GroupBounds {
    fn from(v: u32) -> Self {
        Self(vec![
            GroupBound::by::<[Expr; 0], Expr>([]).with_max_partition_contributions(v),
        ])
    }
}

impl GroupBounds {
    /// # Proof Definition
    /// Return a `GroupBound` with given `by` implied by `self`.
    pub fn get_bound(&self, by: &HashSet<Expr>) -> GroupBound {
        let mut bound = (self.0.iter())
            .find(|b| &b.by == by)
            .cloned()
            .unwrap_or_else(|| GroupBound {
                by: by.clone(),
                ..Default::default()
            });

        let subset_bounds = (self.0.iter())
            .filter(|m| m.by.is_subset(by))
            .collect::<Vec<&GroupBound>>();

        // max partition contributions is the fewest partition contributions
        // of any grouping as coarse or coarser than the current grouping
        bound.max_partition_contributions = (subset_bounds.iter())
            .filter_map(|m| m.max_partition_contributions)
            .min();

        let all_mips = (self.0.iter())
            .filter_map(|b| Some((&b.by, b.max_influenced_partitions?)))
            .collect();

        // in the worst case, the max partition contributions is the product of the max partition contributions of the cover
        bound.max_influenced_partitions = find_min_covering(by.clone(), all_mips)
            .map(|cover| {
                cover
                    .iter()
                    .try_fold(1u32, |acc, (_, v)| acc.inf_mul(v).ok())
            })
            .flatten();

        if by.is_empty() {
            bound.max_influenced_partitions = Some(1);
        }

        bound
    }

    pub fn with_bound(mut self, bound: GroupBound) -> Self {
        if let Some(b) = self.0.iter_mut().find(|m| m.by == bound.by) {
            b.max_influenced_partitions =
                option_min(b.max_influenced_partitions, bound.max_influenced_partitions);
            b.max_partition_contributions = option_min(
                b.max_partition_contributions,
                bound.max_partition_contributions,
            );
        } else {
            self.0.push(bound);
        }
        self
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct GroupBound {
    /// The columns data is grouped by to partition the data into groups.
    pub by: HashSet<Expr>,

    /// The greatest number of contributions that can be made by one unit to any one partition.
    ///
    /// This affects how margins interact with the metric.
    /// The distance between data sets differing by more than this quantity is considered infinite.
    pub max_partition_contributions: Option<u32>,

    /// The greatest number of partitions that can be contributed to.
    ///
    /// This affects how margins interact with the metric.
    /// The distance between data sets differing by more than this quantity is considered infinite.
    pub max_influenced_partitions: Option<u32>,
}

impl GroupBound {
    pub fn by<E: AsRef<[IE]>, IE: Into<Expr> + Clone>(by: E) -> Self {
        Self {
            by: by.as_ref().iter().cloned().map(Into::into).collect(),
            max_partition_contributions: None,
            max_influenced_partitions: None,
        }
    }

    pub fn with_max_partition_contributions(mut self, value: u32) -> Self {
        self.max_partition_contributions = Some(value);
        self
    }
    pub fn with_max_influenced_partitions(mut self, value: u32) -> Self {
        self.max_influenced_partitions = Some(value);
        self
    }
}

impl PartialOrd for GroupBounds {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.total_cmp(other).ok()
    }
}

impl ProductOrd for GroupBounds {
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
