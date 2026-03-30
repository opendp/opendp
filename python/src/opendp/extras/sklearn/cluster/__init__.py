'''
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn.cluster``.    

See also our :ref:`tutorial on diffentially private clustering <dp-clustering>`.
'''

from __future__ import annotations

from opendp.extras.sklearn.cluster._tree import Node, Quadtree, make_private_tree
from opendp.extras.sklearn.cluster._postprocess import solve_tree, postprocess_children

from dataclasses import dataclass
from typing import TYPE_CHECKING, ClassVar, Literal, cast
import opendp.prelude as dp
from opendp._internal import _new_pure_function
from opendp._lib import import_optional_dependency

if TYPE_CHECKING:  # pragma: no cover
    import numpy


def _np():
    return import_optional_dependency("numpy")


class _BaseTreeCluster:
    objective: ClassVar[str | None] = None

    def __init__(
        self,
        n_features: int,
        n_clusters: int = 8,
        *,
        scale: float,
        max_depth: int | None = None,
        level_allocation=None,
        lower,
        upper,
        output_measure=None,
        input_domain=None,
        input_metric=None,
    ):
        if self.objective is None:
            raise TypeError("_BaseTreeCluster must be subclassed")  # pragma: no cover
        if (max_depth is None) == (level_allocation is None):
            raise ValueError("set exactly one of max_depth or level_allocation")
        if max_depth is not None and max_depth <= 0:
            raise ValueError("max_depth must be positive")
        if scale <= 0:
            raise ValueError("scale must be positive")
        if n_clusters <= 0:
            raise ValueError("n_clusters must be positive")

        np = _np()
        self.n_features = n_features
        self.n_clusters = n_clusters
        self.scale = float(scale)
        self.max_depth = max_depth
        self.level_allocation = (
            np.ones(max_depth, dtype=float)
            if level_allocation is None
            else np.asarray(level_allocation, dtype=float)
        )
        self.lower = np.asarray(lower, dtype=float)
        self.upper = np.asarray(upper, dtype=float)
        self.output_measure = (
            dp.max_divergence() if output_measure is None else output_measure
        )
        self.input_domain = (
            dp.numpy.array2_domain(T=float, num_columns=n_features)
            if input_domain is None
            else input_domain
        )
        self.input_metric = (
            dp.symmetric_distance() if input_metric is None else input_metric
        )

        self.measurement_ = make_private_tree_cluster(
            self.input_domain,
            self.input_metric,
            self.output_measure,
            splitter=Quadtree(self.lower, self.upper),
            level_allocation=self.level_allocation,
            scale=self.scale,
            n_clusters=self.n_clusters,
            objective=self.objective,
        )

        self.cluster_centers_ = None
        self.inertia_ = None
        self.labels_ = None
        self.n_iter_ = 1
        self.n_features_in_ = n_features
        self.feature_names_in_ = None
        self.tree_ = None
        self.consistent_tree_ = None

    def fit(self, X, y=None, sample_weight=None):
        if sample_weight is not None:
            raise NotImplementedError("sample_weight is not supported")
        np = _np()
        X = np.asarray(X, dtype=float)
        if X.ndim != 2:
            raise ValueError(f"expected 2D array, got shape {X.shape}")
        if X.shape[1] != self.n_features:
            raise ValueError(f"expected {self.n_features} features, got {X.shape[1]}")

        release = self.measurement_(X)
        self.cluster_centers_ = release.cluster_centers_
        self.inertia_ = release.inertia_
        self.tree_ = release.tree_
        self.consistent_tree_ = release.consistent_tree_
        self.labels_ = None
        return self

    def predict(self, X):
        if self.cluster_centers_ is None:
            raise ValueError("model has not been fitted")
        np = _np()
        X = np.asarray(X, dtype=float)
        if X.ndim != 2:
            raise ValueError(f"expected 2D array, got shape {X.shape}")
        if X.shape[1] != self.n_features:
            raise ValueError(f"expected {self.n_features} features, got {X.shape[1]}")
        dists = self._pairwise_distance(X, self.cluster_centers_)
        return np.argmin(dists, axis=1)

    def fit_predict(self, X, y=None, sample_weight=None):
        self.fit(X, y=y, sample_weight=sample_weight)
        return self.predict(X)

    def transform(self, X):
        if self.cluster_centers_ is None:
            raise ValueError("model has not been fitted")
        np = _np()
        X = np.asarray(X, dtype=float)
        if X.ndim != 2:
            raise ValueError(f"expected 2D array, got shape {X.shape}")
        if X.shape[1] != self.n_features:
            raise ValueError(f"expected {self.n_features} features, got {X.shape[1]}")
        return self._pairwise_distance(X, self.cluster_centers_)

    def fit_transform(self, X, y=None, sample_weight=None):
        self.fit(X, y=y, sample_weight=sample_weight)
        return self.transform(X)

    def score(self, X, y=None, sample_weight=None):
        if sample_weight is not None:
            raise NotImplementedError("sample_weight is not supported")
        np = _np()
        return -float(np.sum(np.min(self.transform(X), axis=1)))


class KMeans(_BaseTreeCluster):
    """Differentially private tree-based k-means clustering.

    This implementation is based on hierarchical tree releases inspired by
    *Scalable Differentially Private Clustering via Hierarchically Separated
    Trees* (Cohen-Addad et al., KDD 2022, DOI: 10.1145/3534678.3539409).

    Set exactly one of ``max_depth`` or ``level_allocation``.

    :param n_features: Number of columns in the input data.
    :param n_clusters: Number of cluster centers to release.
    :param scale: Global, measure-agnostic privacy knob for noise scale.
    :param max_depth: Uniformly allocate privacy budget across this many tree levels.
    :param level_allocation: Relative privacy-budget allocation across tree levels.
    :param lower: Lower corner of the quadtree bounding box.
    :param upper: Upper corner of the quadtree bounding box.
    :param output_measure: Privacy measure for the internal tree release.
    :param input_domain: Optional explicit domain for the input data.
    :param input_metric: Optional explicit metric for the input data.
    """
    objective = "kmeans"

    def _pairwise_distance(self, X, centers):
        np = _np()
        diff = X[:, None, :] - centers[None, :, :]
        return np.sum(diff * diff, axis=2)


class KMedians(_BaseTreeCluster):
    """Differentially private tree-based k-medians clustering.

    This implementation is based on hierarchical tree releases inspired by
    *Scalable Differentially Private Clustering via Hierarchically Separated
    Trees* (Cohen-Addad et al., KDD 2022, DOI: 10.1145/3534678.3539409).

    Set exactly one of ``max_depth`` or ``level_allocation``.

    :param n_features: Number of columns in the input data.
    :param n_clusters: Number of cluster centers to release.
    :param scale: Global, measure-agnostic privacy knob for noise scale.
    :param max_depth: Uniformly allocate privacy budget across this many tree levels.
    :param level_allocation: Relative privacy-budget allocation across tree levels.
    :param lower: Lower corner of the quadtree bounding box.
    :param upper: Upper corner of the quadtree bounding box.
    :param output_measure: Privacy measure for the internal tree release.
    :param input_domain: Optional explicit domain for the input data.
    :param input_metric: Optional explicit metric for the input data.
    """
    objective = "kmedians"

    def _pairwise_distance(self, X, centers):
        np = _np()
        return np.abs(X[:, None, :] - centers[None, :, :]).sum(axis=2)


@dataclass
class ClusterRelease:
    """Differentially private tree-based clustering outputs."""
    tree_: list[Node]
    consistent_tree_: list[Node]
    cluster_centers_: "numpy.ndarray"
    inertia_: float
    objective_: str


def make_private_tree_cluster(
    input_domain,
    input_metric,
    output_measure,
    *,
    splitter: Quadtree,
    level_allocation,
    scale: float,
    n_clusters: int,
    objective: str,
):
    """Construct a private tree-clustering measurement.

    This constructor uses a hierarchical tree release in the style of
    *Scalable Differentially Private Clustering via Hierarchically Separated
    Trees* (Cohen-Addad et al., KDD 2022, DOI: 10.1145/3534678.3539409).

    :param input_domain: Domain of the input dataset.
    :param input_metric: Metric on the input dataset.
    :param output_measure: Privacy measure for the tree release.
    :param splitter: Quadtree splitter defining the recursive partition.
    :param level_allocation: Relative privacy-budget allocation across tree levels.
    :param scale: Global privacy-control knob. Smaller values spend more privacy.
    :param n_clusters: Number of cluster centers to release.
    :param objective: Clustering objective. Must be ``"kmeans"`` or ``"kmedians"``.
    """
    if objective not in {"kmeans", "kmedians"}:
        raise ValueError("objective must be 'kmeans' or 'kmedians'")
    if not isinstance(splitter, Quadtree):
        raise NotImplementedError(
            "tree clustering currently requires a Quadtree splitter"
        )
    if n_clusters <= 0:
        raise ValueError("n_clusters must be positive")

    tree = make_private_tree(
        input_domain,
        input_metric,
        output_measure,
        splitter=splitter,
        level_allocation=level_allocation,
        scale=scale,
        objective=cast(Literal["kmeans", "kmedians"], objective),
    )

    def postprocess(children: list[Node]) -> ClusterRelease:
        np = _np()
        consistent = postprocess_children(children)
        costs, centers = solve_tree(
            consistent, splitter, n_clusters, objective=objective
        )
        cluster_centers = np.asarray(centers[n_clusters], dtype=float)
        inertia = float(costs[n_clusters])
        return ClusterRelease(
            tree_=children,
            consistent_tree_=consistent,
            cluster_centers_=cluster_centers,
            inertia_=inertia,
            objective_=objective,
        )

    return tree >> _new_pure_function(postprocess, TO="ExtrinsicObject")
