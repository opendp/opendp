import itertools
from typing import TYPE_CHECKING, cast
import pytest

from opendp.extras.sklearn.cluster import (
    ClusterRelease,
    KMeans,
    KMedians,
    make_private_tree_cluster,
)
from opendp.extras.sklearn.cluster._postprocess import Node
from opendp.extras.sklearn.cluster._tree import LSH, Quadtree
import opendp.prelude as dp

np = pytest.importorskip("numpy")

if TYPE_CHECKING:
    import numpy


@pytest.fixture
def X_train():
    return np.array(
        [
            [0.0, 0.0],
            [0.0, 1.0],
            [4.0, 4.0],
            [4.0, 5.0],
        ],
        dtype=float,
    )


@pytest.fixture
def fake_release():
    return ClusterRelease(
        tree_=[Node(2.0, 1.0), Node(2.0, 1.0)],
        consistent_tree_=[Node(2.0, 0.5), Node(2.0, 0.5)],
        cluster_centers_=np.array([[0.0, 0.5], [4.0, 4.5]], dtype=float),
        inertia_=1.5,
        objective_="kmeans",
    )


@pytest.fixture
def kmeans(fake_release):
    model = KMeans(
        n_features=2,
        n_clusters=2,
        scale=1.0,
        max_depth=3,
        lower=np.array([0.0, 0.0]),
        upper=np.array([5.0, 5.0]),
    )
    model.measurement_ = lambda X: fake_release
    return model


@pytest.fixture
def kmedians(fake_release):
    model = KMedians(
        n_features=2,
        n_clusters=2,
        scale=1.0,
        max_depth=3,
        lower=np.array([0.0, 0.0]),
        upper=np.array([5.0, 5.0]),
    )
    model.measurement_ = lambda X: ClusterRelease(
        tree_=fake_release.tree_,
        consistent_tree_=fake_release.consistent_tree_,
        cluster_centers_=fake_release.cluster_centers_,
        inertia_=2.0,
        objective_="kmedians",
    )
    return model


@pytest.mark.parametrize(
    "kwargs",
    [
        dict(max_depth=None, level_allocation=None),
        dict(max_depth=3, level_allocation=[1.0, 1.0]),
    ],
)
def test_requires_exactly_one_of_max_depth_or_level_allocation(kwargs):
    with pytest.raises(ValueError, match="exactly one"):
        KMeans(
            n_features=2,
            n_clusters=2,
            scale=1.0,
            lower=np.array([0.0, 0.0]),
            upper=np.array([1.0, 1.0]),
            **kwargs,
        )


@pytest.mark.parametrize("bad_scale", [0.0, -1.0])
def test_requires_positive_scale(bad_scale):
    with pytest.raises(ValueError, match="scale must be positive"):
        KMeans(
            n_features=2,
            n_clusters=2,
            scale=bad_scale,
            max_depth=2,
            lower=np.array([0.0, 0.0]),
            upper=np.array([1.0, 1.0]),
        )


@pytest.mark.parametrize("bad_clusters", [0, -1])
def test_requires_positive_n_clusters(bad_clusters):
    with pytest.raises(ValueError, match="n_clusters must be positive"):
        KMeans(
            n_features=2,
            n_clusters=bad_clusters,
            scale=1.0,
            max_depth=2,
            lower=np.array([0.0, 0.0]),
            upper=np.array([1.0, 1.0]),
        )


@pytest.mark.parametrize("bad_depth", [0, -1])
def test_requires_positive_max_depth(bad_depth):
    with pytest.raises(ValueError, match="max_depth must be positive"):
        KMeans(
            n_features=2,
            n_clusters=2,
            scale=1.0,
            max_depth=bad_depth,
            lower=np.array([0.0, 0.0]),
            upper=np.array([1.0, 1.0]),
        )



def test_max_depth_expands_to_uniform_level_allocation():
    model = KMeans(
        n_features=2,
        n_clusters=2,
        scale=1.0,
        max_depth=4,
        lower=np.array([0.0, 0.0]),
        upper=np.array([1.0, 1.0]),
    )
    assert np.array_equal(model.level_allocation, np.ones(4))



def test_level_allocation_preserved():
    level_allocation = np.array([1.0, 2.0, 3.0])
    model = KMeans(
        n_features=2,
        n_clusters=2,
        scale=1.0,
        level_allocation=level_allocation,
        lower=np.array([0.0, 0.0]),
        upper=np.array([1.0, 1.0]),
    )
    assert np.array_equal(model.level_allocation, level_allocation)
    assert model.max_depth is None



def test_fit_returns_self_and_sets_attributes(kmeans, X_train, fake_release):
    out = kmeans.fit(X_train)
    assert out is kmeans
    assert np.array_equal(kmeans.cluster_centers_, fake_release.cluster_centers_)
    assert kmeans.inertia_ == fake_release.inertia_
    assert kmeans.tree_ == fake_release.tree_
    assert kmeans.consistent_tree_ == fake_release.consistent_tree_
    assert kmeans.labels_ is None
    assert kmeans.n_iter_ == 1
    assert kmeans.n_features_in_ == 2



def test_fit_rejects_sample_weight(kmeans, X_train):
    with pytest.raises(NotImplementedError, match="sample_weight"):
        kmeans.fit(X_train, sample_weight=np.ones(len(X_train)))



def test_fit_rejects_bad_input_shape(kmeans):
    with pytest.raises(ValueError, match="expected 2D array"):
        kmeans.fit(np.array([1.0, 2.0]))



def test_fit_rejects_bad_feature_count(kmeans):
    with pytest.raises(ValueError, match="expected 2 features"):
        kmeans.fit(np.ones((5, 3)))


def test_predict_rejects_bad_input_shape(kmeans):
    kmeans.fit(np.ones((2, 2)))
    with pytest.raises(ValueError, match="expected 2D array"):
        kmeans.predict(np.array([1.0, 2.0]))


def test_predict_rejects_bad_feature_count(kmeans):
    kmeans.fit(np.ones((2, 2)))
    with pytest.raises(ValueError, match="expected 2 features"):
        kmeans.predict(np.ones((5, 3)))



def test_predict_requires_fit(kmeans, X_train):
    fresh = KMeans(
        n_features=2,
        n_clusters=2,
        scale=1.0,
        max_depth=3,
        lower=np.array([0.0, 0.0]),
        upper=np.array([5.0, 5.0]),
    )
    with pytest.raises(ValueError, match="not been fitted"):
        fresh.predict(X_train)



def test_predict_returns_cluster_assignments(kmeans, X_train):
    kmeans.fit(X_train)
    labels = kmeans.predict(X_train)
    assert labels.shape == (len(X_train),)
    assert set(labels.tolist()) <= {0, 1}
    assert labels[0] == labels[1]
    assert labels[2] == labels[3]
    assert labels[0] != labels[2]



def test_transform_returns_pairwise_distances(kmeans, X_train):
    kmeans.fit(X_train)
    dists = kmeans.transform(X_train)
    assert dists.shape == (len(X_train), 2)
    assert np.all(dists >= 0)


def test_transform_requires_fit(X_train):
    fresh = KMeans(
        n_features=2,
        n_clusters=2,
        scale=1.0,
        max_depth=3,
        lower=np.array([0.0, 0.0]),
        upper=np.array([5.0, 5.0]),
    )
    with pytest.raises(ValueError, match="not been fitted"):
        fresh.transform(X_train)


def test_transform_rejects_bad_input_shape(kmeans):
    kmeans.fit(np.ones((2, 2)))
    with pytest.raises(ValueError, match="expected 2D array"):
        kmeans.transform(np.array([1.0, 2.0]))


def test_transform_rejects_bad_feature_count(kmeans):
    kmeans.fit(np.ones((2, 2)))
    with pytest.raises(ValueError, match="expected 2 features"):
        kmeans.transform(np.ones((5, 3)))



def test_fit_predict_matches_fit_then_predict(kmeans, X_train):
    model1 = kmeans
    labels1 = model1.fit_predict(X_train)

    model2 = KMeans(
        n_features=2,
        n_clusters=2,
        scale=1.0,
        max_depth=3,
        lower=np.array([0.0, 0.0]),
        upper=np.array([5.0, 5.0]),
    )
    model2.measurement_ = model1.measurement_
    model2.fit(X_train)
    labels2 = model2.predict(X_train)

    assert np.array_equal(labels1, labels2)



def test_fit_transform_matches_fit_then_transform(kmeans, X_train):
    model1 = kmeans
    Xt1 = model1.fit_transform(X_train)

    model2 = KMeans(
        n_features=2,
        n_clusters=2,
        scale=1.0,
        max_depth=3,
        lower=np.array([0.0, 0.0]),
        upper=np.array([5.0, 5.0]),
    )
    model2.measurement_ = model1.measurement_
    model2.fit(X_train)
    Xt2 = model2.transform(X_train)

    assert np.allclose(Xt1, Xt2)



def test_score_is_negative_sum_of_min_distances(kmeans, X_train):
    kmeans.fit(X_train)
    dists = kmeans.transform(X_train)
    expected = -float(np.sum(np.min(dists, axis=1)))
    assert np.isclose(kmeans.score(X_train), expected)



def test_score_rejects_sample_weight(kmeans, X_train):
    kmeans.fit(X_train)
    with pytest.raises(NotImplementedError, match="sample_weight"):
        kmeans.score(X_train, sample_weight=np.ones(len(X_train)))



def test_kmedians_uses_l1_distance(kmedians, X_train):
    kmedians.fit(X_train)
    dists = kmedians.transform(X_train)
    expected = np.abs(X_train[:, None, :] - kmedians.cluster_centers_[None, :, :]).sum(axis=2)
    assert np.allclose(dists, expected)


def test_make_private_tree_cluster_validates_objective():
    with pytest.raises(ValueError, match="objective"):
        make_private_tree_cluster(
            input_domain=dp.numpy.array2_domain(T=float, num_columns=2),
            input_metric=dp.symmetric_distance(),
            output_measure=dp.max_divergence(),
            splitter=Quadtree(np.array([0.0, 0.0]), np.array([1.0, 1.0])),
            level_allocation=[1.0],
            scale=1.0,
            n_clusters=2,
            objective="bad",
        )


def test_make_private_tree_cluster_requires_quadtree_splitter():
    with pytest.raises(NotImplementedError, match="Quadtree"):
        make_private_tree_cluster(
            input_domain=dp.numpy.array2_domain(T=float, num_columns=2),
            input_metric=dp.symmetric_distance(),
            output_measure=dp.max_divergence(),
            splitter=cast(Quadtree, LSH(np.array([[1.0, 0.0]]))),
            level_allocation=[1.0],
            scale=1.0,
            n_clusters=2,
            objective="kmeans",
        )


@pytest.mark.parametrize("bad_clusters", [0, -1])
def test_make_private_tree_cluster_requires_positive_n_clusters(bad_clusters):
    with pytest.raises(ValueError, match="n_clusters must be positive"):
        make_private_tree_cluster(
            input_domain=dp.numpy.array2_domain(T=float, num_columns=2),
            input_metric=dp.symmetric_distance(),
            output_measure=dp.max_divergence(),
            splitter=Quadtree(np.array([0.0, 0.0]), np.array([1.0, 1.0])),
            level_allocation=[1.0],
            scale=1.0,
            n_clusters=bad_clusters,
            objective="kmeans",
        )


def test_cluster_release_preserves_objective_name():
    release = ClusterRelease(
        tree_=[],
        consistent_tree_=[],
        cluster_centers_=np.zeros((0, 2)),
        inertia_=0.0,
        objective_="kmedians",
    )
    assert release.objective_ == "kmedians"


def _make_blob_dataset(seed: int = 0):
    rng = np.random.default_rng(seed)
    centers = np.array([
        [-4.0, -4.0],
        [0.0, 5.0],
        [5.0, 0.5],
    ])
    counts = [120, 110, 130]
    stds = [0.45, 0.55, 0.5]

    xs = []
    ys = []
    for label, (center, n, std) in enumerate(zip(centers, counts, stds)):
        xs.append(center + rng.normal(scale=std, size=(n, 2)))
        ys.append(np.full(n, label, dtype=int))

    X = np.vstack(xs)
    y = np.concatenate(ys)
    return X, y, centers



def _best_center_errors(found: "numpy.ndarray", truth: "numpy.ndarray"):
    best = None
    for perm in itertools.permutations(range(len(truth))):
        errs = np.linalg.norm(found - truth[list(perm)], axis=1)
        score = float(np.sum(errs))
        if best is None or score < best[0]:
            best = (score, errs, perm)
    assert best is not None
    return best[1], best[2]



def _best_label_accuracy(pred: "numpy.ndarray", truth: "numpy.ndarray", k: int):
    best = 0.0
    for perm in itertools.permutations(range(k)):
        mapped = np.array([perm[i] for i in pred], dtype=int)
        acc = float(np.mean(mapped == truth))
        best = max(best, acc)
    return best



def test_kmeans_end_to_end_on_blob_dataset():
    X, y, true_centers = _make_blob_dataset(seed=7)
    padding = 1.5
    lower = X.min(axis=0) - padding
    upper = X.max(axis=0) + padding

    model = KMeans(
        n_features=2,
        n_clusters=3,
        scale=1e-6,
        max_depth=7,
        lower=lower,
        upper=upper,
    )
    model.fit(X)

    assert model.cluster_centers_ is not None
    assert model.inertia_ is not None
    assert model.cluster_centers_.shape == (3, 2)
    assert np.isfinite(model.inertia_)
    assert model.inertia_ >= 0.0
    assert model.tree_ is not None
    assert model.consistent_tree_ is not None

    center_errors, _ = _best_center_errors(model.cluster_centers_, true_centers)
    assert np.max(center_errors) < 1.25
    assert np.mean(center_errors) < 0.9

    pred = model.predict(X)
    assert pred.shape == (len(X),)
    acc = _best_label_accuracy(pred, y, k=3)
    assert acc > 0.9

    transformed = model.transform(X[:10])
    assert transformed.shape == (10, 3)
    assert np.all(transformed >= 0.0)

    score = model.score(X)
    assert np.isfinite(score)
    assert score <= 0.0
