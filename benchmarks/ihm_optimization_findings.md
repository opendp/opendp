# IHM optimization ablation against OpenDP (2026-11-21)

Experimental only; **production sklearn branch unchanged**. Main baseline:
\`benchmarks/ihm_issp.py\`. The expanded reproducible NumPy/SciPy
scripts and dataset-level CSV results were also exported as a separate
research archive in the conversation.

## Protocol

- Original suite: 12 normalized bounded synthetic regressions plus
  Diabetes and Linnerud; n=20 to 2000, d=3 to 10.
- Matched approximate-DP budget, delta=1e-6 and a repeat at 1/n^2,
  eps=1,4,16, 300 Monte Carlo trials per dataset.
- IHM released-code update, authors' RDP sketch-noise calibration, exact
  Gaussian/Wishart marginal; recalibrate each public k, T and residual
  clipping bound C. No other privacy changes.
- Excess empirical prediction loss relative to nonprivate OLS.
  Results report median of per-dataset IHM/OpenDP loss ratios.
  Ratio below 1 means IHM is better.
- K = max(d, log(4T/varrho)), varrho=delta/10. Authors use k=6K.

## Original suite, delta=1e-6

| IHM config | eps=1 ratio (wins/14) | eps=4 | eps=16 |
|---|---:|---:|---:|
| Authors: T=3, k=6K, C=1 | 1.35 (1) | 2.20 (0) | 2.28 (0) |
| T=3, k=2K, C=.5 | .98 (8) | 1.21 (2) | 1.73 (0) |
| T=7, k=2K, C=.5 | .76 (13) | .78 (13) | .94 (10) |
| T=10, k=2K, C=.25 | **.71 (13)** | **.58 (13)** | .53 (13) |
| T=10, k=4K, C=.25 | .95 (9) | .93 (10) | **.63 (10)** |

T10/2K/C.25 loses catastrophically on one high-sample, high-epsilon
dataset (worst IHM/OpenDP ratio 18 at eps16). T10/4K/C.25 has a
smaller worst-case ratio (1.46). With delta=1/n^2 the
corresponding results are very similar.

Changes versus the authors' baseline:
1. Smaller sketch k: reduces required Wishart/inflation noise but too
   small k increases sketch variance and can destabilize inversion.
2. Smaller residual clipping C: reduces Gaussian gradient noise but
   may bias the fit toward a Huber-like objective.
3. More iterations: overcome heavy Hessian inflation and improve
   convergence, but require composition over more noise releases.

## Out-of-suite stress test

45 additional directly bounded synthetic datasets, d=5,10,20,
n=100,500,2000, with low signal, large coefficients, high residual
noise, 20% corrupted labels, and collinear designs. 150 trials.

| IHM config | eps=1 (wins/45) | eps=4 | eps=16 |
|---|---:|---:|---:|
| Authors | 1.17 (10) | 1.64 (4) | 3.99 (3) |
| T10, 2K, C=.25 | 1.04 (21) | 1.62 (16) | 5.02 (9) |
| T10, 4K, C=.25 | **.72 (32)** | **.93 (27)** | 1.88 (16) |

The tuned improvement is **not universal**. High residuals and
contaminated labels can make small C strongly biased; high epsilon
exposes this bias while OpenDP's noise diminishes. At eps4 the
tuned 4K variant has median ratios 1.47 on high-residual cases and
4.49 on contaminated-label cases. Its eps16 worst-case exceeds 3000.

## CRITICAL: paper versus code

Printed Algorithm 1 at
https://arxiv.org/html/2601.07545 includes
\`-eta**2 * theta_t\` in the clipped gradient. The public
\`omrilev1/HessianMix\` main experimental implementation omits it.
The successful ablations above optimize the **authors' released-code
update**, not the paper-printed algorithm. Both may be calibrated
through the same private sketch/gradient composition, but their
statistical fixed points are different.

In an independent 300-trial test, restoring this term makes the best
tested tuned paper-printed variant much worse: eps4 median
IHM/OpenDP excess-loss ratio 4.76 (0/14 wins). Do not transfer the
paper's original utility theorem to the successful code-style
variant without separate analysis.

## Conclusion

**IHM is substantially more competitive than the initial benchmark
suggested**; on benign datasets it can beat our current two-release
SSCP. Nevertheless, it is sensitive to publicly chosen T, k and C
and exhibits serious failures outside the original suite. Keep
OpenDP's current estimator unchanged; if pursuing IHM, prototype as
a distinct estimator with explicit tuning and a new utility analysis.
