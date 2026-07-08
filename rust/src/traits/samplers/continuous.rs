use std::{cmp::Ordering, marker::PhantomData};

use dashu::{
    integer::{IBig, UBig},
    rational::RBig,
    rbig,
};

use crate::{
    error::Fallible,
    traits::{
        Float,
        samplers::{
            Shuffle, sample_bernoulli_exp, sample_geometric_exp_fast,
            sample_rounded_gaussian_rational, sample_standard_bernoulli, sample_uniform_ubig_below,
        },
    },
};

pub trait RoundedContinuousVectorSampler<T: Float> {
    fn sample_around(&self, input: &[T]) -> Fallible<Vec<T>>;
}

trait LazyRational {
    fn lower(&self) -> RBig;
    fn upper(&self) -> RBig;
    fn refine(&mut self) -> Fallible<()>;

    fn refine_until_disjoint_from<L: LazyRational>(&mut self, other: &mut L) -> Fallible<Ordering> {
        loop {
            let a_lo = self.lower();
            let a_hi = self.upper();
            let b_lo = other.lower();
            let b_hi = other.upper();

            if a_hi < b_lo {
                return Ok(Ordering::Less);
            }
            if b_hi < a_lo {
                return Ok(Ordering::Greater);
            }

            self.refine()?;
            other.refine()?;
        }
    }

    fn greater_than<L: LazyRational>(&mut self, other: &mut L) -> Fallible<bool> {
        Ok(self.refine_until_disjoint_from(other)? == Ordering::Greater)
    }
}

#[derive(Clone, Debug)]
struct LazyDyadicUniform01 {
    prefix: UBig,
    precision: usize,
}

impl LazyDyadicUniform01 {
    fn sample() -> Fallible<Self> {
        Ok(Self {
            prefix: UBig::ZERO,
            precision: 0,
        })
    }

    fn den(&self) -> UBig {
        UBig::ONE << self.precision
    }
}

impl LazyRational for LazyDyadicUniform01 {
    fn lower(&self) -> RBig {
        if self.precision == 0 {
            RBig::ZERO
        } else {
            RBig::from_parts(self.prefix.as_ibig().clone(), self.den())
        }
    }

    fn upper(&self) -> RBig {
        if self.precision == 0 {
            RBig::ONE
        } else {
            RBig::from_parts(
                (self.prefix.clone() + UBig::ONE).as_ibig().clone(),
                self.den(),
            )
        }
    }

    fn refine(&mut self) -> Fallible<()> {
        let bit = if sample_standard_bernoulli()? {
            UBig::ONE
        } else {
            UBig::ZERO
        };
        self.prefix = (&self.prefix << 1usize) + bit;
        self.precision += 1;
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct LazyAffine<L> {
    scale: RBig,
    offset: RBig,
    inner: L,
}

impl<L> LazyAffine<L> {
    fn new(scale: RBig, offset: RBig, inner: L) -> Self {
        Self {
            scale,
            offset,
            inner,
        }
    }
}

impl<L: LazyRational> LazyRational for LazyAffine<L> {
    fn lower(&self) -> RBig {
        if self.scale >= RBig::ZERO {
            &self.offset + &self.scale * self.inner.lower()
        } else {
            &self.offset + &self.scale * self.inner.upper()
        }
    }

    fn upper(&self) -> RBig {
        if self.scale >= RBig::ZERO {
            &self.offset + &self.scale * self.inner.upper()
        } else {
            &self.offset + &self.scale * self.inner.lower()
        }
    }

    fn refine(&mut self) -> Fallible<()> {
        self.inner.refine()
    }
}

#[derive(Clone, Debug)]
struct LazyDiff<A, B> {
    left: A,
    right: B,
}

impl<A: LazyRational, B: LazyRational> LazyRational for LazyDiff<A, B> {
    fn lower(&self) -> RBig {
        self.left.lower() - self.right.upper()
    }

    fn upper(&self) -> RBig {
        self.left.upper() - self.right.lower()
    }

    fn refine(&mut self) -> Fallible<()> {
        self.left.refine()?;
        self.right.refine()
    }
}

fn midpoint(lhs: RBig, rhs: RBig) -> RBig {
    (lhs + rhs) / RBig::from(2)
}

fn half_gap(lhs: RBig, rhs: RBig) -> RBig {
    (rhs - lhs) / RBig::from(2)
}

type RoundingCell = (Option<RBig>, Option<RBig>);

fn rounding_cell<T: Float>(y: T) -> Fallible<RoundingCell> {
    if y == T::infinity() {
        let max = T::max_value();
        let max_r = max.into_rational()?;
        let prev_r = max.next_down_().into_rational()?;
        return Ok((Some(max_r.clone() + half_gap(prev_r, max_r)), None));
    }

    if y == T::neg_infinity() {
        let min = T::min_value();
        let min_r = min.into_rational()?;
        let next_r = min.next_up_().into_rational()?;
        return Ok((None, Some(min_r.clone() - half_gap(min_r, next_r))));
    }

    if y == T::zero() {
        let lower = midpoint(y.next_down_().into_rational()?, RBig::ZERO);
        let upper = midpoint(RBig::ZERO, y.next_up_().into_rational()?);
        return Ok((Some(lower), Some(upper)));
    }

    let y_r = y.into_rational()?;
    let lower = midpoint(y.next_down_().into_rational()?, y_r.clone());
    let upper = midpoint(y_r.clone(), y.next_up_().into_rational()?);
    Ok((Some(lower), Some(upper)))
}

fn interval_inside_cell(interval: &(RBig, RBig), cell: &RoundingCell) -> bool {
    let lower_in = cell.0.as_ref().is_none_or(|lo| &interval.0 >= lo);
    let upper_in = cell.1.as_ref().is_none_or(|hi| &interval.1 <= hi);
    lower_in && upper_in
}

fn finalize_lazy_rational_to_float<T: Float, L: LazyRational>(mut x: L) -> Fallible<T> {
    loop {
        let interval = (x.lower(), x.upper());
        let mid = midpoint(interval.0.clone(), interval.1.clone());
        let candidate = T::from_rational(mid);
        let cell = rounding_cell(candidate)?;
        if interval_inside_cell(&interval, &cell) {
            return Ok(candidate);
        }
        x.refine()?;
    }
}

fn finalize_input_plus_scaled_lazy_vec<T, L>(
    input: &[T],
    scale: RBig,
    coords: Vec<L>,
) -> Fallible<Vec<T>>
where
    T: Float,
    L: LazyRational,
{
    input
        .iter()
        .copied()
        .zip(coords)
        .map(|(x, y)| {
            if !x.is_finite() {
                return fallible!(FailedFunction, "input must be finite");
            }
            finalize_lazy_rational_to_float::<T, _>(LazyAffine::new(
                scale.clone(),
                x.into_rational()?,
                y,
            ))
        })
        .collect()
}

#[derive(Clone, Debug)]
struct LazyStdLaplace {
    negative: bool,
    e: LazyStdExponential,
}

impl LazyRational for LazyStdLaplace {
    fn lower(&self) -> RBig {
        if self.negative {
            -self.e.upper()
        } else {
            self.e.lower()
        }
    }

    fn upper(&self) -> RBig {
        if self.negative {
            -self.e.lower()
        } else {
            self.e.upper()
        }
    }

    fn refine(&mut self) -> Fallible<()> {
        self.e.refine()
    }
}

#[derive(Clone, Debug)]
struct LazyStdExponential {
    k: UBig,
    x: LazyDyadicUniform01,
}

impl LazyRational for LazyStdExponential {
    fn lower(&self) -> RBig {
        RBig::from(self.k.clone()) + self.x.lower()
    }

    fn upper(&self) -> RBig {
        RBig::from(self.k.clone()) + self.x.upper()
    }

    fn refine(&mut self) -> Fallible<()> {
        self.x.refine()
    }
}

fn sample_lazy_standard_laplace() -> Fallible<LazyStdLaplace> {
    Ok(LazyStdLaplace {
        negative: sample_standard_bernoulli()?,
        e: sample_lazy_standard_exponential()?,
    })
}

fn sample_lazy_standard_exponential() -> Fallible<LazyStdExponential> {
    loop {
        let k = sample_geometric_exp_fast(rbig!(1))?;
        let mut x = LazyDyadicUniform01::sample()?;
        if sample_bernoulli_exp_unit_lazy(&mut x)? {
            return Ok(LazyStdExponential { k, x });
        }
    }
}

fn sample_bernoulli_exp_unit_lazy(x: &mut LazyDyadicUniform01) -> Fallible<bool> {
    let mut y = LazyDyadicUniform01::sample()?;

    if !x.greater_than(&mut y)? {
        return Ok(true);
    }

    let mut accept_on_failure = false;
    loop {
        let mut u = LazyDyadicUniform01::sample()?;

        if !y.greater_than(&mut u)? {
            return Ok(accept_on_failure);
        }

        y = u;
        accept_on_failure = !accept_on_failure;
    }
}

#[derive(Clone, Debug)]
pub struct IidContinuousLaplace<T: Float> {
    scale: RBig,
    _marker: PhantomData<T>,
}

impl<T: Float> IidContinuousLaplace<T> {
    pub fn new(scale: RBig) -> Fallible<Self> {
        if scale < RBig::ZERO {
            return fallible!(FailedFunction, "scale must be nonnegative");
        }
        Ok(Self {
            scale,
            _marker: PhantomData,
        })
    }
}

impl<T: Float> RoundedContinuousVectorSampler<T> for IidContinuousLaplace<T> {
    fn sample_around(&self, input: &[T]) -> Fallible<Vec<T>> {
        input
            .iter()
            .copied()
            .map(|x| {
                if !x.is_finite() {
                    return fallible!(FailedFunction, "input must be finite");
                }
                finalize_lazy_rational_to_float::<T, _>(LazyAffine::new(
                    self.scale.clone(),
                    x.into_rational()?,
                    sample_lazy_standard_laplace()?,
                ))
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct IidContinuousGaussian<T: Float> {
    scale: RBig,
    _marker: PhantomData<T>,
}

impl<T: Float> IidContinuousGaussian<T> {
    pub fn new(scale: RBig) -> Fallible<Self> {
        if scale < RBig::ZERO {
            return fallible!(FailedFunction, "scale must be nonnegative");
        }
        Ok(Self {
            scale,
            _marker: PhantomData,
        })
    }
}

impl<T: Float> RoundedContinuousVectorSampler<T> for IidContinuousGaussian<T> {
    fn sample_around(&self, input: &[T]) -> Fallible<Vec<T>> {
        input
            .iter()
            .copied()
            .map(|x| sample_rounded_gaussian_rational(x, self.scale.clone()))
            .collect()
    }
}

#[derive(Clone, Debug)]
struct ScaleIndexTerm {
    coeff_num: UBig,
    coeff_den: UBig,
    exp_b: usize,
    exp_omb: usize,
    q: usize,
}

#[derive(Clone, Debug)]
pub struct DiscreteKnormStaircaseScaleIndex {
    d: usize,
    delta: RBig,
    r: RBig,
    epsilon: RBig,
    terms: Vec<ScaleIndexTerm>,
}

impl DiscreteKnormStaircaseScaleIndex {
    pub fn new(d: usize, delta: RBig, r: RBig, epsilon: RBig) -> Fallible<Self> {
        if d == 0 {
            return fallible!(FailedFunction, "dimension must be positive");
        }
        if delta <= RBig::ZERO {
            return fallible!(FailedFunction, "delta must be positive");
        }
        if r < RBig::ZERO || r > delta {
            return fallible!(FailedFunction, "r must lie in [0, delta]");
        }
        if epsilon <= RBig::ZERO {
            return fallible!(FailedFunction, "epsilon must be positive");
        }

        let terms = build_scale_index_terms(d, &delta, &r)?;
        if terms.is_empty() {
            return fallible!(FailedFunction, "scale-index mixture has no positive terms");
        }
        Ok(Self {
            d,
            delta,
            r,
            epsilon,
            terms,
        })
    }

    pub fn dimension(&self) -> usize {
        self.d
    }

    pub fn sample(&self) -> Fallible<UBig> {
        loop {
            let idx = sample_categorical_rational_weights(
                self.terms
                    .iter()
                    .map(|term| (&term.coeff_num, &term.coeff_den)),
            )?;
            let term = &self.terms[idx];
            if sample_exp_monomial(self.epsilon.clone(), term.exp_b, term.exp_omb)? {
                let y = sample_negative_binomial_integer(term.q + 1, self.epsilon.clone())?;
                return Ok(y + UBig::from(term.q));
            }
        }
    }

    pub fn scale_for(&self, i: &UBig) -> RBig {
        &self.delta * RBig::from(i.clone()) + &self.r
    }
}

fn build_scale_index_terms(d: usize, delta: &RBig, r: &RBig) -> Fallible<Vec<ScaleIndexTerm>> {
    let mut coeffs = vec![RBig::ZERO; d + 1];

    for m in 0..=d {
        let power_coeff = RBig::from(binom_usize(d, m)) * rbig_pow(delta, m) * rbig_pow(r, d - m);

        for q in 0..=m {
            let s = stirling2(m, q);
            if s.is_zero() {
                continue;
            }
            coeffs[q] += power_coeff.clone() * RBig::from(s) * RBig::from(factorial_ubig(q));
        }
    }

    let mut terms = Vec::new();
    for (q, coeff) in coeffs.into_iter().enumerate() {
        if coeff.is_zero() {
            continue;
        }
        let (num, den) = rbig_to_positive_fraction(coeff)?;
        terms.push(ScaleIndexTerm {
            coeff_num: num,
            coeff_den: den,
            exp_b: q,
            exp_omb: d - q,
            q,
        });
    }
    Ok(terms)
}

#[derive(Clone, Debug)]
struct LazyL1BallCoord {
    sign: i8,
    spacing: LazyDiff<LazyPoint01, LazyPoint01>,
}

impl LazyRational for LazyL1BallCoord {
    fn lower(&self) -> RBig {
        if self.sign > 0 {
            self.spacing.lower()
        } else {
            -self.spacing.upper()
        }
    }

    fn upper(&self) -> RBig {
        if self.sign > 0 {
            self.spacing.upper()
        } else {
            -self.spacing.lower()
        }
    }

    fn refine(&mut self) -> Fallible<()> {
        self.spacing.refine()
    }
}

#[derive(Clone, Debug)]
enum LazyPoint01 {
    Const(RBig),
    Uniform(LazyDyadicUniform01),
}

impl LazyRational for LazyPoint01 {
    fn lower(&self) -> RBig {
        match self {
            Self::Const(x) => x.clone(),
            Self::Uniform(u) => u.lower(),
        }
    }

    fn upper(&self) -> RBig {
        match self {
            Self::Const(x) => x.clone(),
            Self::Uniform(u) => u.upper(),
        }
    }

    fn refine(&mut self) -> Fallible<()> {
        match self {
            Self::Const(_) => Ok(()),
            Self::Uniform(u) => u.refine(),
        }
    }
}

fn sample_l1_unit_ball_lazy(d: usize) -> Fallible<Vec<LazyL1BallCoord>> {
    let mut uniforms = (0..d)
        .map(|_| LazyDyadicUniform01::sample())
        .collect::<Fallible<Vec<_>>>()?;
    sort_lazy_uniforms(&mut uniforms)?;

    let mut points = Vec::with_capacity(d + 2);
    points.push(LazyPoint01::Const(RBig::ZERO));
    points.extend(uniforms.into_iter().map(LazyPoint01::Uniform));
    points.push(LazyPoint01::Const(RBig::ONE));

    (0..d)
        .map(|j| {
            Ok(LazyL1BallCoord {
                sign: if sample_standard_bernoulli()? { 1 } else { -1 },
                spacing: LazyDiff {
                    left: points[j + 1].clone(),
                    right: points[j].clone(),
                },
            })
        })
        .collect()
}

fn sort_lazy_uniforms(values: &mut [LazyDyadicUniform01]) -> Fallible<()> {
    for i in 1..values.len() {
        let mut j = i;
        while j > 0 {
            let mut a = values[j - 1].clone();
            let mut b = values[j].clone();
            let ord = a.refine_until_disjoint_from(&mut b)?;
            values[j - 1] = a;
            values[j] = b;
            if ord == Ordering::Greater {
                values.swap(j - 1, j);
                j -= 1;
            } else {
                break;
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct LazyLinfBallCoord {
    uniform: LazyDyadicUniform01,
}

impl LazyRational for LazyLinfBallCoord {
    fn lower(&self) -> RBig {
        self.uniform.lower() * RBig::from(2) - RBig::ONE
    }

    fn upper(&self) -> RBig {
        self.uniform.upper() * RBig::from(2) - RBig::ONE
    }

    fn refine(&mut self) -> Fallible<()> {
        self.uniform.refine()
    }
}

fn sample_linf_unit_ball_lazy(d: usize) -> Fallible<Vec<LazyLinfBallCoord>> {
    (0..d)
        .map(|_| {
            Ok(LazyLinfBallCoord {
                uniform: LazyDyadicUniform01::sample()?,
            })
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct ContinuousVectorL1Staircase<T: Float> {
    scale_index: DiscreteKnormStaircaseScaleIndex,
    _marker: PhantomData<T>,
}

impl<T: Float> ContinuousVectorL1Staircase<T> {
    pub fn new(d: usize, delta: RBig, r: RBig, epsilon: RBig) -> Fallible<Self> {
        Ok(Self {
            scale_index: DiscreteKnormStaircaseScaleIndex::new(d, delta, r, epsilon)?,
            _marker: PhantomData,
        })
    }
}

impl<T: Float> RoundedContinuousVectorSampler<T> for ContinuousVectorL1Staircase<T> {
    fn sample_around(&self, input: &[T]) -> Fallible<Vec<T>> {
        if input.len() != self.scale_index.dimension() {
            return fallible!(
                FailedFunction,
                "input length does not match sampler dimension"
            );
        }
        let i = self.scale_index.sample()?;
        finalize_input_plus_scaled_lazy_vec(
            input,
            self.scale_index.scale_for(&i),
            sample_l1_unit_ball_lazy(input.len())?,
        )
    }
}

#[derive(Clone, Debug)]
pub struct ContinuousVectorLinfStaircase<T: Float> {
    scale_index: DiscreteKnormStaircaseScaleIndex,
    _marker: PhantomData<T>,
}

impl<T: Float> ContinuousVectorLinfStaircase<T> {
    pub fn new(d: usize, delta: RBig, r: RBig, epsilon: RBig) -> Fallible<Self> {
        Ok(Self {
            scale_index: DiscreteKnormStaircaseScaleIndex::new(d, delta, r, epsilon)?,
            _marker: PhantomData,
        })
    }
}

impl<T: Float> RoundedContinuousVectorSampler<T> for ContinuousVectorLinfStaircase<T> {
    fn sample_around(&self, input: &[T]) -> Fallible<Vec<T>> {
        if input.len() != self.scale_index.dimension() {
            return fallible!(
                FailedFunction,
                "input length does not match sampler dimension"
            );
        }
        let i = self.scale_index.sample()?;
        finalize_input_plus_scaled_lazy_vec(
            input,
            self.scale_index.scale_for(&i),
            sample_linf_unit_ball_lazy(input.len())?,
        )
    }
}

#[derive(Clone, Debug)]
pub struct JointDiscreteL1Staircase {
    d: usize,
    delta: usize,
    r: usize,
    epsilon: RBig,
    terms: Vec<RadiusTerm>,
    categorical_coeffs: Vec<UBig>,
}

impl JointDiscreteL1Staircase {
    pub fn new(d: usize, delta: usize, r: usize, epsilon: RBig) -> Fallible<Self> {
        if delta == 0 {
            return fallible!(FailedFunction, "delta must be positive");
        }
        if r == 0 || r > delta {
            return fallible!(FailedFunction, "r must be in 1..=delta");
        }
        if epsilon <= RBig::ZERO {
            return fallible!(FailedFunction, "epsilon must be positive");
        }

        let terms = build_l1_radius_terms(d, delta, r)?;
        let categorical_coeffs = terms.iter().map(|term| term.coeff.clone()).collect();
        Ok(Self {
            d,
            delta,
            r,
            epsilon,
            terms,
            categorical_coeffs,
        })
    }

    pub fn sample_noise(&self) -> Fallible<Vec<IBig>> {
        sample_l1_sphere_integer(self.d, self.sample_radius()?)
    }

    pub fn dimension(&self) -> usize {
        self.d
    }

    pub fn delta(&self) -> usize {
        self.delta
    }

    pub fn r(&self) -> usize {
        self.r
    }

    pub fn sample_radius(&self) -> Fallible<UBig> {
        if self.d == 0 {
            return Ok(UBig::ZERO);
        }
        loop {
            let idx = sample_categorical_integer_weights(&self.categorical_coeffs)?;
            let term = &self.terms[idx];
            if sample_exp_monomial(self.epsilon.clone(), term.exp_b, term.exp_omb)? {
                return match term.choice.clone() {
                    RadiusChoice::Zero => Ok(UBig::ZERO),
                    RadiusChoice::Tail {
                        residue,
                        q,
                        k_shift,
                    } => {
                        let y = sample_negative_binomial_integer(q + 1, self.epsilon.clone())?;
                        let k = y + UBig::from(q) + UBig::from(k_shift);
                        Ok(k * UBig::from(self.delta) + UBig::from(residue))
                    }
                };
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct JointDiscreteLinfStaircase {
    d: usize,
    delta: usize,
    r: usize,
    epsilon: RBig,
    terms: Vec<LinfRadiusTerm>,
    categorical_coeffs: Vec<UBig>,
}

impl JointDiscreteLinfStaircase {
    pub fn new(d: usize, delta: usize, r: usize, epsilon: RBig) -> Fallible<Self> {
        if delta == 0 {
            return fallible!(FailedFunction, "delta must be positive");
        }
        if r == 0 || r > delta {
            return fallible!(FailedFunction, "r must be in 1..=delta");
        }
        if epsilon <= RBig::ZERO {
            return fallible!(FailedFunction, "epsilon must be positive");
        }

        let terms = build_linf_radius_terms(d, delta, r)?;
        let categorical_coeffs = terms.iter().map(|term| term.coeff.clone()).collect();
        Ok(Self {
            d,
            delta,
            r,
            epsilon,
            terms,
            categorical_coeffs,
        })
    }

    pub fn sample_noise(&self) -> Fallible<Vec<IBig>> {
        sample_linf_shell_integer(self.d, self.sample_radius()?)
    }

    pub fn dimension(&self) -> usize {
        self.d
    }

    pub fn delta(&self) -> usize {
        self.delta
    }

    pub fn r(&self) -> usize {
        self.r
    }

    pub fn sample_radius(&self) -> Fallible<UBig> {
        if self.d == 0 {
            return Ok(UBig::ZERO);
        }
        loop {
            let idx = sample_categorical_integer_weights(&self.categorical_coeffs)?;
            let term = &self.terms[idx];
            if sample_exp_monomial(self.epsilon.clone(), term.exp_b, term.exp_omb)? {
                return match term.choice.clone() {
                    LinfRadiusChoice::Zero => Ok(UBig::ZERO),
                    LinfRadiusChoice::Tail {
                        residue,
                        q,
                        k_shift,
                    } => {
                        let y = sample_negative_binomial_integer(q + 1, self.epsilon.clone())?;
                        let k = y + UBig::from(q) + UBig::from(k_shift);
                        Ok(k * UBig::from(self.delta) + UBig::from(residue))
                    }
                };
            }
        }
    }
}

#[derive(Clone, Debug)]
enum RadiusChoice {
    Zero,
    Tail {
        residue: usize,
        q: usize,
        k_shift: usize,
    },
}

#[derive(Clone, Debug)]
struct RadiusTerm {
    coeff: UBig,
    exp_b: usize,
    exp_omb: usize,
    choice: RadiusChoice,
}

fn build_l1_radius_terms(d: usize, delta: usize, r: usize) -> Fallible<Vec<RadiusTerm>> {
    let mut terms = vec![RadiusTerm {
        coeff: UBig::ONE,
        exp_b: 0,
        exp_omb: d,
        choice: RadiusChoice::Zero,
    }];

    if d == 0 {
        return Ok(terms);
    }

    let coeffs_j0 = l1_shell_coeffs_binom_k(d, delta, delta - 1)?;
    for (q, coeff) in coeffs_j0.into_iter().enumerate() {
        if coeff.is_zero() {
            continue;
        }
        terms.push(RadiusTerm {
            coeff,
            exp_b: q + 1,
            exp_omb: d - q - 1,
            choice: RadiusChoice::Tail {
                residue: 0,
                q,
                k_shift: 1,
            },
        });
    }

    for j in 1..delta {
        let coeffs = l1_shell_coeffs_binom_k(d, delta, j - 1)?;
        let staircase_bump = usize::from(j >= r);
        for (q, coeff) in coeffs.into_iter().enumerate() {
            if coeff.is_zero() {
                continue;
            }
            terms.push(RadiusTerm {
                coeff,
                exp_b: q + staircase_bump,
                exp_omb: d - q - 1,
                choice: RadiusChoice::Tail {
                    residue: j,
                    q,
                    k_shift: 0,
                },
            });
        }
    }

    Ok(terms)
}

fn l1_shell_coeffs_binom_k(d: usize, delta: usize, offset: usize) -> Fallible<Vec<UBig>> {
    let mut coeffs = vec![UBig::ZERO; d];

    for s in 1..=d {
        let t = s - 1;
        let h = block_subset_coeffs(delta, offset, t)?;
        let shell_multiplier = (UBig::ONE << s) * binom_usize(d, s);

        for q in 0..=t {
            coeffs[q] += &shell_multiplier * &h[q];
        }
    }

    Ok(coeffs)
}

fn block_subset_coeffs(delta: usize, offset: usize, t: usize) -> Fallible<Vec<UBig>> {
    let mut h = vec![UBig::ZERO; t + 1];

    let mut block_poly = vec![UBig::ZERO; t + 1];
    for a in 1..=t.min(delta) {
        block_poly[a] = binom_usize(delta, a);
    }

    let mut extra_poly = vec![UBig::ZERO; t + 1];
    for a in 0..=t.min(offset) {
        extra_poly[a] = binom_usize(offset, a);
    }

    let mut pow = vec![UBig::ZERO; t + 1];
    pow[0] = UBig::ONE;
    for q in 0..=t {
        let prod = poly_mul_trunc(&pow, &extra_poly, t);
        h[q] = prod[t].clone();

        if q < t {
            pow = poly_mul_trunc(&pow, &block_poly, t);
        }
    }

    Ok(h)
}

#[derive(Clone, Debug)]
enum LinfRadiusChoice {
    Zero,
    Tail {
        residue: usize,
        q: usize,
        k_shift: usize,
    },
}

#[derive(Clone, Debug)]
struct LinfRadiusTerm {
    coeff: UBig,
    exp_b: usize,
    exp_omb: usize,
    choice: LinfRadiusChoice,
}

fn build_linf_radius_terms(d: usize, delta: usize, r: usize) -> Fallible<Vec<LinfRadiusTerm>> {
    let mut terms = vec![LinfRadiusTerm {
        coeff: UBig::ONE,
        exp_b: 0,
        exp_omb: d,
        choice: LinfRadiusChoice::Zero,
    }];

    if d == 0 {
        return Ok(terms);
    }

    let coeffs_j0 = linf_shell_coeffs_binom_k(d, 2 * delta, 2 * delta - 1)?;
    for (q, coeff) in coeffs_j0.into_iter().enumerate() {
        if coeff.is_zero() {
            continue;
        }
        terms.push(LinfRadiusTerm {
            coeff,
            exp_b: q + 1,
            exp_omb: d - q - 1,
            choice: LinfRadiusChoice::Tail {
                residue: 0,
                q,
                k_shift: 1,
            },
        });
    }

    for j in 1..delta {
        let coeffs = linf_shell_coeffs_binom_k(d, 2 * delta, 2 * j - 1)?;
        let staircase_bump = usize::from(j >= r);
        for (q, coeff) in coeffs.into_iter().enumerate() {
            if coeff.is_zero() {
                continue;
            }
            terms.push(LinfRadiusTerm {
                coeff,
                exp_b: q + staircase_bump,
                exp_omb: d - q - 1,
                choice: LinfRadiusChoice::Tail {
                    residue: j,
                    q,
                    k_shift: 0,
                },
            });
        }
    }

    Ok(terms)
}

fn linf_shell_coeffs_binom_k(d: usize, block: usize, offset: usize) -> Fallible<Vec<UBig>> {
    let mut coeffs = vec![UBig::ZERO; d];

    for s in 1..=d {
        let m = d - s;
        let h = affine_power_coeffs_binom_k(block, offset, m);
        let multiplier = binom_usize(d, s) * (UBig::ONE << s);
        for q in 0..=m {
            coeffs[q] += &multiplier * &h[q];
        }
    }

    Ok(coeffs)
}

fn affine_power_coeffs_binom_k(block: usize, offset: usize, m: usize) -> Vec<UBig> {
    let mut out = vec![UBig::ZERO; m + 1];

    for t in 0..=m {
        let coeff_k_power =
            binom_usize(m, t) * ubig_pow_usize(block, t) * ubig_pow_usize(offset, m - t);

        for q in 0..=t {
            let s = stirling2(t, q);
            if s.is_zero() {
                continue;
            }
            out[q] += &coeff_k_power * s * factorial_ubig(q);
        }
    }

    out
}

fn sample_l1_sphere_integer(d: usize, radius: UBig) -> Fallible<Vec<IBig>> {
    if radius.is_zero() {
        return Ok(vec![IBig::ZERO; d]);
    }

    let s = sample_l1_nonzero_count(d, &radius)?;
    let indices = sample_subset_usize(d, s)?;
    let parts = sample_positive_composition(radius, s)?;

    let mut out = vec![IBig::ZERO; d];
    for (idx, part) in indices.into_iter().zip(parts) {
        let magnitude = part.as_ibig().clone();
        out[idx] = if sample_standard_bernoulli()? {
            magnitude
        } else {
            -magnitude
        };
    }

    Ok(out)
}

fn sample_l1_nonzero_count(d: usize, radius: &UBig) -> Fallible<usize> {
    let radius_minus_one = radius - UBig::ONE;

    let mut weights = Vec::new();
    let mut values = Vec::new();
    for s in 1..=d {
        if UBig::from(s) > *radius {
            break;
        }
        weights.push(
            (UBig::ONE << s) * binom_usize(d, s) * binom_ubig_usize(&radius_minus_one, s - 1),
        );
        values.push(s);
    }

    let idx = sample_categorical_integer_weights(&weights)?;
    Ok(values[idx])
}

fn sample_linf_shell_integer(d: usize, radius: UBig) -> Fallible<Vec<IBig>> {
    if radius.is_zero() {
        return Ok(vec![IBig::ZERO; d]);
    }

    let boundary_count = sample_linf_boundary_count(d, &radius)?;
    let boundary_indices = sample_subset_usize(d, boundary_count)?;
    let mut is_boundary = vec![false; d];
    for idx in boundary_indices {
        is_boundary[idx] = true;
    }

    let interior_radius = &radius - UBig::ONE;
    (0..d)
        .map(|j| {
            if is_boundary[j] {
                let mag = radius.as_ibig().clone();
                Ok(if sample_standard_bernoulli()? {
                    mag
                } else {
                    -mag
                })
            } else {
                sample_uniform_ibig_symmetric(&interior_radius)
            }
        })
        .collect()
}

fn sample_linf_boundary_count(d: usize, radius: &UBig) -> Fallible<usize> {
    let width = radius * UBig::from(2usize) - UBig::ONE;
    let mut weights = Vec::with_capacity(d);
    let mut values = Vec::with_capacity(d);

    for s in 1..=d {
        weights.push(binom_usize(d, s) * (UBig::ONE << s) * ubig_pow(&width, d - s));
        values.push(s);
    }

    let idx = sample_categorical_integer_weights(&weights)?;
    Ok(values[idx])
}

fn sample_uniform_ibig_symmetric(m: &UBig) -> Fallible<IBig> {
    let width = m * UBig::from(2usize) + UBig::ONE;
    let draw = sample_uniform_ubig_below(width)?;
    Ok(draw.as_ibig().clone() - m.as_ibig().clone())
}

fn sample_subset_usize(n: usize, k: usize) -> Fallible<Vec<usize>> {
    let mut indices = (0..n).collect::<Vec<_>>();
    indices.shuffle()?;
    indices.truncate(k);
    Ok(indices)
}

fn sample_positive_composition(radius: UBig, s: usize) -> Fallible<Vec<UBig>> {
    if s == 0 {
        return fallible!(FailedFunction, "number of parts must be positive");
    }
    if s == 1 {
        return Ok(vec![radius]);
    }

    let n = &radius - UBig::ONE;
    let k = s - 1;
    let total = binom_ubig_usize(&n, k);
    let rank = sample_uniform_ubig_below(total)?;
    let mut cuts = unrank_combination_colex(n, k, rank)?
        .into_iter()
        .map(|c| c + UBig::ONE)
        .collect::<Vec<_>>();

    cuts.sort();

    let mut parts = Vec::with_capacity(s);
    let mut prev = UBig::ZERO;
    for cut in cuts {
        parts.push(&cut - &prev);
        prev = cut;
    }
    parts.push(radius - prev);
    Ok(parts)
}

fn unrank_combination_colex(mut n: UBig, k: usize, mut rank: UBig) -> Fallible<Vec<UBig>> {
    let mut result = Vec::with_capacity(k);

    for remaining in (1..=k).rev() {
        loop {
            if n.is_zero() {
                return fallible!(FailedFunction, "combination rank exhausted");
            }
            n -= UBig::ONE;
            let count = binom_ubig_usize(&n, remaining - 1);
            if rank < count {
                result.push(n.clone());
                break;
            }
            rank -= count;
        }
    }

    Ok(result)
}

fn sample_negative_binomial_integer(shape: usize, epsilon: RBig) -> Fallible<UBig> {
    let mut out = UBig::ZERO;
    for _ in 0..shape {
        out += sample_geometric_exp_fast(epsilon.clone())?;
    }
    Ok(out)
}

fn sample_exp_monomial(epsilon: RBig, exp_b: usize, exp_omb: usize) -> Fallible<bool> {
    for _ in 0..exp_b {
        if !sample_bernoulli_exp(epsilon.clone())? {
            return Ok(false);
        }
    }
    for _ in 0..exp_omb {
        if sample_bernoulli_exp(epsilon.clone())? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn sample_categorical_integer_weights(weights: &[UBig]) -> Fallible<usize> {
    let total = weights.iter().fold(UBig::ZERO, |acc, w| acc + w);
    if total.is_zero() {
        return fallible!(FailedFunction, "all categorical weights are zero");
    }

    let draw = sample_uniform_ubig_below(total)?;
    let mut cumulative = UBig::ZERO;
    for (idx, weight) in weights.iter().enumerate() {
        cumulative += weight;
        if draw < cumulative {
            return Ok(idx);
        }
    }
    unreachable!("draw is strictly less than total weight")
}

fn sample_categorical_rational_weights<'a, I>(weights: I) -> Fallible<usize>
where
    I: IntoIterator<Item = (&'a UBig, &'a UBig)>,
{
    let pairs = weights.into_iter().collect::<Vec<_>>();
    if pairs.is_empty() {
        return fallible!(FailedFunction, "empty categorical weights");
    }

    let common_den = pairs.iter().fold(UBig::ONE, |acc, (_, den)| acc * *den);
    let integer_weights = pairs
        .into_iter()
        .map(|(num, den)| num * (&common_den / den))
        .collect::<Vec<_>>();
    sample_categorical_integer_weights(&integer_weights)
}

fn rbig_to_positive_fraction(x: RBig) -> Fallible<(UBig, UBig)> {
    if x < RBig::ZERO {
        return fallible!(FailedFunction, "expected nonnegative rational");
    }
    let (num, den) = x.into_parts();
    let (_, num_abs) = num.into_parts();
    Ok((num_abs, den))
}

fn rbig_pow(x: &RBig, p: usize) -> RBig {
    (0..p).fold(RBig::ONE, |acc, _| acc * x)
}

fn ubig_pow(base: &UBig, exp: usize) -> UBig {
    (0..exp).fold(UBig::ONE, |acc, _| acc * base)
}

fn ubig_pow_usize(base: usize, exp: usize) -> UBig {
    (0..exp).fold(UBig::ONE, |acc, _| acc * UBig::from(base))
}

fn factorial_ubig(n: usize) -> UBig {
    (1..=n).fold(UBig::ONE, |acc, k| acc * UBig::from(k))
}

fn stirling2(n: usize, k: usize) -> UBig {
    if k > n {
        return UBig::ZERO;
    }
    let mut dp = vec![vec![UBig::ZERO; k + 2]; n + 1];
    dp[0][0] = UBig::ONE;
    for i in 1..=n {
        for j in 1..=k.min(i) {
            dp[i][j] = &dp[i - 1][j - 1] + UBig::from(j) * &dp[i - 1][j];
        }
    }
    dp[n][k].clone()
}

fn binom_usize(n: usize, k: usize) -> UBig {
    if k > n {
        return UBig::ZERO;
    }
    let k = k.min(n - k);
    (0..k).fold(UBig::ONE, |acc, i| {
        acc * UBig::from(n - i) / UBig::from(i + 1)
    })
}

fn binom_ubig_usize(n: &UBig, k: usize) -> UBig {
    (0..k).fold(UBig::ONE, |acc, i| {
        acc * (n - UBig::from(i)) / UBig::from(i + 1)
    })
}

fn poly_mul_trunc(left: &[UBig], right: &[UBig], degree: usize) -> Vec<UBig> {
    let mut out = vec![UBig::ZERO; degree + 1];

    for i in 0..=degree {
        if left[i].is_zero() {
            continue;
        }
        for j in 0..=(degree - i) {
            if right[j].is_zero() {
                continue;
            }
            out[i + j] += &left[i] * &right[j];
        }
    }

    out
}
