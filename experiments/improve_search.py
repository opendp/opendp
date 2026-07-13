"""Test two improvements to DP spherical Lloyd, in the fast numpy sim:

 (A) OVER-CLUSTER + MERGE (free, postprocessing): fit k'=over*k centers, then
     greedily merge released centers down to k by cosine similarity. Same per-row
     sensitivity (one row -> one of k' clusters), same rho. Attacks the #1 wart:
     random init occasionally leaves a true cluster unclaimed / double-claimed.

 (B) SOFT (EM-style) responsibilities: assign each row fractionally via softmax of
     cosine sims (temperature beta); release responsibility-weighted feature sums.
     This is DP EM for a latent-class / Bernoulli-mixture model -- the principled
     generalization of hard Lloyd. Sensitivity: one row's responsibilities sum to 1
     and it has <=L tags, so its contribution to the (k x d) sum matrix has L2
     <= sqrt(L) (responsibilities in [0,1], one row's mass spread across clusters
     only lowers L2). Same rho.
"""
import sys, dataclasses
import numpy as np
from scipy import sparse
from sklearn.metrics import adjusted_rand_score

sys.path.insert(0, "/Users/shoebox/OpenDP/openDP/dp_clustering_bench")
from dpclustbench.audience_synthetic import make_idgraph_tag_mixture, DEFAULT_FAMILIES


def prep(chp, shared, k):
    fam = tuple(dataclasses.replace(f, core_hit_prob=f.core_hit_prob * chp) for f in DEFAULT_FAMILIES)
    x, y, _ = make_idgraph_tag_mixture(n=50_000, k=k, families=fam, mean_active_tags=64,
                                       shared_core_fraction=shared, seed=20260713)
    x = x.tocsr().astype(np.float32)
    nrm = np.sqrt(np.asarray(x.multiply(x).sum(axis=1)).ravel()); nrm[nrm == 0] = 1
    xn = x.multiply(1.0 / nrm[:, None]).tocsr()
    return x, xn, y


def clip_first_L(x, L):
    if int(np.diff(x.indptr).max()) <= L:
        return x
    ind, ptr = [], [0]
    for i in range(x.shape[0]):
        c = x.indices[x.indptr[i]:x.indptr[i+1]][:L]
        ind.append(c); ptr.append(ptr[-1]+c.size)
    return sparse.csr_matrix((np.ones(ptr[-1], np.float32), np.concatenate(ind), np.asarray(ptr)), shape=x.shape)


def rand_centers(k, d, m, rng):
    C = np.zeros((k, d), np.float32)
    for j in range(k):
        C[j, rng.choice(d, m, replace=False)] = 1.0
    C /= np.maximum(np.linalg.norm(C, axis=1, keepdims=True), 1e-9)
    return sparse.csr_matrix(C)


def topm_norm(S, m):
    k, d = S.shape
    rows, ptr = [], [0]; vals = []
    for j in range(k):
        s = S[j]; pos = np.flatnonzero(s > 0)
        if pos.size == 0: ptr.append(ptr[-1]); continue
        top = pos[np.argpartition(s[pos], -m)[-m:]] if pos.size > m else pos
        v = s[top]; nv = np.linalg.norm(v)
        if nv <= 0: ptr.append(ptr[-1]); continue
        rows.append(top.astype(np.int32)); vals.append((v/nv).astype(np.float32)); ptr.append(ptr[-1]+top.size)
    idx = np.concatenate(rows) if rows else np.empty(0,np.int32)
    dat = np.concatenate(vals) if vals else np.empty(0,np.float32)
    return sparse.csr_matrix((dat, idx, np.asarray(ptr)), shape=(k, d))


def run(x, xn, y, k, rho, L=128, m=96, T=5, over=1, soft=False, beta=8.0, seed=0):
    rng = np.random.default_rng(seed)
    d = x.shape[1]; kk = over * k
    xc = clip_first_L(x, L)
    sigma = np.sqrt(L * T / (2.0 * rho))
    C = rand_centers(kk, d, m, rng)
    for _ in range(T):
        sims = xn.dot(C.T.tocsr()).toarray()
        if soft:
            R = sims - sims.max(1, keepdims=True); R = np.exp(beta*R); R /= R.sum(1, keepdims=True)
            S = (xc.T.dot(R)).T  # (kk x d) responsibility-weighted sums
            S = np.asarray(S)
        else:
            lab = sims.argmax(1)
            S = np.zeros((kk, d))
            for j in range(kk):
                idx = np.flatnonzero(lab == j)
                if idx.size: S[j] = np.asarray(xc[idx].sum(0)).ravel()
        S = S + rng.normal(0, sigma, S.shape)
        C = topm_norm(S, m)
    # merge kk -> k by agglomerative cosine (postprocessing of released centers)
    Cd = C.toarray()
    if kk > k:
        from sklearn.cluster import AgglomerativeClustering
        # cosine distance on centers; some rows may be all-zero
        act = np.flatnonzero(np.linalg.norm(Cd, axis=1) > 0)
        grp = np.arange(kk)
        if act.size >= k:
            al = AgglomerativeClustering(n_clusters=k, metric="cosine", linkage="average").fit(Cd[act])
            grp = np.full(kk, -1); grp[act] = al.labels_
        merged = np.zeros((k, d))
        for g in range(k):
            members = np.flatnonzero(grp == g)
            if members.size: merged[g] = Cd[members].sum(0)
        Cd = merged
        Cd /= np.maximum(np.linalg.norm(Cd, axis=1, keepdims=True), 1e-9)
    lab = xn.dot(sparse.csr_matrix(Cd).T.tocsr()).toarray().argmax(1)
    return adjusted_rand_score(y, lab)


if __name__ == "__main__":
    PRESETS = {"default": (1.0, 0.15, 16), "overlap": (1.0, 0.5, 16), "hard": (0.4, 0.5, 40)}
    variants = [("baseline hard-assign", dict(over=1, soft=False)),
                ("over-cluster x2 + merge", dict(over=2, soft=False)),
                ("soft EM (beta=8)", dict(over=1, soft=True))]
    for pname, (chp, sh, k) in PRESETS.items():
        x, xn, y = prep(chp, sh, k)
        for rho in (0.1, 0.5):
            line = f"{pname:8s} rho={rho}: "
            for vname, kw in variants:
                a = np.mean([run(x, xn, y, k, rho, seed=s, **kw) for s in range(3)])
                line += f"{vname}={a:.3f}  "
            print(line, flush=True)
