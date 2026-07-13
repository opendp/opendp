"""Non-private recoverability ceiling for the idgraph tag mixture.

Establishes how much cluster signal exists at all, independent of DP.
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
print(f"n={n} d={d} nnz/row={x.nnz/n:.1f} k={k}")


def cosine_kmeans(x, k, n_iter=50, seed=0, tfidf=False):
    rng = np.random.default_rng(seed)
    xw = x
    if tfidf:
        df = np.asarray((x > 0).sum(axis=0)).ravel()
        idf = np.log((n + 1) / (df + 1)) + 1.0
        xw = x.multiply(idf).tocsr()
    # L2-normalize rows
    norms = np.sqrt(np.asarray(xw.multiply(xw).sum(axis=1)).ravel())
    norms[norms == 0] = 1.0
    xn = xw.multiply(1.0 / norms[:, None]).tocsr()
    # init: random distinct rows
    centers = xn[rng.choice(n, size=k, replace=False)].toarray()
    labels = np.zeros(n, dtype=np.int32)
    for it in range(n_iter):
        sims = xn.dot(centers.T)  # n x k
        new = np.asarray(sims.argmax(axis=1)).ravel().astype(np.int32)
        if it > 0 and np.array_equal(new, labels):
            labels = new
            break
        labels = new
        for j in range(k):
            idx = np.flatnonzero(labels == j)
            if idx.size == 0:
                centers[j] = xn[rng.integers(0, n)].toarray()
                continue
            c = np.asarray(xn[idx].sum(axis=0)).ravel()
            cn = np.linalg.norm(c)
            centers[j] = c / cn if cn > 0 else c
    return labels


for tfidf in (False, True):
    best = (-1, None)
    for seed in range(3):
        lab = cosine_kmeans(x, k, seed=seed, tfidf=tfidf)
        ari = adjusted_rand_score(y, lab)
        nmi = normalized_mutual_info_score(y, lab)
        if ari > best[0]:
            best = (ari, nmi)
    print(f"cosine_kmeans tfidf={tfidf}: best ARI={best[0]:.4f} NMI={best[1]:.4f}")
