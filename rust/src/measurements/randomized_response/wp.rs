//! Relational weakest-preexpectation proof of the *production* Boolean RR kernel.
//!
//! This proof uses the actual BoolRR::release method, whose executable body
//! is shared verbatim between OpenDP and Verus (dual_spec). The sole
//! probabilistic assumption is the uniform-word contract of sample_uniform_u64.
//! This is not yet a proof of the constructor's float bound or closure erasure.
use vstd::prelude::*;
use crate::{operator, wp};
use crate::bernoulli_proof::{prefix_threshold, lemma_success_is_prefix};

verus! {

/// Precompose any postexpectation with OpenDP's executable RR operator.
/// In a probabilistic program logic this is the deterministic wp rule.
pub open spec fn post_after_release<E>(
    truth: bool, numerator: u64,
    post: spec_fn(Result<bool, E>) -> real,
) -> spec_fn(Result<u64, E>) -> real {
    |entropy: Result<u64, E>| post(
        (operator::BoolRR { numerator, privacy_cost: 0u64 }).release(truth, entropy)
    )
}

/// wp of Boolean RR derived from the actual executable release method.
/// The parameter `sample_wp` must denote the wp of the *actual* production
/// sample_uniform_u64 call, not an independently postulated output law.
pub open spec fn rr_wp<E>(
    truth: bool, numerator: u64,
    sample_wp: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    post: spec_fn(Result<bool, E>) -> real,
) -> real {
    sample_wp(post_after_release(truth, numerator, post))
}

/// Push the uniform-word sampler's expectation contract through the actual
/// executable operator. The output error is the exact original error value.
pub proof fn lemma_rr_wp_from_uniform<E>(
    truth: bool, numerator: u64,
    sample_wp: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    error_wp: spec_fn(spec_fn(E) -> real) -> real,
    success: real,
    post: spec_fn(Result<bool, E>) -> real,
)
    requires
        numerator <= 9_007_199_254_740_992u64,
        wp::bounded_post(post),
        wp::uniform_word_wp_contract(sample_wp, error_wp, success),
    ensures
        rr_wp(truth, numerator, sample_wp, post)
        == success * (
            (numerator as real / 9_007_199_254_740_992real) * post(Ok(truth))
            + (1real - numerator as real / 9_007_199_254_740_992real) * post(Ok(!truth))
        ) + error_wp(|err: E| post(Err(err))),
{
    let threshold = prefix_threshold(numerator);
    let on_error = |err: E| post(Err(err));
    let before = post_after_release(truth, numerator, post);
    let prefix = wp::word_prefix_post(
        threshold, post(Ok(truth)), post(Ok(!truth)), on_error,
    );
    assert(wp::bounded_post(on_error)) by {
        assert forall|err: E| 0real <= #[trigger] on_error(err) <= 1real by {
            assert(0real <= post(Err(err)) <= 1real);
        }
    };
    assert forall|entropy: Result<u64, E>| before(entropy) == prefix(entropy) by {
        match entropy {
            Ok(word) => {
                lemma_success_is_prefix(numerator, word);
                if (word as nat) < threshold {} else {}
            }
            Err(err) => {}
        }
    }
    assert(before =~= prefix);
    assert(rr_wp(truth, numerator, sample_wp, post) == sample_wp(prefix));

    assert(threshold <= 18_446_744_073_709_551_616nat) by(nonlinear_arith)
        requires numerator <= 9_007_199_254_740_992u64;
    assert(sample_wp(prefix) == success * (
        (threshold as real / wp::word64_cardinality()) * post(Ok(truth))
        + (1real - threshold as real / wp::word64_cardinality()) * post(Ok(!truth))
    ) + error_wp(on_error));
    assert(threshold as real / wp::word64_cardinality()
        == numerator as real / 9_007_199_254_740_992real)
        by(nonlinear_arith);
}

/// Finite-epsilon RR compares expectations, rather than comparing PMF atoms.
/// The same error payoff occurs on both sides for every observable error.
pub proof fn lemma_rr_wp_domination<E>(
    numerator: u64,
    sample_wp: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    error_wp: spec_fn(spec_fn(E) -> real) -> real,
    success: real,
    factor: real,
    left: bool, right: bool,
    post: spec_fn(Result<bool, E>) -> real,
)
    requires
        4_503_599_627_370_496u64 <= numerator < 9_007_199_254_740_992u64,
        factor >= 1real,
        numerator as real / 9_007_199_254_740_992real
            <= factor * (1real - numerator as real / 9_007_199_254_740_992real),
        wp::bounded_post(post),
        wp::uniform_word_wp_contract(sample_wp, error_wp, success),
    ensures
        rr_wp(left, numerator, sample_wp, post)
            <= factor * rr_wp(right, numerator, sample_wp, post),
{
    lemma_rr_wp_from_uniform(left, numerator, sample_wp, error_wp, success, post);
    lemma_rr_wp_from_uniform(right, numerator, sample_wp, error_wp, success, post);
    let p = numerator as real / 9_007_199_254_740_992real;
    let a = post(Ok(left));
    let b = post(Ok(!left));
    let shared_error = error_wp(|err: E| post(Err(err)));
    assert(0real <= a <= 1real);
    assert(0real <= b <= 1real);
    assert(0real <= success);
    assert(shared_error >= 0real) by {
        let f = |err: E| post(Err(err));
        assert(wp::bounded_post(f)) by {
            assert forall|err: E| 0real <= #[trigger] f(err) <= 1real by {
                assert(0real <= post(Err(err)) <= 1real);
            }
        }
    };
    if left == right {
        assert(rr_wp(left, numerator, sample_wp, post)
            == rr_wp(right, numerator, sample_wp, post));
        assert(rr_wp(right, numerator, sample_wp, post) >= 0real)
            by(nonlinear_arith);
    } else {
        assert(1real / 2real <= p <= 1real);
        assert(1real - p <= p);
        assert(1real - p <= factor * p) by(nonlinear_arith);
        assert(p * a + (1real - p) * b
            <= factor * ((1real - p) * a + p * b))
            by(nonlinear_arith)
            requires
                a >= 0real, b >= 0real,
                p <= factor * (1real - p),
                1real - p <= factor * p;
        assert(success * (p * a + (1real - p) * b) + shared_error
            <= factor * (success * ((1real - p) * a + p * b) + shared_error))
            by(nonlinear_arith)
            requires success >= 0real, shared_error >= 0real, factor >= 1real,
                p * a + (1real - p) * b
                    <= factor * ((1real - p) * a + p * b);
    }
}

/// The Boolean discrete metric has no more than one unit of separation.
/// For p=1, the infinite privacy map is correct; otherwise use the
/// verified likelihood-ratio factor for all positive distances.
pub open spec fn rr_factor_map(numerator: u64, factor: real, distance: u32) -> Option<real> {
    if distance == 0 { Some(1real) }
    else if numerator == 9_007_199_254_740_992u64 { None }
    else { Some(factor) }
}

pub proof fn lemma_valid_rr_measurement_wp<E>(
    numerator: u64,
    sample_wp: spec_fn(spec_fn(Result<u64, E>) -> real) -> real,
    error_wp: spec_fn(spec_fn(E) -> real) -> real,
    success: real,
    factor: real,
)
    requires
        4_503_599_627_370_496u64 <= numerator <= 9_007_199_254_740_992u64,
        factor >= 1real,
        numerator < 9_007_199_254_740_992u64 ==>
            numerator as real / 9_007_199_254_740_992real
                <= factor * (1real - numerator as real / 9_007_199_254_740_992real),
        wp::uniform_word_wp_contract(sample_wp, error_wp, success),
    ensures
        wp::valid_wp_max_divergence::<bool, Result<bool, E>, u32>(
            |_: bool| true,
            |left: bool, right: bool, distance: u32| left == right || distance > 0,
            |truth: bool, post: spec_fn(Result<bool, E>) -> real|
                rr_wp(truth, numerator, sample_wp, post),
            |distance: u32| rr_factor_map(numerator, factor, distance),
        ),
{
    assert forall|left: bool, right: bool, distance: u32,
                  post: spec_fn(Result<bool, E>) -> real|
        #[trigger] (left == right || distance > 0)
        && wp::bounded_post(post)
        implies match rr_factor_map(numerator, factor, distance) {
            None => true,
            Some(cost) => rr_wp(left, numerator, sample_wp, post)
                <= cost * rr_wp(right, numerator, sample_wp, post),
        } by {
        if distance == 0 {
            assert(left == right);
            assert(rr_wp(left, numerator, sample_wp, post)
                == rr_wp(right, numerator, sample_wp, post));
            lemma_rr_wp_from_uniform(right, numerator, sample_wp, error_wp, success, post);
            assert(rr_wp(right, numerator, sample_wp, post) >= 0real)
                by(nonlinear_arith);
        } else if numerator == 9_007_199_254_740_992u64 {
        } else {
            lemma_rr_wp_domination(
                numerator, sample_wp, error_wp, success,
                factor, left, right, post,
            );
        }
    }
}

} // verus!
