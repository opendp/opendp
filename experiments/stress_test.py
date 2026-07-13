"""Stress test: does the DP-Lloyd advantage survive harder / less-separable data?

Vary the generator away from the (easy) default and measure, per difficulty:
  - non-private cosine k-means CEILING (how separable the data still is)
  - DP-Lloyd (our method) at rho in {0.1, 0.5}
The default is deliberately easy (random-feature init hits ARI 1.0). We make it
harder along three axes that plausibly reflect real ID-graph data:
  weak   : core_hit_prob x0.4  (fainter per-cluster signal)
  overlap: shared_core_fraction 0.15 -> 0.5  (clusters share more cores)
  many_k : k=16 -> 40           (more, smaller clusters)
  hard   : all of the above combined
"""
import sys, dataclasses
import numpy as np
from scipy import sparse
from sklearn.metrics import adjusted_rand_score, normalized_mutual_info_score

sys.path.insert(0, "/Users/shoebox/OpenDP/openDP/dp_clustering_bench")
sys.path.insert(0, "/private/tmp/claude-503/-Users-shoebox-OpenDP-openDP/604e3fb5-3b64-4659-a08c-e537c1f57452/scratchpad")
from dpclustbench.audience_synthetic import make_idgraph_tag_mixture, DEFAULT_FAMILIES
from noisy_sim import clip_rows, normalize_rows_csr, dp_spherical_lloyd


def scaled_families(chp_mult):
    return tuple(dataclasses.replace(f, core_hit_prob=f.core_hit_prob * chp_mult) for f in DEFAULT_FAMILIES)


def ceiling(x, y, k, seeds=2):
    df = np.asarray((x > 0).sum(axis=0)).ravel()
    idf = np.log((x.shape[0] + 1) / (df + 1)) + 1.0
    xw = x.multiply(idf).tocsr()
    nrm = np.sqrt(np.asarray(xw.multiply(xw).sum(axis=1)).ravel()); nrm[nrm == 0] = 1
    xn = xw.multiply(1.0 / nrm[:, None]).tocsr()
    best = -1
    for s in range(seeds):
        rng = np.random.default_rng(s)
        C = xn[rng.choice(x.shape[0], size=k, replace=False)].toarray()
        lab = np.zeros(x.shape[0], np.int32)
        for _ in range(40):
            new = np.asarray(xn.dot(C.T).argmax(axis=1)).ravel().astype(np.int32)
            if np.array_equal(new, lab): break
            lab = new
            for j in range(k):
                idx = np.flatnonzero(lab == j)
                if idx.size:
                    c = np.asarray(xn[idx].sum(axis=0)).ravel(); n = np.linalg.norm(c)
                    C[j] = c / n if n > 0 else c
        best = max(best, adjusted_rand_score(y, lab))
    return best


PRESETS = {
    "default":  dict(chp=1.0, shared=0.15, k=16),
    "weak":     dict(chp=0.4, shared=0.15, k=16),
    "overlap":  dict(chp=1.0, shared=0.5,  k=16),
    "many_k":   dict(chp=1.0, shared=0.15, k=40),
    "hard":     dict(chp=0.4, shared=0.5,  k=40),
}

print(f"{'preset':10s} {'ceiling':>8s} {'DPL rho=0.1':>12s} {'DPL rho=0.5':>12s}")
for name, p in PRESETS.items():
    fam = scaled_families(p["chp"])
    x, y, meta = make_idgraph_tag_mixture(
        n=50_000, k=p["k"], families=fam, mean_active_tags=64,
        shared_core_fraction=p["shared"], seed=20260713)
    x = x.tocsr().astype(np.float32)
    ceil = ceiling(x, y, p["k"])
    res = {}
    for rho in (0.1, 0.5):
        aris = []
        for s in range(3):
            lab, _ = dp_spherical_lloyd(x, p["k"], T=5, rho=rho, L=128, m=96, seed=s)
            aris.append(adjusted_rand_score(y, lab))
        res[rho] = (np.mean(aris), np.std(aris))
    print(f"{name:10s} {ceil:8.3f} {res[0.1][0]:6.3f}±{res[0.1][1]:.2f} {res[0.5][0]:6.3f}±{res[0.5][1]:.2f}")
