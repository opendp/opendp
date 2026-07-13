"""Run the REAL OpenDP PE and DP-Lloyd on the hard/weak presets, same data.

Substantiates (or refutes) whether PE degrades more gracefully than DP-Lloyd on
harder, less-separable structure -- the literature's premise for HST/LSH/PE.
No public side information is passed to PE (compliant).
"""
import sys, dataclasses
import numpy as np
from sklearn.metrics import adjusted_rand_score, normalized_mutual_info_score

sys.path.insert(0, "/Users/shoebox/OpenDP/openDP/dp_clustering_bench")
sys.path.insert(0, "/private/tmp/claude-503/-Users-shoebox-OpenDP-openDP/604e3fb5-3b64-4659-a08c-e537c1f57452/scratchpad")
from dpclustbench.audience_synthetic import make_idgraph_tag_mixture, DEFAULT_FAMILIES
from noisy_sim import dp_spherical_lloyd

from opendp.extras.sklearn.cluster import (
    SparsePrivateEvolutionMeans, SparsePrivateEvolutionConfig,
)


def scaled_families(m):
    return tuple(dataclasses.replace(f, core_hit_prob=f.core_hit_prob * m) for f in DEFAULT_FAMILIES)


def run_pe(x, k, rho, seed):
    cfg = SparsePrivateEvolutionConfig(  # benchmark defaults, no public structure
        iterations=16, population_size=512, center_active_tags=96,
        min_active_tags=16, max_active_tags=160, distance="weighted_jaccard",
        feature_prior=None, feature_groups=None, public_seed_candidates=None,
    )
    model = SparsePrivateEvolutionMeans(n_features=x.shape[1], n_clusters=k,
                                        rho=rho, random_state=seed, config=cfg)
    model.fit(x)
    return adjusted_rand_score(y, model.labels_)


PRESETS = {"weak": dict(chp=0.4, shared=0.15, k=16),
           "hard": dict(chp=0.4, shared=0.5, k=40)}

print(f"{'preset':6s} {'rho':>4s} {'PE_ARI':>8s} {'DPL_ARI':>8s}")
for name, p in PRESETS.items():
    fam = scaled_families(p["chp"])
    x, y, meta = make_idgraph_tag_mixture(n=50_000, k=p["k"], families=fam,
                                          mean_active_tags=64, shared_core_fraction=p["shared"],
                                          seed=20260713)
    x = x.tocsr().astype(np.float32)
    for rho in (0.1, 0.5):
        pe = np.mean([run_pe(x, p["k"], rho, s) for s in range(2)])
        dpl = np.mean([adjusted_rand_score(y, dp_spherical_lloyd(x, p["k"], T=5, rho=rho, L=128, m=96, seed=s)[0])
                       for s in range(2)])
        print(f"{name:6s} {rho:4.1f} {pe:8.3f} {dpl:8.3f}", flush=True)
