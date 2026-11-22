# Private linear regression: utility experiments (scratch)

This note is exploratory. The production privacy proof continues to rely on
the certified SSCP and sum transformations and on native Gaussian/Laplace noise.
Do not confuse a smaller coefficient-MSE estimate with a privacy theorem.

## Deployed in the regression branch

**One-sided Gaussian tail.** If the \`svec\` noise has standard deviation
\(\sigma\), the reconstructed symmetric Gaussian noise is \(E=(Z+Z^T)/2\)
for iid entries \(Z_{ij}\sim N(0,\sigma^2)\).
Gaussian comparison gives \(\mathbb E\lambda_{\max}(E)\le
\sigma\sqrt{2d}\); concentration of the 1-Lipschitz maximum-eigenvalue
functional yields, with probability at least \(1-\alpha\),

\[
\lambda_{\min}(E)\ge -\sigma\min\left\{
\sqrt{2d}+\sqrt{2\log(1/\alpha)},
\sqrt{(d+1)\log(d/\alpha)}
\right\}.
\]

The second option follows directly by evaluating the Gaussian matrix mgf.

**One-sided Laplace tail.** For an \`svec\` release with iid Laplace(b)
coordinate noise, the reconstructed diagonal entries have scale b and
off-diagonal entries scale \(b/\sqrt 2\). Decompose the symmetric noise into
independent rank-one diagonal and rank-two off-diagonal summands. Their
matrix variance is

\[
V=\sum\mathbb E X_k^2=b^2(d+1)I.
\]

The independent summands give an exact matrix log-mgf upper bound

\[
K(u)=-\log(1-u^2)-(d-1)\log(1-u^2/2),\quad 0<u<1.
\]

The matrix Laplace-transform inequality yields

\[
\Pr\{\lambda_{\max}(-E)>b[t+K(u)]/u\}\le de^{-t}.
\]

Use \(t=\log(d/\alpha)\) and the public choice
\(u=\sqrt{2t/(d+2t)}\). This is close to the numerically optimized
Chernoff parameter but is noniterative. The implementation also takes
the minimum with the entrywise union bound. Both bounds are valid
individually at failure probability alpha. Reference: J. A. Tropp,
*User-Friendly Tail Bounds for Sums of Random Matrices* (2012),
DOI 10.1007/s10208-011-9099-z.


**Eigenvalue flooring.** For a released symmetric matrix
\(\widetilde G=Q\operatorname{diag}(\lambda)Q^T\), solve using
\(Q\operatorname{diag}(\max(\lambda_i,B)^{-1})Q^T\widetilde b\)
instead of shifting *every* eigenvalue by a common ridge penalty.
This is privacy-preserving postprocessing. It preserves already-strong
directions but does not dominate ridge in every statistical risk measure.

**Public count.** When \`fit_intercept=True\` and dataset size is public,
the bottom-right entry of the augmented Gram is exactly n. Replacing its
noisy value by n is free postprocessing. For private n, the same entry
already contains a noisy count, so no extra column is needed just for that.

## Benchmarks / deferred alternatives

Run \`python python/test/extras/scratch_linear_regression_utility.py\`.
The script uses a known diagonal population Gram and draws privatized
statistics directly. It does *not* call OpenDP or test data-independent
errors, model consistency, clipping, or utility on real datasets.

**Analytic split.** For isotropic G and *public reference* theta norm,
first-order mean squared coefficient error has the form
\(A/\rho_G+B/\rho_b\) for zCDP, so
\(\rho_G/\rho=\sqrt A/(\sqrt A+\sqrt B)\).
For pure DP it has the form \(A/\varepsilon_G^2+B/\varepsilon_b^2\),
so \(\varepsilon_G/\varepsilon=A^{1/3}/(A^{1/3}+B^{1/3})\).
These optima depend on the unknown regression coefficients and geometry.
They are kept in scratch rather than silently adopted as defaults.

**Joint fixed-size substitution sensitivity (now used for Gaussian).**
We concatenate the Gram and cross-product vectors after dividing each by
its certified per-replacement sensitivity. For a single row replacement,
let \(\|x\|,\|x'\|\le R\), \(|y|,|y'|\le Y\).
With full radii and labels, \(t=|\cos(x,x')|\), the squared normalized
L2 distance is

\[
\frac{\|xx^T-x'x'^T\|_F^2}{2R^4}
+\frac{\|xy-x'y'\|_2^2}{4R^2Y^2}
\le (1-t^2)+\frac12(1+t).
\]

This quadratic attains its maximum \(25/16\) at \(t=1/4\).
The full-radius restriction is without loss: with radii \(a,c\in[0,1]\),
the unnormalized objective is
\(A(a^4+c^4-2a^2c^2t^2)+B(a^2+c^2+2ac\,t)\), \(A,B>0\).
Maximizing over t gives either
\(A(a^4+c^4)+B(a^2+c^2)+B^2/(2A)\), increasing in a,c,
or \(A(a^2-c^2)^2+B(a+c)^2\).
In the second regime \(B\ge2Aac\), its derivative in a is
\(2(a+c)[B+2Aa(a-c)]\ge0\), likewise for c.
Thus the supremum occurs at a=c=1. This is a valid upper bound
even when the appended intercept restricts the set of possible rows.

So the one-replacement L2 sensitivity of the normalized vector is
\(5/4\), not \(\sqrt2\) from separate composition. For groups of k
replacements, the triangle inequality gives \(5k/4\).
For add/remove adjacency the joint bound is \(\sqrt2\) per edit;
for Laplace L1 it is 2 per edit/replacement. Those reproduce
the old composition-equivalent noise scales. In the fixed-size
Gaussian case the new common vector mechanism reduces both
Gram and cross-product noise variance by \(1-25/32=21.875\%\),
relative to equal-budget separate composition. No trusted private
metadata is used to select between the two public adjacency regimes.


**Synthetic slack feature.** Let
\(s(x)=\sqrt{R^2-\|x\|_2^2}\) and release the SSCP of \([x,s(x)]\).
Its Frobenius per-row sensitivity stays \(R^2\); Gaussian noise can
therefore reveal additional Gram entries with no extra budget, including
a noisy trace-based count \(nR^2\). It does *not* improve the original
Gram's independent Gaussian coordinates automatically. For public n,
projecting the extra Gram's trace to \(nR^2\) reduces expected original
Gram Frobenius noise by only \(2/(d+1)^2\) relative to the original Gram
noise, under iid svec Gaussian perturbations. Laplace L1 sensitivity
generally increases, so this is not a free Laplace augmentation.

**Augmenting with y.** A single SSCP of \([a x,b y]\) also provides Gram,
cross-product, and target second moment, but the L2 radius and matrix
sensitivity increase. In a simple add/remove isotropic linearized-MSE
calculation (\(R=Y=\|\theta\|=1\)), the best rescaled augmented-SSCP
release has about 15% larger coefficient MSE than optimally splitting
two independent Gaussian releases at d=10. It is not a free win.

**Laplace weighted triangle.** A diagonal weight 1 and off-diagonal
weight \(w\) has generic L1 sensitivity
\(\Delta_F\sqrt{d+d(d-1)w^2/4}\). Matrix Bernstein's one-sided
spectral bound may slightly favor \(w<\sqrt2\), even though \`svec\`
(\(w=\sqrt2\)) minimizes Frobenius mean-squared error. Around d=10
this reduces the spectral margin by about 5% at best. Benchmark the
coefficient MSE before adding another production representation.


**Unpenalized intercept via Schur complement.** When n is public, one
could separately compute the intercept as
\((\widetilde{\sum y}-\widetilde{\sum x}^T\hat\beta)/n\)
while flooring only the noisy centered feature Gram. Experiments show
that this can reduce intercept-specific bias when n is small, but can
*increase total coefficient MSE* through noisy centering. For example,
with 10 centered features, n=30 and a unit true intercept, a small
Gaussian simulation found total MSE about 0.14 with flooring all
coordinates and about 0.21 with the unpenalized-intercept variant.
Keep the public-count correction, but do not deploy block elimination
as a universal regression default.


For the same equal-budget fixed-size comparison, the scratch Monte Carlo
uses svec Gaussian SD \(\sqrt2/\sqrt\rho\) and cross-product SD
\(2/\sqrt\rho\) for independent composition, then multiplies both SDs
by \(5/(4\sqrt2)\) for the joint vector release. At d=10,
\(G=30I\), \(\rho=1\), an earlier 1,000-trial experiment found
coefficient MSE about 0.059 (composition) versus 0.046 (joint),
consistent with the 21.875% noise-variance reduction.


## Allocating the joint-release noise between statistics

The existing code weights normalized \`svec(Gram)\` and \`X.T@y\`
equally and uses a single Gaussian vector mechanism. For **fixed-size
row substitution**, we can weight the cross statistic by a public
factor \(w\), with output

\[
F_w(D)=\left(
 \frac{\operatorname{svec}(X^\top X)}{\sqrt2R^2},
 w\frac{X^\top y}{2RY}
\right).
\]

Its **exact one-replacement L2 sensitivity** (in dimension >= 2,
without additional intercept constraints) is

\[
S(w)=
\begin{cases}
1+w^2/4,&0<w\le2,\\
w,&w\ge2.
\end{cases}
\]

This follows by optimizing \(1-t^2+\tfrac{w^2}{2}(1+t)\) over
\(0\le t\le1\), where \(t\) is the absolute angle cosine between two
full-radius replacement rows. Thus Gaussian standard deviations at
zCDP budget \(\rho\) are

\[
\sigma_G=\frac{R^2 S(w)}{\sqrt\rho},\quad
\sigma_b=\frac{\sqrt2 RY S(w)}{w\sqrt\rho}.
\]

Current \(w=1\) recovers \(S=5/4\). The equal-weight choice is
convenient but not generally statistically optimal.

For isotropic \(G=kI\), let \(r=w^2/2\). First-order coefficient
MSE is proportional to

\[
(1+r/2)^2\left[
 \frac{d+1}{2} R^4\|\theta\|^2
 +\frac{dR^2Y^2}{r}
\right].
\]

Its minimizer \(r^*\in(0,2]\) solves
\(A(r^*)^2+\frac B2r^*-B=0\), where
\(A=(d+1)R^4\|\theta\|^2/2\) and \(B=dR^2Y^2\).
For d=10, R=Y=||theta||=1, this favors \(w\approx1.39\)
rather than w=1.

In the **pure-DP Laplace** joint release, assigning relative weight
\(w\) to the normalized cross statistic instead results in L1
sensitivity \(1+w\) (a valid generic upper bound), corresponding
to Gram and cross Laplace scales \(b_G=(1+w)\Delta_G/\varepsilon\),
\(b_b=(1+w)\Delta_b/(w\varepsilon)\).
For isotropic linearized MSE, the optimal public reference weight is
\(w=(B/A)^{1/3}\) with
\(A=(d+1)\Delta_G^2\|\theta\|^2\) and
\(B=2d\Delta_b^2\).

**Do not hard-code either analytic optimum yet.** Both depend on
the unobserved coefficients and spectrum. With Gaussian d=10,
\(G=30I\), \(\rho=1\), independent 2,500-trial checks gave
MSE approximately 0.0460 at the current w=1 and 0.0411 at
w=sqrt(2) (10.6% lower). At \(G=30I\) with ||theta||=2,
the same change *increased* MSE from about 0.0762 to 0.0858
(12.7% worse). The sampling design was an isotropic Gram and
noiseless bounded targets; results are not universal risk bounds.

The scratch script \`scratch_joint_allocation.py\` makes the joint
allocation and a reference-theta-optimal alternative reproducible.
It is intentionally separate from the public sklearn estimator.

## Gaussian matrix denoising through positive-part James-Stein (scratch)

Let \(H=G+E\), where the coordinates of \(\operatorname{svec}(E)\)
are independent \(N(0,\sigma_G^2)\), and let
\(m=d(d+1)/2\). Split the matrix into its noisy trace
\(\bar h=\operatorname{tr}(H)/d\) and traceless residual
\(T=H-\bar h I\).

The traceless subspace has orthonormal dimension \(m-1\), under
the Frobenius inner product. For \(d\ge3\), positive-part
James-Stein shrinkage therefore gives

\[
\widehat G_{\rm JS}
=\bar h I+
 \max\left\{0,\;
   1-\frac{(m-3)\sigma_G^2}{\|T\|_F^2}
 \right\}\,T.
\]

For the *raw matrix release*, this uniformly reduces
\(\mathbb E\|\widehat G-G\|_F^2\) relative to \(H\)
for every fixed true \(G\). The trace component is unchanged.

**It does not uniformly dominate in regression coefficient MSE.**
An exploratory 900-trial Gaussian experiment at d=10, rho around 1,
R=Y=1 and eigenvalue flooring showed:

- \(G=30I\): 23.6% smaller coefficient MSE.
- Eigenvalues 10--30: 16.8% smaller coefficient MSE.
- Eigenvalues 10--100: 5.7% smaller coefficient MSE.
- Eigenvalues 3--9: **2.8% larger** coefficient MSE.

Furthermore, when the last Gram diagonal is replaced by the public
count, the noise is no longer isotropic; apply the shrinker to the
original noisy Gram *before* enforcing the count constraint if using
its Gaussian risk analysis. No uniform coefficient-risk guarantee
follows from the matrix-risk theorem in either case. Keep the
experiment out of production until tested against realistic data.

## Scalar specialization

For \(d=1\), \`svec\` is simply the Gram scalar and the one-sided
spectral noise tail is exact. For \(\alpha<1/2\), its positive
\((1-\alpha)\)-quantile is:

- Gaussian: \(\sigma[-\Phi^{-1}(\alpha)]\).
- Laplace: \(b\log[1/(2\alpha)]\).

At \(\alpha=0.05\), these are \(1.645\sigma\) and \(2.303b\),
versus much more conservative general-purpose matrix bounds.
For \(\alpha\ge 1/2\), retain the positive matrix concentration bound
to avoid zero or negative floors.


For **one-column fixed-size SSCP**, \(\|x^2-y^2\|\le R^2\)
under Frobenius, L1, and nuclear metrics; the prior general matrix
bound \(\sqrt2R^2\) was unnecessary. This scalar replacement
certificate is now applied in \`make_np_sscp\` for all output metrics.


## Improved joint Laplace substitution bound (implemented)

For fixed-size datasets, the **two normalized blocks** satisfy the
previously proved joint Gaussian sensitivity inequality

\[
a^2+b^2\le 25/16,
\]

where \(a\) is the Frobenius norm of the Gram change divided by its
per-replacement sensitivity, and \(b\) is the L2 norm of the
cross-product change divided by its per-replacement sensitivity.

The Laplace coordinates are separately normalized using L1 sensitivity
bounds of \(\sqrt m\) and \(\sqrt d\) times their respective L2 bounds.
Thus the **entire concatenated L1 change** is bounded by

\[
\|\Delta F\|_1\le a+b\le\sqrt{2(a^2+b^2)}
\le 5\sqrt2/4\approx 1.7678.
\]

The previous generic composition-style bound was 2. This reduction
lowers each Laplace noise scale by 11.6% and its variance by 21.9%
at equal pure-DP privacy budget. Unsized add/remove adjacency still
uses 2, which is needed for the generic bound.

When the total Gram dimension is 1, one has, after normalization,
\(\Delta G=|x^2-z^2|\) and
\(\Delta b=|xy-zy'|/2\), where \(|x|,|z|,|y|,|y'|\le1\).
Writing \(a=|x|\ge c=|z|\) and maximizing labels,

\[
\Delta G+\Delta b\le (a^2-c^2)+(a+c)/2.
\]

This is increasing in a, so a=1; the remaining quadratic
\(3/2+c/2-c^2\) reaches \(25/16\) at c=1/4.
It is tight. For the scalar Gaussian joint L2 bound,
\((1-c^2)^2+(1+c)^2/4\le
5/4+c/2-3c^2/4\le4/3\), giving sensitivity \(\sqrt{4/3}\).
These scalar improvements apply only under fixed-size substitution.


## Bounded-design utility ablation (scratch)

Run \`scratch_bounded_regression_utility.py\`. It constructs 72 synthetic
clipped datasets: dimensions 5 and 10; n=100,300,1000; isotropic,
moderately anisotropic, collinear, and approximate-low-rank designs;
coefficient norm 0.5,1,2; row radius R=1, labels clipped to Y=1.
All mechanisms use fixed-size Gaussian substitution accounting with
rho=1 and **paired random draws**, and compare errors against the
nonprivate OLS solution on each *clipped* dataset. These are empirical
experiments, not a new privacy or utility proof.

A separate 500--1000 trial sweep of the same designs showed:
- Increasing the normalized cross weight from 1 to **1.1** reduced
  median coefficient MSE by about 4.9% and median prediction loss
  by about 6.1%; the worst prediction loss change was about -0.4%.
- Weight **sqrt(2)** increased median predicted utility further
  (about 14.7% lower prediction loss), but harmed some scenarios
  by around 10%. It should not be the unconditional default.
- Traceless **James--Stein** alone usually changed prediction loss
  very little and was as much as 18% worse in the worst tested case,
  notwithstanding its uniform *matrix*-MSE advantage.
- Increasing the spectral floor to **1.5 times** the certified noise
  margin improved coefficient MSE in many near-singular scenarios,
  but worsened other scenarios by about 9.5%. Reducing the floor
  was often dramatically harmful due to unstable inversion.

The nonprivate coefficient norm and Gram eigenstructure are unknown
at calibration time. Keep these alternative choices experimental
until a robust public selection criterion is justified. In particular,
do not select the best weight *after* viewing private data or private
benchmark outcomes without additional privacy accounting.


## Fixed-size Laplace empirical outcome

The standalone \`scratch_laplace_utility.py\` uses the same 72 clipped
synthetic datasets as the Gaussian benchmark and compares the old
joint sensitivity 2 against the proved \(5\sqrt2/4\), under
pure-DP epsilons 4, 8, and 16. Noise realizations are paired.
The one-sided spectral margin scales consistently with the noise.

At epsilon 8, across 72 datasets, **median empirical excess
prediction-loss reduction was 13.2%**, median coefficient-MSE reduction
was 4.1%, and all 72 cases improved prediction loss. At epsilons 4
and 16, the median prediction-loss reductions were 11.6% and 13.9%.
This is empirical support, not a uniform prediction-loss guarantee:
the deterministic sensitivity result only guarantees less injected
noise at the same privacy budget.
