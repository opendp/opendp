.. _spherical-kmeans-analysis:

Private spherical k-means
=========================

:py:class:`~opendp.extras.sklearn.cluster.SphericalKMeans` clusters canonical
boolean SciPy CSR arrays: each row is a set of active features. The framework
entrypoint is :py:func:`~opendp.extras.sklearn.cluster.make_private_spherical_kmeans`.
The public feature universe must be specified or inferable from the array shape;
private row count is not inferred. Fitting supports both pure DP and zCDP.

Objective and update
--------------------

For a record with :math:`r_i` active features, define
:math:`a_i=1` without a cap, or :math:`a_i=\min(1,B/\sqrt{r_i})` with the
public ``max_l1_contribution=B``. Nonempty records contribute
:math:`u_i=a_i x_i/\sqrt{r_i}`; empty records contribute zero.
For unit center directions :math:`c_j`, the weighted spherical objective is

.. math::

    \max_{c,z}\sum_{i:r_i>0} a_i\left\langle
        \frac{x_i}{\sqrt{r_i}},c_{z_i}\right\rangle.

Assigning by cosine similarity maximizes each record's term. For fixed
assignments, the maximizing direction is the direction of the sum of these
weighted normalized records. Clamping a noisy sum nonnegative and retaining its
largest :math:`m` coordinates gives an optimal nonnegative :math:`m`-sparse
direction for that fixed clamped sum. Without noise, projection and quantization,
these are spherical Lloyd updates. Noise and numerical approximation mean that
the objective need not improve at every private iteration.

The cap uniformly downweights the entire record; it never removes selected
feature IDs. It has the same meaning under both privacy measures. The estimator's
``score`` reports the negative sum of ordinary, **unweighted** cosine distances,
including distance one for an empty record or zero center. Thus it remains
comparable across different training caps. Assignment ties select the first
center; top-coordinate ties prefer the lowest feature ID. Empty clusters are
updated from noise, rather than reseeded from private records.

Discrete arithmetic and privacy
-------------------------------

Internally, one int64 weight is computed per record:

.. math::

    w(r)=\begin{cases}
      0 & r=0,\\
      \lfloor Q/\sqrt r\rfloor & r>0,\ B=\mathrm{None},\\
      \min(\lfloor Q/\sqrt r\rfloor,\lfloor QB/r\rfloor) & r>0,\ B>0.
    \end{cases}

The first floor is computed as ``isqrt(Q * Q // r)`` using unbounded Python
integers. The cap is represented as an exact rational, including the exact
represented value of a floating-point argument. Neither calculation rounds up.
``X`` stays boolean CSR; a sparse assignment matrix ``A`` contains one row weight
at the assigned cluster, and aggregates are computed as ``A @ X``. No normalized
or weighted copy of the input matrix is materialized.

Let :math:`M=2^{63}-1`. Aggregation is defined as coordinatewise saturation of
the exact nonnegative sums at :math:`M`. Every sparse multiplication uses at most
:math:`\lfloor M/\max_i w_i\rfloor` records, so every intermediate sum fits int64
regardless of the sparse multiplication's summation order. Most datasets need one
multiplication. When several blocks are necessary, adding
``min(block, M - total)`` saturates **before** an addition could overflow. Clipping
an already-wrapped sparse product is never used.

Conditionally on previously released centers, assignment is a deterministic
per-record operation. Adding/removing a record changes exactly one cluster by
:math:`w(r)x`. Therefore

.. math::

    \Delta_2\le Q,\qquad
    \Delta_1\le\lfloor Q\sqrt d\rfloor,
    \qquad \Delta_1\le\lfloor QB\rfloor\quad\text{when capped}.

The uncapped L1 integer bound is ``isqrt(Q * Q * d)``; with a cap we take the
smaller of the two integer bounds. The capped L1 bound also bounds L2, so it
further tightens Gaussian calibration when B is below one. Coordinatewise
saturation is nonexpansive under both L1 and L2. For group distance :math:`g`, integer multiplication gives
:math:`g\Delta`; conversion to the mechanism's float64 distance type rounds
upward if necessary. These are conservative representable bounds, not a privacy
argument based on idealized floating-point aggregation. Float64 cosine arithmetic
only affects which cluster a record chooses, not its sensitivity bound.

OpenDP's discrete Laplace mechanism uses the L1 bound for pure DP; discrete
Gaussian uses the L2 bound for zCDP. Native integer mechanisms add noise in
arbitrary-precision integer arithmetic and saturate to int64. The fixed number of
iterations is adaptively composed. Equal per-iteration budgets and noise scales
are searched against OpenDP's conservatively rounded privacy maps, not obtained
from unchecked floating-point budget division. A substitution costs two symmetric
row edits. No ``idealized-numerics`` feature is needed.

Precision and feasibility
-------------------------

The public internal power-of-two scale :math:`Q` is at least :math:`2^{32}` and at
least :math:`2^{24}\lceil\sqrt d\rceil`; with a cap it is also at least
:math:`2^{24}\lceil d/B\rceil`. Consequently every possible nonempty record has
weight at least :math:`2^{24}`. Its per-coordinate relative quantization error is
less than :math:`2^{-24}`. This includes very large feature universes and does not
require a public bound on dataset size. Configurations requiring a scale above
int64 capacity are rejected before accessing private records. The scale is not an
estimator parameter.

Saturation can distort directions on extremely large datasets; the coordinate
ceiling in normalized units is :math:`M/Q`. Correctness and privacy still hold for
unsized datasets, including those needing the saturating fallback. No promise is
made that saturation is statistically negligible for every possible dataset.

Input aggregation is sparse, but the noise vector still contains
``n_clusters * n_features`` integers. Work and peak memory therefore scale with
the public feature universe, not just the number of observed features. A
trillion-dimensional CSR schema can have well-defined weights while a dense noise
release over that universe remains infeasible. This implementation does not
privately select a smaller feature universe.

Initialization and practical limits
-----------------------------------

``init="random"`` builds a public feature-to-cluster random partition, covering
every feature rather than sampling a few feature IDs. It is independent of the
private records and their order. Alternatively supply public/already-DP
nonnegative center directions as a dense array or sparse matrix. Supplied buffers
are snapshotted; never use unaccounted private records as initial centers.
``random_state`` controls public initialization only, never DP noise.

Private initialization is not implemented. Rare clusters, many clusters, weak
signals and inadequate budgets can still produce poor utility. Without an L1
cap, pure-DP noise scales with :math:`\sqrt d`; normalized records already have
bounded L2 sensitivity, so zCDP needs no cap for privacy.

Utility evaluation
------------------

The manual benchmark at ``python/benchmarks/spherical_kmeans.py`` compares the
previous PR's binary-count/truncated update and sparse random initialization with
normalized updates, public feature-partition initialization, and supplied public
directions. It uses only synthetic public data and runs each fit in a fresh
process. Seeds never fix DP noise. The results below record the evaluated configurations and selected defaults.

Recorded synthetic results
~~~~~~~~~~~~~~~~~~~~~~~~~~

These are illustrative observations, not a guarantee or a tuned model-selection
procedure. Each number is the mean of three fresh-process fits, using public
initialization seeds 0, 1 and 2 and independently generated DP noise. All datasets
have 3,000 records, three clusters, and 32 informative features per cluster at
low, middle and high feature IDs. Mixed-length data has 2,048 features and record
lengths 8, 32 or 256, chosen uniformly. The unequal-cluster variant has cluster
sizes 2,400/500/100 rather than 1,000/1,000/1,000. The high-dimensional variant
has 20,000 features and equal-length 16-feature records. Every record contains
eight randomly chosen informative features; remaining features are background.
Public supplied directions are the generator's feature sets, not estimates from
sampled records.

Pure-DP runs use epsilon=4; zCDP runs use rho=0.5, each over five iterations.
The previous PR baseline is ``ddb274a56`` with only sklearn descriptor adapters;
its algorithm AST was verified unchanged. Because its naive allocation can
slightly exceed the nominal budget through rounding, benchmark-only calibration
slack of 1e-12 is applied and its actual privacy map is checked against the same
outer allowance. New constructors use rounded calibration directly.

The full results (accuracy, balanced accuracy, mean maximum cosine similarity,
fit time, traced allocation peak and process peak RSS) are available as
:download:`CSV <spherical-kmeans-benchmark.csv>`. The common spherical objective
reported here is unweighted mean maximum cosine similarity, larger is better;
capped training itself optimizes a weighted variant. Accuracy uses an optimal
permutation of cluster labels; balanced accuracy averages recall over clusters.

Initialization isolated from the update (zCDP)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

All normalized variants in this table have unrestricted centers and no L1 cap.
Thus sparse versus partition versus supplied initialization changes only the
public starting directions. The old PR also uses its old update and defaults.

.. list-table::
   :header-rows: 1

   * - Dataset
     - Update / initialization
     - Balanced accuracy
     - Cosine objective
   * - Mixed
     - Old PR / sparse
     - 0.693
     - 0.212
   * - Mixed
     - Normalized / sparse
     - 0.561
     - 0.312
   * - Mixed
     - Normalized / partition
     - 0.629
     - 0.314
   * - Mixed
     - Normalized / public
     - 1.000
     - 0.304
   * - Unequal
     - Old PR / sparse
     - 0.651
     - 0.246
   * - Unequal
     - Normalized / sparse
     - 0.453
     - 0.339
   * - Unequal
     - Normalized / partition
     - 0.600
     - 0.338
   * - Unequal
     - Normalized / public
     - 0.970
     - 0.305
   * - High-dimensional
     - Old PR / sparse
     - 0.333
     - 0.203
   * - High-dimensional
     - Normalized / sparse
     - 0.389
     - 0.192
   * - High-dimensional
     - Normalized / partition
     - 0.889
     - 0.285
   * - High-dimensional
     - Normalized / public
     - 1.000
     - 0.301

Pure-DP cap and initialization tradeoffs
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

"Capped" means B=4; no center-sparsity limit is applied to the new variants.

.. list-table::
   :header-rows: 1

   * - Dataset
     - Configuration
     - Accuracy
     - Balanced accuracy
     - Cosine objective
   * - Mixed
     - Old PR
     - 0.422
     - 0.422
     - 0.058
   * - Mixed
     - Partition
     - 0.473
     - 0.473
     - 0.116
   * - Mixed
     - Partition, capped
     - 0.582
     - 0.582
     - 0.231
   * - Mixed
     - Public, capped
     - 1.000
     - 1.000
     - 0.251
   * - Unequal
     - Old PR
     - 0.612
     - 0.514
     - 0.086
   * - Unequal
     - Partition
     - 0.728
     - 0.501
     - 0.130
   * - Unequal
     - Partition, capped
     - 0.668
     - 0.616
     - 0.274
   * - Unequal
     - Public, capped
     - 0.696
     - 0.749
     - 0.277
   * - High-dimensional
     - Old PR
     - 0.412
     - 0.412
     - 0.011
   * - High-dimensional
     - Partition
     - 0.441
     - 0.441
     - 0.020
   * - High-dimensional
     - Partition, capped
     - 0.667
     - 0.667
     - 0.155
   * - High-dimensional
     - Public, capped
     - 1.000
     - 1.000
     - 0.163

Defaults and resource costs
^^^^^^^^^^^^^^^^^^^^^^^^^^^

The public partition covers the full universe and avoids the zero-overlap
problem of the previous sparse initializer. With the normalized update, it
improved high-dimensional zCDP balanced accuracy from 0.389 to 0.889 here; it does
not universally improve every dataset or privacy setting. Supplied public
centers can be considerably better, particularly for rare clusters. No extra
initialization strategies are exposed by the estimator.

The default ``max_features_per_center=None`` avoids imposing an arbitrary feature
support prior. In zCDP, unrestricted/32-coordinate/128-coordinate cosine objectives
were respectively 0.314/0.279/0.243 on mixed data, 0.338/0.277/0.275 on unequal
clusters, but 0.285/0.354/0.348 in the high-dimensional case. Sparse projection
improved the latter substantially while hurting other objectives. Choose it when
a sparse-center prior is appropriate; it is not universally optimal. Similarly,
the default cap is None to retain the ordinary normalized spherical objective;
the cap is an explicit, useful modeling and privacy/utility choice for pure DP.

The rewrite prioritizes safe arithmetic and rounded accounting rather than a
speed claim. Relative to the old implementation it performs more calibration and
retains int64, potentially dense centers. Representative mean fit times and
traced peak allocations for the default partition initializer are below; the CSV
also records process-wide peak RSS, including the interpreter/native libraries.
Traced allocations do not capture every native allocation.

.. list-table::
   :header-rows: 1

   * - Dataset / measure
     - Old / new fit time (s)
     - Old / new traced peak (MiB)
   * - Mixed / epsilon
     - 0.059 / 0.158
     - 4.86 / 5.64
   * - Mixed / rho
     - 0.116 / 0.219
     - 4.86 / 5.65
   * - Unequal / epsilon
     - 0.062 / 0.168
     - 4.86 / 5.64
   * - Unequal / rho
     - 0.117 / 0.218
     - 4.86 / 5.64
   * - High-dimensional / epsilon
     - 0.365 / 0.733
     - 2.61 / 8.73
   * - High-dimensional / rho
     - 0.902 / 1.256
     - 2.61 / 8.06

Measured on macOS/Apple Silicon with Python 3.12.12, NumPy 2.5.0, SciPy 1.17.1 and scikit-learn 1.8.0.
These small synthetic cases and three repetitions are not evidence of universal
accuracy, runtime or memory superiority. In particular, the unbounded pure-DP
sensitivity in a large universe can overwhelm signal without a cap, and a public
partition is not a substitute for domain-informed public initialization.
