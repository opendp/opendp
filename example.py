import numpy as np
from scipy import sparse
from sklearn.metrics import adjusted_rand_score

import opendp.prelude as dp

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


# method 1. sparse binary input, zCDP
from opendp.extras.sklearn.cluster import SphericalKMeans, SphericalKMeansConfig

# A single Context holds the data and a total zCDP budget (rho = 1.0), split across
# three queries that are each accounted separately:
#     release 1: cluster centers      (60% of the budget)  -- est.fit(query)
#     release 2: DP cluster sizes      (20%)                -- est.cluster_sizes(query)
#     release 3: DP silhouette score   (20%)                -- est.silhouette(query)
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

print("1) spherical     centers", est.cluster_centers_.shape,
      "ARI", round(adjusted_rand_score(y, np.asarray(est.predict(x))), 3))
print("     DP cluster sizes:", np.asarray(sizes), " (true:",
      np.bincount(y, minlength=k), ")")
print("     DP silhouette   :", round(float(silhouette), 3))
print("     budget spent    :", ctx.current_privacy_loss(), "(total rho = 0.5)")


# method 2. sparse binary input, zCDP
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
