"""Differentially private clustering with OpenDP -- release examples.

Demonstrates a DP release with each of the three clustering methods in
``opendp.extras.sklearn.cluster``:

  1. tree-based KMeans / KMedians (HST)      -- dense data
  2. sparse Private Evolution (PE) means     -- sparse binary data
  3. spherical (cosine) k-means              -- sparse binary data, Context API

and, for the spherical setting, two further separately-accounted releases over the
fitted model:

  * DP cluster sizes    -- ``est.cluster_sizes(query)``, postprocessing ``est.predict``
  * DP silhouette score -- ``est.silhouette(query)``,     postprocessing ``est.transform``

Run with:  pip install 'opendp[scikit-learn]' scikit-learn
"""
import numpy as np
from scipy import sparse
from sklearn.metrics import adjusted_rand_score

import opendp.prelude as dp

# Only "contrib" is required -- none of these mechanisms need "honest-but-curious".
dp.enable_features("contrib")


def make_sparse_binary_blobs(n=3000, d=48, k=4, seed=0):
    """k blobs, each defined by a small set of 'core' features that fire with high
    probability, plus a little per-row background noise. Returns (csr matrix, labels)."""
    rng = np.random.default_rng(seed)
    cores = [rng.choice(d, size=6, replace=False) for _ in range(k)]
    y = rng.integers(0, k, size=n)
    rows, cols = [], []
    for i in range(n):
        hit = cores[y[i]][rng.random(6) < 0.7]
        noise = rng.choice(d, size=3, replace=False)
        tags = np.unique(np.concatenate([hit, noise]))
        rows.append(np.full(tags.size, i))
        cols.append(tags)
    data = np.ones(int(sum(c.size for c in cols)), dtype=np.float32)
    x = sparse.csr_matrix((data, (np.concatenate(rows), np.concatenate(cols))), shape=(n, d))
    x.data[:] = 1.0
    return x, y


x, y = make_sparse_binary_blobs()
n, d = x.shape
k = 4
print(f"data: n={n} d={d} k={k}\n")


# ======================================================================================
# Method 1 -- tree-based KMeans (HST).  Dense input; pure-DP (Laplace) by default.
# The budget is set by the measure-agnostic `scale`; `.fit(X)` performs the DP release.
# ======================================================================================
X_dense = x.toarray().astype(float)  # 0/1 features, so bounds are [0, 1]
km = dp.sklearn.cluster.KMeans(
    n_features=d,
    n_clusters=k,
    scale=1.0,
    max_depth=6,
    lower=np.zeros(d),
    upper=np.ones(d),
    random_state=0,
)
km.fit(X_dense)  # <-- DP release of the tree-derived centers
print("1) tree KMeans   centers", km.cluster_centers_.shape,
      "ARI", round(adjusted_rand_score(y, np.asarray(km.predict(X_dense))), 3))


# ======================================================================================
# Method 2 -- sparse Private Evolution means.  Sparse binary input; zCDP.
# The budget `rho` is passed to the estimator; `.fit(x)` performs the DP release.
# ======================================================================================
from opendp.extras.sklearn.cluster import SparsePrivateEvolutionMeans, SparsePrivateEvolutionConfig

pe = SparsePrivateEvolutionMeans(
    n_features=d,
    n_clusters=k,
    rho=0.5,
    random_state=0,
    config=SparsePrivateEvolutionConfig(
        iterations=8, population_size=128,
        center_active_tags=8, min_active_tags=2, max_active_tags=12,
    ),
)
pe.fit(x)  # <-- DP release
print("2) PE means      centers", pe.cluster_centers_.shape,
      "ARI", round(adjusted_rand_score(y, np.asarray(pe.labels_)), 3))


# ======================================================================================
# Method 3 -- spherical (cosine) k-means.  Sparse binary input; zCDP; Context API.
#
# A single Context holds the data and a total zCDP budget (rho = 1.0), split across
# three queries that are each accounted separately:
#     release 1: cluster centers      (60% of the budget)  -- est.fit(query)
#     release 2: DP cluster sizes      (20%)                -- est.cluster_sizes(query)
#     release 3: DP silhouette score   (20%)                -- est.silhouette(query)
# The input domain is inferred from the CSR matrix (sparse_binary_domain(d)).
# ======================================================================================
from opendp.extras.sklearn.cluster import SphericalKMeans, SphericalKMeansConfig

ctx = dp.Context.compositor(
    data=x,
    privacy_unit=dp.unit_of(contributions=1),
    privacy_loss=dp.loss_of(rho=0.5),
    split_by_weights=[0.6, 0.2, 0.2],  # centers / sizes / silhouette
)

est = SphericalKMeans(
    n_clusters=k,
    random_state=0,
    config=SphericalKMeansConfig(iterations=5, center_active=8, max_active=16, init_active=8),
)
est.fit(ctx.query())                                # release 1: centers

# Diagnostics are methods on the fitted model. Each is its own DP release over the
# model's transformations (predict / transform) and consumes its own share of budget.
sizes = est.cluster_sizes(ctx.query())              # release 2: DP cluster sizes
silhouette = est.silhouette(ctx.query())            # release 3: DP silhouette

print("3) spherical     centers", est.cluster_centers_.shape,
      "ARI", round(adjusted_rand_score(y, np.asarray(est.predict(x))), 3))
print("     DP cluster sizes:", np.asarray(sizes), " (true:",
      np.bincount(y, minlength=k), ")")
print("     DP silhouette   :", round(float(silhouette), 3))
print("     budget spent    :", ctx.current_privacy_loss(), "(total rho = 1.0)")
