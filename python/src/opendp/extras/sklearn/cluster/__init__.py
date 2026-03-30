"""
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn.cluster``.

See also our :ref:`tutorial on diffentially private clustering <dp-clustering>`.
"""

from __future__ import annotations

from opendp.extras.sklearn.cluster._tree import Quadtree, make_private_tree
from opendp.extras.sklearn.cluster._postprocess import (
    estimate_group_sizes,
    estimate_silhouette_score,
    postprocess_children,
    solve_tree,
)

from typing import TYPE_CHECKING, ClassVar, Literal, cast
import opendp.prelude as dp
from opendp._internal import _make_transformation, _new_pure_function
from opendp._lib import import_optional_dependency

if TYPE_CHECKING:  # pragma: no cover
    import numpy
    from opendp.extras.sklearn.cluster._tree import Node


def _np():
    return import_optional_dependency("numpy")


class _BaseTreeCluster:
    objective: ClassVar[Literal["kmeans", "kmedians"] | None] = None

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

        self.splitter = Quadtree(self.lower, self.upper)
        self.cluster_centers_: numpy.ndarray | None = None
        self.inertia_: float | None = None
        self.labels_ = None
        self.n_iter_ = 1
        self.n_features_in_ = n_features
        self.feature_names_in_ = None
        self.tree_: list[Node] | None = None
        self.consistent_tree_: list[Node] | None = None
        self.labels_ = None

        def postprocess(tree):
            np = _np()
            consistent = postprocess_children(tree)
            costs, centers = solve_tree(
                consistent,
                self.splitter,
                self.n_clusters,
                objective=cast(Literal["kmeans", "kmedians"], self.objective),
            )
            self.tree_ = tree
            self.consistent_tree_ = consistent
            self.cluster_centers_ = np.asarray(centers[self.n_clusters], dtype=float)
            self.inertia_ = float(costs[self.n_clusters])
            self.objective_ = self.objective
            self.labels_ = None
            return self

        self.measurement_ = make_private_tree(
            self.input_domain,
            self.input_metric,
            self.output_measure,
            splitter=self.splitter,
            level_allocation=self.level_allocation,
            scale=self.scale,
            objective=cast(Literal["kmeans", "kmedians"], self.objective),
        ) >> _new_pure_function(postprocess)

    def _coerce_X(self, X):
        np = _np()
        X = np.asarray(X, dtype=float)
        if X.ndim != 2:
            raise ValueError(f"expected 2D array, got shape {X.shape}")
        if X.shape[1] != self.n_features:
            raise ValueError(f"expected {self.n_features} features, got {X.shape[1]}")
        return X

    def fit(self, X, y=None, sample_weight=None):
        if sample_weight is not None:
            raise NotImplementedError("sample_weight is not supported")
        X = self._coerce_X(X)
        return self.measurement_(X)

    @property
    def transform(self):
        if self.cluster_centers_ is None:
            raise ValueError("model has not been fitted")
        centers = self.cluster_centers_

        def pairwise_distances(X):
            np = _np()
            X = self._coerce_X(X)
            if self.objective == "kmedians":
                return np.abs(X[:, None, :] - centers[None, :, :]).sum(
                    axis=2
                )
            diff = X[:, None, :] - centers[None, :, :]
            return np.sum(diff * diff, axis=2)

        return _make_transformation(
            input_domain=self.input_domain,
            input_metric=self.input_metric,
            output_domain=dp.numpy.array2_domain(
                T=float,
                num_columns=len(centers),
            ),
            output_metric=dp.symmetric_distance(),
            function=pairwise_distances,
            stability_map=lambda d_in: d_in,
        )

    @property
    def predict(self):
        np = _np()
        if self.cluster_centers_ is None:
            raise ValueError("model has not been fitted")
        return self.transform >> _make_transformation(
            input_domain=dp.numpy.array2_domain(
                T=float,
                num_columns=len(self.cluster_centers_),
            ),
            input_metric=dp.symmetric_distance(),
            output_domain=dp.vector_domain(dp.atom_domain(T=dp.i32)),
            output_metric=dp.symmetric_distance(),
            function=lambda dists: np.argmin(dists, axis=1).astype(np.int32),
            stability_map=lambda d_in: d_in,
        )

    def fit_predict(self, X, y=None, sample_weight=None):
        self.fit(X, y=y, sample_weight=sample_weight)
        return self.predict(X)

    def fit_transform(self, X, y=None, sample_weight=None):
        self.fit(X, y=y, sample_weight=sample_weight)
        return self.transform(X)

    def score(self, X, y=None, sample_weight=None):
        if sample_weight is not None:
            raise NotImplementedError("sample_weight is not supported")
        np = _np()
        return -float(np.sum(np.min(self.transform(X), axis=1)))

    def group_sizes(self):
        if self.consistent_tree_ is None or self.cluster_centers_ is None:
            raise ValueError("model has not been fitted")
        return estimate_group_sizes(
            self.consistent_tree_,
            self.splitter,
            self.cluster_centers_,
            objective=cast(str, self.objective_),
        )

    def silhouette_score(self):
        if self.consistent_tree_ is None or self.cluster_centers_ is None:
            raise ValueError("model has not been fitted")
        return estimate_silhouette_score(
            self.consistent_tree_,
            self.splitter,
            self.cluster_centers_,
            objective=cast(str, self.objective_),
        )


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
