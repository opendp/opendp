//! Probe Verus's ability to prove dispatch through a single-trait Arc.
//! This is verification-only, not a new OpenDP runtime abstraction.
use vstd::prelude::*;
use std::sync::Arc;

verus! {
pub trait Callable {
    fn eval(&self, input: u64) -> (output: u64)
        requires input <= 100,
        ensures output == input + 1;
}

pub struct OneAdder {}

impl Callable for OneAdder {
    fn eval(&self, input: u64) -> (output: u64) {
        input + 1
    }
}

pub fn make_arc_callback() -> (out: Arc<dyn Callable>) {
    Arc::new(OneAdder {})
}

pub fn invoke_arc_callback(callback: &Arc<dyn Callable>, input: u64)
    -> (output: u64)
    requires input <= 100,
    ensures output == input + 1,
{
    callback.eval(input)
}
}
