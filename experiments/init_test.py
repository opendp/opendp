"""Does cosine k-means still recover clusters WITHOUT data-row init?

DP-legal init cannot look at raw rows. Test:
 (a) random feature-space centers (k random sparse unit vectors)
 (b) over-cluster (k' > k) from random rows then keep k
 (c) how few Lloyd iterations suffice with good (row) init
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

# tf-idf weight
df = np.asarray((x > 0).sum(axis=0)).ravel()
idf = np.log((n + 1) / (df + 1)) + 1.0
xw = x.multiply(idf).tocsr()
norms = np.sqrt(np.asarray(xw.multiply(xw).sum(axis=1)).ravel()); norms[norms == 0] = 1.0
xn = xw.multiply(1.0 / norms[:, None]).tocsr()


def lloyd(centers, n_iter, verbose=False):
    labels = np.zeros(n, dtype=np.int32)
    for it in range(n_iter):
        sims = xn.dot(centers.T)
        new = np.asarray(sims.argmax(axis=1)).ravel().astype(np.int32)
        if it > 0 and np.array_equal(new, labels):
            labels = new; break
        labels = new
        for j in range(centers.shape[0]):
            idx = np.flatnonzero(labels == j)
            if idx.size == 0:
                continue
            c = np.asarray(xn[idx].sum(axis=0)).ravel()
            cn = np.linalg.norm(c); centers[j] = c / cn if cn > 0 else c
        if verbose:
            print(f"  it{it} ARI={adjusted_rand_score(y,labels):.3f}")
    return labels


def report(tag, centers, n_iter=50):
    lab = lloyd(centers.copy(), n_iter)
    print(f"{tag}: ARI={adjusted_rand_score(y,lab):.4f} NMI={normalized_mutual_info_score(y,lab):.4f}")


# (a) random feature-space init: each center random ~90 nonzero tags, unit norm
rng = np.random.default_rng(0)
C = np.zeros((k, d), dtype=np.float32)
for j in range(k):
    cols = rng.choice(d, size=90, replace=False)
    C[j, cols] = 1.0
    C[j] /= np.linalg.norm(C[j])
report("(a) random-feature init", C)

# (a2) random init weighted by global marginal (df) — DP-legal if df released privately
C2 = np.zeros((k, d), dtype=np.float32)
p = df / df.sum()
for j in range(k):
    cols = rng.choice(d, size=90, replace=False, p=p)
    C2[j, cols] = 1.0
    C2[j] /= np.linalg.norm(C2[j])
report("(a2) marginal-weighted random init", C2)

# (c) row init but only few iterations
for T in (3, 5, 8):
    C3 = xn[rng.choice(n, size=k, replace=False)].toarray()
    report(f"(c) row-init T={T}", C3, n_iter=T)
