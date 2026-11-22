# IHM and ISSP benchmark against the two-release OpenDP estimator

**Experimental only; no production changes.**

Paper: Lev et al. (ICML 2026), https://arxiv.org/html/2601.07545
Authors' implementation: https://github.com/omrilev1/HessianMix
ISSP: Brown et al. (COLT 2024), https://arxiv.org/html/2404.15409

Run from the repository root with NumPy, SciPy and sklearn:

    OPENBLAS_NUM_THREADS=1 python benchmarks/ihm_issp.py --trials 300 --output ihm_issp_results.csv
    OPENBLAS_NUM_THREADS=1 python benchmarks/ihm_issp.py --trials 200 --delta-policy n-2 --output ihm_issp_n2.csv

The suite comprises 12 synthetic designs over (n,d)=(100,5),(500,5),
(500,10),(2000,10), with isotropic/moderate/collinear spectra, plus
sklearn Diabetes and Linnerud datasets. Data are publicly centered and
normalized to ||x||<=1, |y|<=1. This preprocessing is **not differentially
private** if applied to confidential data.

Metric: average normalized empirical excess prediction loss relative to
ordinary least squares, 300 or 200 Monte Carlo trials, no intercept.
IHM's Gaussian sketches are sampled via an exactly equivalent Wishart
marginal. IHM uses the paper's T=3, k=6*max(d,log(4T/varrho)) choices,
varrho=delta/10, authors' RDP sketch calibration and analytic Gaussian
clipped-gradient noise. OpenDP uses native zCDP converted to (eps,delta).

| delta | eps | Median IHM/OpenDP loss | IHM wins |
|---|---:|---:|---:|
| 1e-6 | 1 | 1.37x | 1/14 |
| 1e-6 | 4 | 2.13x | 0/14 |
| 1e-6 | 16 | 2.25x | 0/14 |
| 1/n^2 | 1 | 1.36x | 1/14 |
| 1/n^2 | 4 | 2.10x | 0/14 |
| 1/n^2 | 16 | 2.13x | 0/14 |

Ratio > 1 favors OpenDP. Linnerud at eps=1 favors IHM. Results are
not universal; IHM was shown to outperform other, less optimized
AdaSSP variants by the paper's authors.

**Paper/code discrepancy:** IHM Algorithm 1 includes an
-eta^2*theta_t term in the clipped gradient that is omitted by the
authors' released experiment script. We report the authors' code as
the main IHM baseline and provide the printed-paper variant as a
separate sensitivity analysis. The paper-printed variant performs
considerably worse in much of this suite.

**ISSP:** There is no valid successful ISSP fit in these experiments.
ISSP Algorithm 1 prescribes k=ceil(12log(3/delta)/eps)+8 and
L0 <= min(1/(96k),3eps/(56log(12/delta))). Any positive-weight,
full-rank design with n records has max leverage >= d/n.
Thus n >= d/L0 is a **necessary**, not sufficient, eligibility
condition. Every benchmark dataset violates it. For delta=1e-6,
minimum required n values at d=10 are 179520 (eps=1),
50880 (eps=4), and 19200 (eps=16). The paper's main
utility theorem assumes eps<1; eps>=1 entries are diagnostics
of its stated Algorithm 1 thresholds, not theorem instances.
No ISSP oracle noise was passed off as a DP algorithm.

This is a NumPy benchmark, not an end-to-end compiled OpenDP test.
No test-time or data-dependent hyperparameter tuning was used.
