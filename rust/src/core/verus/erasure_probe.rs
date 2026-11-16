//! Verify whether Verus can check Arc<dyn Fn> construction and invocation.
//! This isolated probe is compiled only in the Verus harness, not production.
use vstd::prelude::*;
use std::sync::Arc;

verus! {
pub fn make_arc_callback<F: Fn(&u64) -> u64 + 'static>(
    f: F,
) -> (out: Arc<dyn Fn(&u64) -> u64>) {
    Arc::new(f)
}

pub fn invoke_arc_callback(
    callback: &Arc<dyn Fn(&u64) -> u64>,
    input: &u64,
) -> (out: u64) {
    callback(input)
}
}
