# Exploratory linear regression utility work

This companion branch preserves the optimization experiments that were
removed from `sklearn-linear-regression-adassp` to keep its production
diff reviewable. It is **not** intended to be merged with the regression
PR.

The scripts under `python/test/extras/scratch_*.py` compare analytic
allocation, Laplace weighting, James-Stein denoising, and other
alternatives. See `python/test/extras/linear_regression_utility_notes.md`
for derivations and caveats.

`benchmark_vs_adassp_revised.py` independently implements Algorithm 3
of Aryanfard, Henzinger and Rostamkhani (2026), arXiv:2611.08737,
against the two-release joint Gaussian mechanism. It matches
`(epsilon,delta)` via the standard zCDP conversion
`epsilon = rho + 2 sqrt(rho log(1/delta))`.

On 14 bounded benchmark problems (12 synthetic, 2 bundled real
datasets), 300 Gaussian trials each, alpha=.05 and delta=1e-6:

| epsilon | Median paper/joint excess loss | Cases favoring joint |
|---|---:|---:|
| 1 | 1.62x | 12 / 14 |
| 4 | 4.94x | 14 / 14 |
| 16 | 6.84x | 14 / 14 |

The 20-row Linnerud example at epsilon=1 favored the paper by about
1.13x (ratio paper/joint = 0.888). Real-data preprocessing is performed
*before* the mechanism as a benchmark fixture and is not a DP operation;
this experiment is not an end-to-end private data pipeline. Results
depend on the chosen sample sizes, normalization, privacy budget, and
the paper's prescribed conservative regularization constants.

These are numerical comparisons, **not** a statistical utility theorem
for the two-release estimator. The published theorem applies to
AdaSSP-Revised, not to the eigenvalue-flooring variant.
