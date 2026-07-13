"""Full noisy zCDP spherical-Lloyd simulation (numpy).

Center_j = normalize(top-m entries of noisy cluster-sum S_j). No count release.
Sensitivity of S (concatenated k x d) under add/remove one clipped row = sqrt(L).
Split rho equally over T iterations: rho_iter = rho/T, sigma = sqrt(L*T/(2*rho)).
"""
import sys
import numpy as np
from scipy import sparse
from sklearn.metrics import adjusted_rand_score, normalized_mutual_info_score

sys.path.insert(0, "/Users/shoebox/OpenDP/openDP/dp_clustering_bench")
from dpclustbench.audience_synthetic import make_idgraph_tag_mixture

x, y, meta = make_idgraph_tag_mixture(n=50_000, k=16, mean_active_tags=64, seed=20260713)
x = x.tocsr().astype(np.float32)
n, d = x.shape
k = 16
row_nnz = np.diff(x.indptr)
print(f"n={n} d={d} k={k} mean_nnz={row_nnz.mean():.1f} max_nnz={row_nnz.max()}")


def clip_rows(x, L, seed=0):
    """Deterministic-ish clip each row to <= L tags (subsample if over)."""
    rng = np.random.default_rng(seed)
    x = x.tocsr()
    keep_data, keep_ind, indptr = [], [], [0]
    for i in range(x.shape[0]):
        s, e = x.indptr[i], x.indptr[i + 1]
        cols = x.indices[s:e]
        if cols.size > L:
            cols = rng.choice(cols, size=L, replace=False)
        keep_ind.append(cols)
        indptr.append(indptr[-1] + cols.size)
    indices = np.concatenate(keep_ind) if keep_ind else np.empty(0, np.int32)
    data = np.ones(indices.size, np.float32)
    return sparse.csr_matrix((data, indices, np.array(indptr)), shape=x.shape)


def normalize_rows_csr(m):
    m = m.tocsr().astype(np.float32)
    nrm = np.sqrt(np.asarray(m.multiply(m).sum(axis=1)).ravel())
    nrm[nrm == 0] = 1.0
    return m.multiply(1.0 / nrm[:, None]).tocsr()


def dp_spherical_lloyd(x, k, T, rho, L, m, seed=0, idf=None, verbose=False):
    rng = np.random.default_rng(seed)
    n, d = x.shape
    xc = clip_rows(x, L, seed=seed)          # bounded-sensitivity data for sums
    xn = normalize_rows_csr(x if idf is None else x.multiply(idf).tocsr())  # for assignment
    sigma = np.sqrt(L * T / (2.0 * rho))
    # random-feature init: k random ~m-tag unit centers
    centers = np.zeros((k, d), dtype=np.float32)
    for j in range(k):
        cols = rng.choice(d, size=m, replace=False)
        centers[j, cols] = 1.0
    cn = np.linalg.norm(centers, axis=1, keepdims=True); cn[cn == 0] = 1.0
    centers /= cn
    labels = None
    for t in range(T):
        sims = xn.dot(centers.T)              # assignment (postprocessing of private centers)
        labels = np.asarray(sims.argmax(axis=1)).ravel().astype(np.int32)
        new_centers = np.zeros((k, d), dtype=np.float32)
        for j in range(k):
            idx = np.flatnonzero(labels == j)
            S = np.asarray(xc[idx].sum(axis=0)).ravel() if idx.size else np.zeros(d)
            S = S + rng.normal(0.0, sigma, size=d)       # DP Gaussian release
            top = np.argpartition(S, -m)[-m:]
            top = top[S[top] > 0]
            v = np.zeros(d, dtype=np.float32)
            if top.size:
                v[top] = S[top]
                nv = np.linalg.norm(v)
                if nv > 0:
                    v /= nv
            new_centers[j] = v
        centers = new_centers
        if verbose:
            print(f"    it{t} ARI={adjusted_rand_score(y,labels):.3f}")
    return labels, sigma


if __name__ == "__main__":
  for rho in (0.1, 0.5):
    for T in (5, 8, 12):
        for m in (150, 300):
            aris, nmis = [], []
            for seed in range(2):
                lab, sigma = dp_spherical_lloyd(x, k, T=T, rho=rho, L=128, m=m, seed=seed)
                aris.append(adjusted_rand_score(y, lab)); nmis.append(normalized_mutual_info_score(y, lab))
            print(f"rho={rho} T={T} m={m} sigma={sigma:6.1f}  ARI={np.mean(aris):.4f}  NMI={np.mean(nmis):.4f}")
