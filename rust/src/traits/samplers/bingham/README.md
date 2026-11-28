# Certified rounded Bingham sampling

For finite input entries interpreted as exact dyadics, let \(C\) denote the
symmetric part of the supplied matrix. The target density with respect to
uniform sphere measure is proportional to \(\exp(\kappa u^T C u)\).
Translation of \(C\) by a scalar identity does not change this density.

Set \(P=bI+2\kappa(LI-C)\) using exact rationals, where \(b,L\) and an
approximate factor \(B\) are untrusted numerical hints. Let
\(G=B^TPB\) and \(H=B^TB\). The Gram matrices need only be enclosed. For approximate matrices
\(A_0,B_0\) with maximum absolute entries \(a,b\) and uniform entrywise
errors \(e_A,e_B\), the exact product differs from \(A_0B_0\) by at most
\(d(e_A(b+e_B)+a e_B)\) per entry. The conversion errors are computed
exactly. Rust evaluates each dot product with ordinary IEEE-754 binary64
multiplications and additions, without reassociation. Each rounding
has error at most \(\epsilon |x|+\eta\), where \(\epsilon=2^{-52}\)
and \(\eta=2^{-1074}\); the additive term accounts for gradual underflow.
For \(2d\epsilon\le1/2\), expanding the at most \(2d\) roundings gives
a further error at most \(4d\epsilon\,d a b+4d\eta\):
\((1+\epsilon)^{2d}-1\le4d\epsilon\) and
\((1+\epsilon)^{2d}\le2\).
Nonfinite products cannot certify anything.
For each symmetric exact Gram matrix, enlarge the Gershgorin
radius of the approximate matrix by \(d\) times the product error.
If this does not certify a positive lower bound on \(G\), compute
the Gram matrices exactly by lifting the dyadic entries to common
power-of-two denominators and multiplying integer numerators.
The Gershgorin certificate \(G\succeq mI\),
\(m>0\), implies that \(P\) is positive definite and \(B\) is nonsingular.
If the certificate fails, use \(B=I,b=d\) and a Gershgorin upper bound
on the eigenvalues of \(C\). This always yields a positive certificate.

Draw independent standard normals \(z\), let \(v=Bz\), and let
\(u=v/\|v\|\). Define \(x=z^TGz/(d z^THz)\) and
\(y=m z^Tz/(d z^THz)\). Since \(0<y\le x\),
\(a=(y\exp(1-x))^{d/2}\le1\). The angular central Gaussian proposal
density is proportional to
\((u^TB^{-T}B^{-1}u)^{-d/2}\), so multiplication by \(a\) gives a density
proportional to \(\exp(-u^TPu/2)\), the target. Rejection terminates
almost surely since the acceptance probability is positive.

Standard normals use the exact Karney decomposition. Draw a nonnegative
integer \(K\) with mass proportional to \(\exp(-K^2/2)\), and an
independent uniform fractional part \(X\in(0,1)\). Accept the pair with
probability \(\exp(-KX-X^2/2)\), using exact decreasing-uniform
Bernoulli factories, then draw an independent fair sign.
The resulting density of \(S(K+X)\) is proportional to
\(\exp(-z^2/2)\). The integer proposal uses a geometric law of mass
proportional to \(\exp(-K/2)\) and accepts with probability
\(\exp(-(K-1/2)^2/2)\). The combined exponent is \(-K^2/2-1/8\).
For a fixed fractional part \(x\), the probability of completing
\(n\) decreasing uniform comparisons with initial threshold \(x\)
is \(x^n/n!\). Returning true on even parity therefore gives
\(\exp(-x)\). Starting with threshold \(x/2\) and requiring an
additional independent comparison below \(x\) on each continuation
gives \( (x^2/2)^n/n! \), hence \(\exp(-x^2/2)\).
The accepted fractional part is a PSRN and is refined without
resampling. Its endpoints, and hence the normal endpoints, are
dyadic. Matrix-vector products and quadratic forms preserve this
property without intermediate rounding.

For symmetric \(G\), the bounds
\(\sum_i (G_{ii}-r_i)z_i^2\le z^TGz\le
\sum_i(G_{ii}+r_i)z_i^2\), \(r_i=\sum_{j\ne i}|G_{ij}|\),
follow from \(2|z_i z_j|\le z_i^2+z_j^2\).
The same holds for \(H\). When screening is inconclusive, dense interval evaluation of the original exact forms
\(v^TPv\) and \(v^Tv\), with \(v=Bz\), followed by refinement converges to the actual comparison.
The comparison with an independent continuous uniform has equality
with probability zero. Logarithm bounds use correctly rounded rational-to-binary-float
conversion and natural logarithms in Dashu 0.6 or later, directed
downward and upward respectively. Monotonicity of the logarithm and
increasing precision give convergent rational enclosures. Correct output rounding encloses the shared norm using the exact
integer square root of a scaled rational floor. Division of each
coordinate interval by this positive norm interval encloses the
normalized coordinate. The dyadic bounds converge, and equal
nearest-float rounding (including the sign of zero) of both endpoints
certifies the result. After acceptance, refine the same normal samples
without repeating the acceptance test. Midpoint boundaries have probability
zero for dimensions greater than one. The one-dimensional sphere is
sampled directly by an unbiased sign.

The sampler's distributional contract does not include running time,
resource exhaustion, or any observable rejection/refinement trace.
The surrounding PCA transformations and privacy maps still require
the idealized-numerics feature. No claim of vetting is made.
