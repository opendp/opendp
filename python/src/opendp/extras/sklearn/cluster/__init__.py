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

from opendp.extras.sklearn.cluster._refine import make_private_lloyd_refinement
from opendp.extras.sklearn.cluster._tree import (
    Quadtree,
    _budget_with_slack,
    _budget_without_slack,
    _noise_scale_for_budget,
    _privacy_budget_for_noise_scale,
    make_private_tree,
)
from opendp.extras.sklearn.cluster._postprocess import (
    estimate_group_sizes,
    estimate_silhouette_score,
    estimate_tree_inertia,
    solve_weighted_coreset_kmeans,
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


def _is_boolean_input_domain(input_domain) -> bool:
    """Return ``True`` when the input domain is a boolean NumPy array domain."""
    if not str(input_domain).startswith("NPArray2Domain"):
        return False
    return input_domain.descriptor.T == "bool"


def _resolve_random_shift(
    *,
    random_shift: bool | Literal["auto"],
    input_domain,
) -> bool:
    """Resolve whether to apply public random quadtree shifting."""
    is_boolean = _is_boolean_input_domain(input_domain)
    if random_shift == "auto":
        return not is_boolean
    if random_shift is False:
        return False
    if random_shift is True:
        if is_boolean:
            raise ValueError(
                "random_shift=True is not supported for boolean input domains"
            )
        return True
    raise ValueError("random_shift must be one of False, True, or 'auto'")


def _make_shifted_quadtree(lower, upper, *, random_state: int | None) -> Quadtree:
    """Construct a publicly shifted quadtree that contains the original box."""
    np = _np()
    lower = np.asarray(lower, dtype=float)
    upper = np.asarray(upper, dtype=float)
    width = upper - lower
    if np.any(width < 0):
        raise ValueError("upper must be greater than or equal to lower")

    rng = np.random.default_rng(random_state)
    offset = np.where(
        width > 0,
        rng.uniform(0.0, 1.0, size=width.shape) * width,
        0.0,
    )
    shifted_lower = lower - offset
    shifted_upper = np.where(width > 0, shifted_lower + 2.0 * width, upper)
    return Quadtree(shifted_lower, shifted_upper)


def _input_domain_dtype(input_domain):
    """Return a NumPy dtype implied by an ``NPArray2Domain`` when available."""
    if not str(input_domain).startswith("NPArray2Domain"):
        return None

    np = _np()
    runtime_type = str(input_domain.descriptor.T)
    runtime_to_dtype = {
        "bool": np.bool_,
        "f32": np.float32,
        "f64": np.float64,
        "i8": np.int8,
        "i16": np.int16,
        "i32": np.int32,
        "i64": np.int64,
        "u8": np.uint8,
        "u16": np.uint16,
        "u32": np.uint32,
        "u64": np.uint64,
    }
    return runtime_to_dtype.get(runtime_type)


def _is_float_input_domain(input_domain) -> bool:
    if not str(input_domain).startswith("NPArray2Domain"):
        return False
    return str(input_domain.descriptor.T) in {"f32", "f64"}


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
        select_branch_fraction: float = 0.0,
        select_branch_max_depth: int | None = None,
        random_shift: bool | Literal["auto"] = "auto",
        random_state: int | None = None,
        refinement: Literal["none", "dp_lloyd"] = "none",
        refinement_iters: int = 1,
        refinement_fraction: float = 0.0,
        refinement_count_fraction: float = 0.2,
        refinement_min_count: float | Literal["auto"] = "auto",
        init_solver: Literal["tree_dp", "weighted_coreset"] = "tree_dp",
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
        if not (0.0 <= float(select_branch_fraction) < 1.0):
            raise ValueError("select_branch_fraction must be in [0, 1)")
        if select_branch_max_depth is not None and select_branch_max_depth <= 0:
            raise ValueError("select_branch_max_depth must be positive when set")

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
        self.select_branch_fraction = float(select_branch_fraction)
        self.select_branch_max_depth = select_branch_max_depth
        self.random_shift = random_shift
        self.random_state = random_state
        self.refinement = refinement
        self.refinement_iters = int(refinement_iters)
        self.refinement_fraction = float(refinement_fraction)
        self.refinement_count_fraction = float(refinement_count_fraction)
        self.refinement_min_count = refinement_min_count
        self.init_solver = init_solver

        if refinement not in {"none", "dp_lloyd"}:
            raise ValueError("refinement must be 'none' or 'dp_lloyd'")
        if init_solver not in {"tree_dp", "weighted_coreset"}:
            raise ValueError("init_solver must be 'tree_dp' or 'weighted_coreset'")
        if init_solver == "weighted_coreset" and self.objective != "kmeans":
            raise ValueError(
                "init_solver='weighted_coreset' is only supported for kmeans"
            )
        if refinement == "dp_lloyd":
            if self.objective != "kmeans":
                raise ValueError("refinement='dp_lloyd' is only supported for kmeans")
            if self.refinement_iters <= 0:
                raise ValueError("refinement_iters must be positive")
            if not (0.0 < self.refinement_fraction < 1.0):
                raise ValueError("refinement_fraction must be in (0, 1)")
            if not (0.0 < self.refinement_count_fraction < 1.0):
                raise ValueError("refinement_count_fraction must be in (0, 1)")
            if (
                self.refinement_min_count != "auto"
                and float(self.refinement_min_count) <= 0.0
            ):
                raise ValueError("refinement_min_count must be positive or 'auto'")
            if not _is_float_input_domain(self.input_domain):
                raise ValueError(
                    "refinement='dp_lloyd' currently requires a floating-point input domain"
                )
            if self.input_metric != dp.symmetric_distance():
                raise ValueError(
                    "refinement='dp_lloyd' currently requires symmetric_distance()"
                )

        should_shift = _resolve_random_shift(
            random_shift=random_shift,
            input_domain=self.input_domain,
        )
        self.splitter = (
            _make_shifted_quadtree(
                self.lower,
                self.upper,
                random_state=random_state,
            )
            if should_shift
            else Quadtree(self.lower, self.upper)
        )
        self.quadtree_offset_ = self.lower - self.splitter.lower
        self.tree_lower_ = self.splitter.lower
        self.tree_upper_ = self.splitter.upper
        self.cluster_centers_: numpy.ndarray | None = None
        self.inertia_: float | None = None
        self.labels_ = None
        self.n_iter_ = 1
        self.n_features_in_ = n_features
        self.feature_names_in_ = None
        self.tree_: list[Node] | None = None
        self.consistent_tree_: list[Node] | None = None
        self.refinement_enabled_ = False
        self.refinement_iters_ = 0
        self.refinement_counts_ = None
        self.labels_ = None

        def solve_private_tree(tree):
            np = _np()
            consistent = postprocess_children(tree)
            if self.init_solver == "weighted_coreset":
                inertia, centers = solve_weighted_coreset_kmeans(
                    consistent,
                    self.splitter,
                    self.n_clusters,
                    lower=self.lower,
                    upper=self.upper,
                    random_state=self.random_state,
                )
                return consistent, float(inertia), np.clip(
                    np.asarray(centers, dtype=float),
                    self.lower,
                    self.upper,
                )
            costs, centers = solve_tree(
                consistent,
                self.splitter,
                self.n_clusters,
                objective=cast(Literal["kmeans", "kmedians"], self.objective),
            )
            return consistent, float(costs[self.n_clusters]), np.clip(
                np.asarray(centers[self.n_clusters], dtype=float),
                self.lower,
                self.upper,
            )

        def store_release(tree, consistent, cluster_centers, inertia):
            self.tree_ = tree
            self.consistent_tree_ = consistent
            self.cluster_centers_ = cluster_centers
            self.inertia_ = inertia
            self.objective_ = self.objective
            self.labels_ = None
            return self

        total_budget = _privacy_budget_for_noise_scale(self.output_measure, self.scale)
        tree_budget = (
            total_budget
            if refinement == "none"
            else (1.0 - self.refinement_fraction) * total_budget
        )
        tree_scale = (
            self.scale
            if refinement == "none"
            else _noise_scale_for_budget(
                self.output_measure,
                _budget_without_slack(tree_budget),
            )
        )
        m_tree = make_private_tree(
            self.input_domain,
            self.input_metric,
            self.output_measure,
            splitter=self.splitter,
            level_allocation=self.level_allocation,
            scale=tree_scale,
            objective=cast(Literal["kmeans", "kmedians"], self.objective),
            select_branch_fraction=self.select_branch_fraction,
            select_branch_max_depth=self.select_branch_max_depth,
        )

        if refinement == "none":
            def postprocess_tree_only(tree):
                consistent, inertia, cluster_centers = solve_private_tree(tree)
                self.refinement_enabled_ = False
                self.refinement_iters_ = 0
                self.refinement_counts_ = None
                self.n_iter_ = 1
                return store_release(tree, consistent, cluster_centers, inertia)

            self.measurement_ = m_tree >> _new_pure_function(postprocess_tree_only)
        else:
            refinement_budget = self.refinement_fraction * total_budget
            per_iter_budget = refinement_budget / float(self.refinement_iters)
            count_budget = self.refinement_count_fraction * per_iter_budget
            sum_budget = (1.0 - self.refinement_count_fraction) * per_iter_budget
            prototype_centers = _np().tile(
                0.5 * (self.lower + self.upper),
                (self.n_clusters, 1),
            )
            m_refine_prototype = make_private_lloyd_refinement(
                self.input_domain,
                self.input_metric,
                self.output_measure,
                initial_centers=prototype_centers,
                lower=self.lower,
                upper=self.upper,
                refinement_iters=self.refinement_iters,
                count_budget=count_budget,
                sum_budget=sum_budget,
                refinement_min_count=self.refinement_min_count,
            )
            comp = dp.c.make_adaptive_composition(
                input_domain=self.input_domain,
                input_metric=self.input_metric,
                output_measure=self.output_measure,
                d_in=1,
                d_mids=[
                    _budget_with_slack(m_tree.map(1)),
                    _budget_with_slack(m_refine_prototype.map(1)),
                ],
            )

            def postprocess_tree_and_refine(qbl):
                tree = qbl(m_tree)
                consistent, _, cluster_centers = solve_private_tree(tree)
                refinement_release = qbl(
                    make_private_lloyd_refinement(
                        self.input_domain,
                        self.input_metric,
                        self.output_measure,
                        initial_centers=cluster_centers,
                        lower=self.lower,
                        upper=self.upper,
                        refinement_iters=self.refinement_iters,
                        count_budget=count_budget,
                        sum_budget=sum_budget,
                        refinement_min_count=self.refinement_min_count,
                    )
                )
                self.refinement_enabled_ = True
                self.refinement_iters_ = refinement_release.iters
                self.refinement_counts_ = refinement_release.counts
                self.n_iter_ = 1 + refinement_release.iters
                return store_release(
                    tree,
                    consistent,
                    refinement_release.centers,
                    estimate_tree_inertia(
                        consistent,
                        self.splitter,
                        refinement_release.centers,
                        objective="kmeans",
                    ),
                )

            self.measurement_ = comp >> _new_pure_function(
                postprocess_tree_and_refine,
                TO="ExtrinsicObject",
            )

    def _coerce_X(self, X):
        np = _np()
        dtype = _input_domain_dtype(self.input_domain)
        X = np.asarray(X, dtype=dtype) if dtype is not None else np.asarray(X)
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
                return np.abs(X[:, None, :] - centers[None, :, :]).sum(axis=2)
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
    :param select_branch_fraction: Share of each selected level's budget used for private axis choice.
    :param select_branch_max_depth: Maximum depth at which to privately choose split axes.
    :param random_shift: Whether to apply a public random shift to quadtree root bounds. ``"auto"`` enables shifting for non-boolean NumPy array domains and disables it for boolean domains.
    :param random_state: Public seed used to sample the quadtree shift when shifting is enabled.
    :param refinement: Optional private center refinement. ``"none"`` keeps tree-derived centers. ``"dp_lloyd"`` uses the tree centers as a private initialization and spends part of the privacy budget on DP Lloyd updates.
    :param refinement_iters: Number of DP Lloyd refinement iterations.
    :param refinement_fraction: Fraction of the total privacy budget reserved for refinement.
    :param refinement_count_fraction: Fraction of each refinement iteration's budget used for noisy cluster counts. The remaining fraction is used for noisy cluster sums.
    :param refinement_min_count: Minimum noisy cluster count required before updating a center from private Lloyd statistics. ``"auto"`` uses a noise-aware threshold based on ``sqrt(count_var)``.
    :param init_solver: Postprocessing solver used to turn the released private tree into initial centers. ``"tree_dp"`` uses the tree dynamic program, while ``"weighted_coreset"`` runs nonprivate weighted k-means on the released leaf histogram.
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
    :param select_branch_fraction: Share of each selected level's budget used for private axis choice.
    :param select_branch_max_depth: Maximum depth at which to privately choose split axes.
    :param random_shift: Whether to apply a public random shift to quadtree root bounds. ``"auto"`` enables shifting for non-boolean NumPy array domains and disables it for boolean domains.
    :param random_state: Public seed used to sample the quadtree shift when shifting is enabled.
    :param refinement: Refinement mode. ``"dp_lloyd"`` is currently unsupported for k-medians.
    :param init_solver: Initial solver mode. ``"weighted_coreset"`` is currently unsupported for k-medians.
    """
    objective = "kmedians"

    def _pairwise_distance(self, X, centers):
        np = _np()
        return np.abs(X[:, None, :] - centers[None, :, :]).sum(axis=2)
