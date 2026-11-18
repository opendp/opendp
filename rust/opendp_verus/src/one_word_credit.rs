//! Minimal Alerus-style probabilistic error credit for one entropy draw.
//!
//! This is verification-only. The credit is a linear Verus resource rather
//! than an ordinary ghost real: callers cannot mint or copy positive credits.
//! No rejection sampling, recursion, credit splitting, or amplification is
//! needed for the one-word Boolean randomized-response sampler.
//!
//! The random primitive's credit transfer is the one trusted probabilistic
//! rule. Interpreting these judgments as distributional expectations relies
//! on the soundness of the error-credit semantics, not ordinary Hoare logic.

use vstd::prelude::*;
use vstd::resource::pcm::*;
use vstd::resource::Loc;
#[cfg(verus_keep_ghost)]
use vstd::resource::algebra::ResourceAlgebra;

verus! {

/// Resource location shared by probabilistic error credits.
pub uninterp spec fn credit_location() -> Loc;

/// Credit ownership: Value(epsilon) is valid exactly when 0 <= epsilon < 1.
/// Empty is the unit, and combining values sums their credit.
pub enum CreditCarrier {
    Value { amount: real },
    Empty,
    Invalid,
}

impl vstd::resource::algebra::ResourceAlgebra for CreditCarrier {
    closed spec fn valid(self) -> bool {
        match self {
            CreditCarrier::Value { amount } => 0real <= amount < 1real,
            CreditCarrier::Empty => true,
            CreditCarrier::Invalid => false,
        }
    }

    closed spec fn op(left: Self, right: Self) -> Self {
        match (left, right) {
            (CreditCarrier::Value { amount: a }, CreditCarrier::Value { amount: b }) => {
                if a < 0real || b < 0real { CreditCarrier::Invalid }
                else { CreditCarrier::Value { amount: a + b } }
            },
            (CreditCarrier::Empty, x) | (x, CreditCarrier::Empty) => x,
            _ => CreditCarrier::Invalid,
        }
    }

    proof fn valid_op(a: Self, b: Self) {}
    proof fn commutative(a: Self, b: Self) {}
    proof fn associative(a: Self, b: Self, c: Self) {}
}

impl PCM for CreditCarrier {
    closed spec fn unit() -> Self { CreditCarrier::Empty }

    proof fn op_unit(self) {}
    proof fn unit_valid() {}
}

pub struct ErrorCredit {
    resource: Resource<CreditCarrier>,
}

impl ErrorCredit {
    #[verifier::type_invariant]
    closed spec fn wf(self) -> bool {
        self.resource.loc() == credit_location()
    }

    pub closed spec fn view(self) -> CreditCarrier {
        self.resource.value()
    }
}

} // verus!
