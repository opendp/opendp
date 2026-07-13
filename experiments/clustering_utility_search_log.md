# Clustering Utility Search Log

Problem: cluster sparse binary / multi-hot categorical "audience tag" data under zCDP.
Dataset: `make_idgraph_tag_mixture` — n=50,000 rows, d=75,688 tags, k=16 clusters,
~90 active tags/row (mean). Cluster signal = per-cluster "core" tags that appear with
elevated probability (core_hit_prob ≈ 0.16–0.32); heavy background noise + 3,000 hot
background tags (8× boost). Privacy unit = one consumer row; neighboring = add/remove
(`dp.symmetric_distance()`).

Environment to reproduce (opendp built from this repo's Rust target):
```
PY=/Users/shoebox/.pyenv/versions/3.12.12/envs/opendp2_312/bin/python
export PYTHONPATH=/Users/shoebox/OpenDP/openDP/python/src
export OPENDP_LIB_DIR=<dir containing exactly one symlink to rust/target/debug/libopendp.dylib>
cd dp_clustering_bench
```

Given baselines (reported in task):
| method | rho=0.1 ARI | rho=0.1 NMI | rho=0.5 ARI | rho=0.5 NMI |
|---|---|---|---|---|
| Google LSH | 0.0345 | 0.0635 | 0.0947 | 0.1469 |
| Generic PE | 0.0028 | 0.0139 | 0.0000 | 0.0000 |
| OpenDP PE | 0.0615 | 0.2102 | 0.0585 | 0.2035 |

---

## Attempt 0 — Diagnostic: is OpenDP PE noise-limited or algorithm-limited?

**Hypothesis:** if PE utility barely changes when we remove privacy noise, it is
algorithm-limited (the search cannot find the cluster cores), not noise-limited.

**Config changed:** none (used existing benchmark). Ran compliant (no public side
information: `--opendp-pe-no-public-structure`, per rule 3). Compared normal noise vs
near-zero noise (`--opendp-pe-noise-sigma 0.001`).

**Command:**
```
$PY -m dpclustbench.benchmark_audience_compare --algorithms opendp_pe_means \
  --runs 1 --rho-values 0.5 --opendp-pe-no-public-structure [--opendp-pe-noise-sigma 0.001]
```

**Results (1 run, seed 20260713):**
| setting | ARI | NMI |
|---|---|---|
| OpenDP PE, rho=0.5 (real noise, σ=4.0) | 0.0936 | 0.1547 |
| OpenDP PE, σ=0.001 (≈ no privacy) | 0.1077 | 0.1511 |

**Interpretation:** Removing essentially all privacy noise moves ARI only 0.094 → 0.108.
**PE is algorithm-limited, not noise-limited.** The PE search over random sparse
candidates in 75,688-dim space never locates the true cluster cores; privacy noise is
almost free here. (Note: the compliant no-public-structure PE already ~matches or beats
the reported OpenDP-PE baseline, confirming the reported numbers weren't the bottleneck.)

**Next action:** find the recoverability ceiling with a non-private clustering that
actually exploits the sparse structure.

---

## Attempt 1 — Non-private recoverability ceiling (cosine / spherical k-means)

**Hypothesis:** the cluster cores are recoverable by a cosine/spherical k-means directly
on the sparse binary rows; PE's poor result is an algorithm choice, not a data limit.

**Config:** standalone script (`scratchpad/ceiling.py`) — spherical k-means (row sums,
L2-normalized centers, cosine assignment), raw vs TF-IDF weighting, 3 seeds, 50 iters.

**Results (non-private, n=50k):**
| method | best ARI | best NMI |
|---|---|---|
| cosine k-means (raw binary) | 0.8913 | 0.9556 |
| cosine k-means + TF-IDF | 0.9717 | 0.9843 |

**Interpretation:** Signal is almost fully recoverable (ARI up to 0.97). The gap from
PE (0.11) to ceiling (0.9) is entirely algorithmic. **Spherical k-means is the right
objective.** Need a zCDP version.

**Next action:** check that a DP-*legal* initialization (no data-row access) still works.

---

## Attempt 2 — DP-legal initialization test (non-private)

**Hypothesis:** spherical k-means recovers clusters even from a random feature-space init
(k random sparse unit vectors) — the only init available under DP without touching rows.

**Config:** `scratchpad/init_test.py`, TF-IDF cosine k-means, various inits.

**Results (non-private):**
| init | ARI | NMI |
|---|---|---|
| random feature-space (k random ~90-tag unit vectors) | **1.0000** | **1.0000** |
| marginal-weighted random init | 0.8613 | 0.9468 |
| data-row init, T=5 iters | 0.9018 | 0.9721 |

**Interpretation:** Random feature-space init works perfectly and costs **zero privacy**
(no data access). Convergence in ~5 iterations. This removes the classic DP-Lloyd init
obstacle for this problem.

**Next action:** implement zCDP spherical Lloyd — random-feature init + T noisy
cluster-sum releases + top-m sparse center projection. Design/accounting below.

---

## Attempt 3 — Noisy simulation of zCDP spherical Lloyd (numpy)

**Hypothesis (from advisor):** the non-private wins don't prove the *noisy* version
works — noise enters the sums that build the centers. Must verify random-feature
init converges at the *small T* the budget allows, under real noise.

**Design.** Center_j = normalize(top-m features of noisy cluster-sum S_j). No count
release (cosine centers only need sums → also sidesteps rule 6). Rows clipped to a
fixed public cap L. One row → one cluster → ≤L ones in one block, so L2 sensitivity
of the flattened (k×d) sum is **sqrt(L)**, independent of k and d. Split rho evenly
over T releases: `sigma = sqrt(L*T/(2*rho))`.

**Results (`scratchpad/noisy_sim.py`, `refine.py`, 2 seeds, L=128):**
| rho | T | m | sigma | ARI | NMI |
|---|---|---|---|---|---|
| 0.1 | 5 | 80 | 56.6 | 0.842 | 0.895 |
| 0.1 | 5 | 150 | 56.6 | 0.781 | 0.876 |
| 0.5 | 5 | 100 | 25.3 | 0.946 | 0.954 |
| 0.5 | 4 | 100 | 22.6 | 0.930 | 0.942 |

**Interpretation:** Random-feature init converges under noise at **T=5**. Smaller T
(less noise) and moderate m (~80–100) win. ARI ~0.8 (rho=0.1) / ~0.9 (rho=0.5) —
an order of magnitude over every baseline. Noise is not the limiter here.

**Next action:** implement as a real OpenDP zCDP measurement (machine-checked
accounting) and run through the benchmark.

---

## Attempt 4 — OpenDP measurement + benchmark integration

**Code added:** `dpclustbench/algorithms/dp_spherical_lloyd.py` (new algorithm
`dp_spherical_lloyd`), wired into `benchmark_audience_compare.py` with `--dsl-*`
flags. Each Lloyd step is an OpenDP custom measurement (`_make_measurement`,
`privacy_map = L/(2*sigma^2)` for d_in=1) composed with
`make_adaptive_composition` under `zero_concentrated_divergence`. Gaussian noise is
sampled in numpy (releasing the full 1.2M-entry k×d vector through OpenDP's native
FFI sampler costs ~40–70s/step; numpy is ~ms). The privacy *accounting* is the
machine-checked OpenDP map — verified:
```
rho=0.1: rho_total=0.100000 rho_step=0.020000 n_releases=5 scale=56.569 L2sens=11.314 clip_L=128
rho=0.5: rho_total=0.500000 rho_step=0.100000 n_releases=5 scale=25.298 L2sens=11.314 clip_L=128
```
(`rho_step * n_releases == rho_total`; `sigma == sqrt(L*T/(2*rho))`.)

**Reproducible command:**
```
$PY -m dpclustbench.benchmark_audience_compare \
  --algorithms opendp_pe_means,google_lsh,dp_spherical_lloyd --runs 3 \
  --rho-values 0.1,0.5 --opendp-pe-no-public-structure \
  --dsl-iterations 5 --dsl-center-active-tags 96 --out results/final_compare.csv
```

**Results (3 runs, mean; compliant no-public-structure):**
| method | rho=0.1 ARI | rho=0.1 NMI | rho=0.5 ARI | rho=0.5 NMI | s/run |
|---|---|---|---|---|---|
| Google LSH | 0.090±0.02 | 0.150 | 0.086±0.02 | 0.145 | 1.1 |
| OpenDP PE | 0.121±0.05 | 0.183 | 0.140±0.02 | 0.202 | 65 |
| **DP spherical Lloyd** | **0.851±0.06** | **0.897** | **0.764±0.09** | **0.886** | **0.5** |

**Interpretation:** ~6–10× better ARI than the best baseline, and ~130× faster than
PE. The OpenDP-accounted numbers match the numpy simulation. NOTE the high run-to-run
variance (std 0.06–0.09) from random init: single runs can put rho=0.1 above rho=0.5.
Over 3 runs the two rho means are within one std — this is single-shot DP-Lloyd init
sensitivity (two centers can grab one true cluster), not a budget inversion. Report
mean±std; multi-restart would reduce it but costs budget.

**Next action:** stress-test whether the advantage is an artifact of the (easy)
default generator or a real, general improvement.

---

## Attempt 5 — Stress test across difficulty (is it overfitting the synthetic data?)

**Hypothesis:** the default generator is unusually easy (random-feature init reaches
ARI 1.0 non-privately → clusters are near-linearly-separable in tag space, likely
easier than the real ID Graph). Harder, more realistic structure may erode the
advantage. Vary: core_hit_prob ×0.4 (fainter signal), shared_core_fraction 0.15→0.5
(cluster overlap), k 16→40 (more/smaller clusters), and all combined.

**Results (`scratchpad/stress_test.py`, 3 seeds; ceiling = non-private TF-IDF cosine k-means):**
| preset | non-private ceiling | DPL rho=0.1 | DPL rho=0.5 |
|---|---|---|---|
| default | 0.972 | 0.843±0.06 | 0.879±0.09 |
| weak (cores ×0.4) | 0.782 | 0.292±0.08 | 0.747±0.02 |
| overlap (shared 0.5) | 0.866 | 0.658±0.05 | 0.750±0.03 |
| many_k (k=40) | 0.680 | 0.511±0.01 | 0.763±0.03 |
| hard (all) | 0.760 | 0.189±0.08 | 0.413±0.01 |

**Interpretation:**
- The ceiling drops as intended (0.97→0.68–0.76) — the presets are genuinely harder.
- At **rho=0.5** DP-Lloyd tracks the ceiling well across all but the hard preset
  (0.75–0.88 vs 0.68–0.97 ceiling) → still *algorithm-competitive*, near-ceiling.
- At **rho=0.1** the gap to ceiling widens sharply for faint/combined signal
  ("weak" 0.29 vs 0.78; "hard" 0.19 vs 0.76). Here the method is **noise-limited**:
  faint core sums (~0.4×) sit near the σ=56 floor and top-m selection admits spurious
  features. Halving the noise (rho=0.5) recovers "weak" to 0.75.
**Honest caveat:** the large margin on the default benchmark partly reflects that this
synthetic problem has clean, separable per-cluster cores that spherical k-means is
ideally suited to. The advantage narrows and becomes noise-limited as signal weakens
or budget shrinks. Real ID-Graph structure may be harder than "default" and closer to
"overlap"/"weak".

---

## Attempt 6 — Does PE degrade *more gracefully* than DP-Lloyd on hard data?

**Hypothesis (user + advisor):** the DP-clustering literature (HST/LSH/PE) is designed
to beat naive DP-Lloyd; PE might hold up better on harder, less-separable data even if
it loses on the easy default. Test the real OpenDP PE (compliant, no public structure,
benchmark-default config) against DP-Lloyd on the *same* `weak` and `hard` preset data.

**Results (`scratchpad/pe_on_hard.py`, 2 seeds):**
| preset | rho | PE ARI | DP-Lloyd ARI |
|---|---|---|---|
| weak | 0.1 | 0.006 | 0.326 |
| weak | 0.5 | 0.004 | 0.756 |
| hard | 0.1 | 0.004 | 0.248 |
| hard | 0.5 | 0.006 | 0.407 |

**Interpretation:** The premise does **not** hold here. PE *collapses* to ARI ≈ 0 on
the harder presets (worse than its ~0.12 on the easy default) — fainter cores + more
overlap + more clusters make its random-candidate search hopeless. DP-Lloyd retains
0.25–0.76. **The DP-Lloyd advantage widens, not narrows, on harder data.** PE/LSH do
not offer a graceful fallback on this sparse-categorical family (substantiated, not
assumed).

---

## Headline (DP spherical Lloyd, 10 runs, default generator, machine-accounted zCDP)

| rho | ARI mean±std | ARI range | NMI mean |
|---|---|---|---|
| 0.1 | 0.828 ± 0.066 | 0.70–0.90 | 0.873 |
| 0.5 | 0.837 ± 0.091 | 0.66–0.95 | 0.926 |

Correct budget ordering (rho=0.5 ≥ rho=0.1) once averaged over enough runs; single
runs vary due to random-init sensitivity.

---

## Conclusion

**What improved.** Replacing Private Evolution with a **zCDP spherical k-means
(DP-Lloyd)** raised ARI from ~0.06–0.14 (PE / Google LSH) to **~0.83** at both
rho∈{0.1,0.5} — a 6–10× gain — at ~130× lower runtime (0.5s vs 65s). Recipe:
public random-feature initialization (zero privacy cost), T=5 Gaussian releases of the
per-cluster feature-sum matrix (L2 sensitivity √L, machine-composed under adaptive
composition), and top-m sparse center projection to kill d-dimensional noise
accumulation. No count/cluster-size release (rule 6): spherical centers need only the
direction `normalize(sum)`.

**What failed / limits.** (1) Single-shot DP-Lloyd has meaningful run-to-run variance
(±0.07–0.09 ARI) from random init occasionally merging two centers onto one cluster.
(2) On deliberately harder generators (faint cores, high cluster overlap, k=40),
utility drops and, at rho=0.1, becomes **noise-limited** (weak: 0.29; hard: 0.19) —
faint core sums sit near the σ≈56 floor; doubling the budget (rho=0.5) recovers "weak"
to 0.75.

**Noise-limited vs algorithm-limited.** The original PE result was firmly
**algorithm-limited** (Attempt 0: near-zero noise barely changed ARI). DP-Lloyd on the
default data is neither strongly limited — it sits close to the non-private ceiling
(0.83 vs 0.97). As data hardens or budget shrinks, DP-Lloyd transitions to
**noise-limited** at rho=0.1 while remaining near-ceiling at rho=0.5.

---

## Attempt 7 — Improvement search (are we just picking the oldest method?)

**Framing.** The win is not "Lloyd beats fancy methods." It is (1) matching the
*geometry* — multi-hot categorical data lives in set/cosine space, whereas PE/HST/LSH
operate in Euclidean proposal/embedding space — and (2) releasing the *sufficient
statistic* of the natural latent-class model (per-cluster feature sums, clean √L
sensitivity). Lloyd is just the simplest solver for that objective. The interesting
levers are the objective and the init. Tested three, in the numpy sim (3 seeds):

**(A) Over-cluster (k'=2k) + postprocessing merge** — attacks init variance for free.
**(B) Soft/EM responsibilities** (softmax cosine) — the principled generalization.
**(C) DP TF-IDF** — one extra DP df release; idf downweights hot/shared tags.

| preset / rho | baseline | (A) over+merge | (B) soft EM | (C) +DP-idf |
|---|---|---|---|---|
| default 0.1 | 0.857 | 0.431 | 0.234 | 0.854 |
| default 0.5 | 0.896 | 0.517 | 0.199 | 0.842 |
| overlap 0.1 | 0.683 | 0.001 | 0.147 | **0.817** |
| overlap 0.5 | 0.766 | 0.001 | 0.240 | **0.802** |
| hard 0.1 | 0.209 | 0.098 | 0.007 | 0.002 |
| hard 0.5 | 0.440 | 0.330 | 0.011 | 0.265 |

**Interpretation.**
- (A) and (B) **regress everywhere**, for understandable reasons: over-clustering
  halves each cluster's row count → halves core-sum SNR, and merging noisy top-m
  centers can't recover it; soft assignment with noisy centers stays indecisive at
  the random-init symmetry point and collapses toward the global mean. **Under DP
  noise, hard assignment's symmetry-breaking and full per-cluster SNR are features,
  not naïveté.**
- (C) DP TF-IDF is **regime-dependent**: clearly helps when clusters share tags
  (overlap +0.13) — the realistic-data direction — neutral on the easy default, and
  harmful in the faint-signal hard case (budget spent on df + noisy idf). A useful
  optional lever, not a default.

**Not yet tested — where I'd invest next (with rationale):**
1. **Variance reduction / init** — the real open problem (±0.07–0.09 ARI). Principled
   fix: spend a small held-out budget to pick the best of R random restarts via
   report-noisy-max on a DP clustering objective (or DP k-means++ seeding). This is
   exactly where the literature's methods legitimately add value.
2. **Adaptive budget across iterations** — less on early (rough) steps, more on later
   (refinement) steps.
3. **One-shot DP feature screening** before Lloyd (cut d from 75,688 to a few
   thousand via a single noisy-marginal release) to de-noise top-m selection at low
   rho / hard regime.

**Conclusion of the search:** the two "smarter" objectives lose under DP; the only
lever that helps (DP TF-IDF) helps only on overlapping-cluster data. The simple
hard-assignment DP spherical Lloyd is a well-justified default, and the remaining
headroom is in initialization, not in the objective.

---

## Attempt 8 — Framework-ized OpenDP Library implementation

**Code added:** `python/src/opendp/extras/sklearn/cluster/_spherical_lloyd_binary.py`
(self-contained; depends only on the OpenDP core so it can merge independently of the
tree/HST estimators), exported from the cluster package `__init__.py`. Tests:
`python/test/extras/test_cluster/test_spherical_lloyd_binary.py` (8 tests, all pass).

Constructor-based, classless core (mirrors the PE constructors):
- `make_sparse_binary_domain(n_features, *, max_active)`
- `make_cluster_feature_sums(...)` / `then_cluster_feature_sums(...)` — √L-stable
  transformation from `symmetric_distance` into the L2 count metric.
- `make_private_cluster_feature_sums(...)` / `then_...` — one Gaussian release.
- `make_private_spherical_lloyd_binary(...)` / `then_...` — full mechanism (adaptive
  composition of T releases + public init + top-m postprocessing).
- `SphericalKMeans` (alias `SphericalLloydMeans`) — independent sklearn wrapper with
  `fit/predict/fit_predict/transform/fit_transform/score`, `rho` or `(epsilon,delta)`
  or explicit `noise_sigma` budget, and machine-verified accounting in `extra_`.

**Difference from the benchmark version:** the library uses OpenDP's **native vetted
Gaussian sampler** (`dp.m.then_noise`) with an exact `sqrt(L)` L2 sensitivity via a
`f64` count metric — not the benchmark's fast numpy noise. Correct-and-vetted over
fast: a full-`k*d` release with the secure sampler costs ~1 min/iteration at
d=75,688.

**Validation (real 50k data, native sampler, rho=0.5, seed 0):**
`ARI=0.927  NMI=0.947  rho_total=0.500 (== meas.map(1))  scale=25.30`, i.e. the
vetted-sampler path reproduces the strong utility of the simulation.

**Accounting verified by tests:** `rho_step * n_releases == rho_total == requested rho`;
per-release `rho = L/(2σ²)`; transformation stability `map(d_in) = d_in·√L`; requires
`dp.symmetric_distance()`.

---

## Conclusion (deliverable)

**On overfitting (the honest bottom line).** The default synthetic problem is easy
(separable cores; random-feature init reaches ARI 1.0 non-privately), so absolute
numbers likely overstate what to expect on the real ID Graph. But two facts argue the
improvement is *real and general* for this data family, not a benchmark artifact:
(a) a **single fixed** hyperparameter set (T=5, m=96, L=128) works across all five
difficulty presets at rho=0.5 with no re-tuning; and (b) DP-Lloyd **dominates PE at
every tested difficulty**, with the margin widening as the problem gets harder (PE→0,
DP-Lloyd holds). The right objective for sparse multi-hot categorical clustering is a
spherical/cosine mean, and DP-Lloyd realizes it under zCDP far better than the
candidate-search approaches.
