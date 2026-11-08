use std::{os::raw::c_char, path::Path};

use opendp_derive::bootstrap;
use polars::lazy::frame::LazyFrame;

use crate::{
    core::FfiResult,
    ffi::{
        any::{AnyObject, AnyQueryable, Downcast},
        util,
    },
    polars::{ExtractLazyFrame, OnceFrameAnswer, OnceFrameQuery, TakeLazyFrame, split::split_plan},
};

#[bootstrap(
    name = "onceframe_collect",
    arguments(onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"))
)]
/// Internal function. Collects a DataFrame from a OnceFrame, exhausting the OnceFrame.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__onceframe_collect(
    onceframe: *mut AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let query = AnyObject::new(OnceFrameQuery::Collect);
    let answer: OnceFrameAnswer = try_!(try_!(queryable.eval(&query)).downcast());
    let OnceFrameAnswer::Collect(frame) = answer;

    Ok(AnyObject::new(frame)).into()
}

#[bootstrap(
    features("honest-but-curious"),
    name = "onceframe_lazy",
    arguments(onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"))
)]
/// Internal function. Extracts a LazyFrame from a OnceFrame,
/// circumventing protections against multiple evaluations.
///
/// Each collection consumes the entire allocated privacy budget.
/// To remain DP at the advertised privacy level, only collect the LazyFrame once.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
///
/// # Why honest-but-curious?
/// The privacy guarantees only apply if:
///
/// 1. The LazyFrame (compute plan) is only ever executed once.
/// 2. The analyst does not observe ordering of rows in the output.
///
/// To ensure that row ordering is not observed:
///
/// 1. Do not extend the compute plan with order-sensitive computations.
/// 2. Shuffle the output once collected ([in Polars sample all, with shuffling enabled](https://docs.pola.rs/api/python/stable/reference/dataframe/api/polars.DataFrame.sample.html)).
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__onceframe_lazy(
    onceframe: *mut AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());

    let answer: LazyFrame = try_!(queryable.eval_internal(&ExtractLazyFrame));
    Ok(AnyObject::new(answer)).into()
}

#[bootstrap(
    features("contrib"),
    name = "onceframe_split",
    arguments(
        onceframe(c_type = "AnyObject *", rust_type = "AnyQueryable"),
        work_dir(rust_type = "String")
    )
)]
/// Internal executor-only function. Consumes a OnceFrame and splits its native plan.
///
/// The OnceFrame is exhausted before splitting, even if splitting fails.
/// Returns an ordered list of native LazyFrames: ordinary executable deterministic
/// heads followed by the randomized continuation. Head i corresponds to
/// work_dir/i.arrow. Splitting performs no execution, I/O or schema resolution.
/// Additional sources outside the opaque private boundary are unsupported.
///
/// # Arguments
/// * `onceframe` - The queryable holding a LazyFrame.
/// * `work_dir` - Protected, executor-owned directory for intermediate IPC files.
///
/// # Internal executor requirements
/// The plans and the intermediate files expose deterministic private information.
/// They are internal executor artifacts, not differentially private releases, and
/// must never be exposed to the analyst. The trusted executor must execute each
/// deterministic head exactly once, materialize its
/// complete tabular result at work_dir/i.arrow, then execute the continuation once.
/// It must protect and clean up all intermediate files on success or failure,
/// preserve native plugins and plan semantics, and shuffle the complete final
/// output before releasing it without exposing row order. Do not retry execution
/// after consumption or extend the plans with analyst-controlled computations.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__onceframe_split(
    onceframe: *mut AnyObject,
    work_dir: *const c_char,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(onceframe).downcast_mut::<AnyQueryable>());
    let lazyframe: LazyFrame = try_!(queryable.eval_internal(&TakeLazyFrame));
    let work_dir = Path::new(try_!(util::to_str(work_dir)));
    let plans = try_!(split_plan(lazyframe.logical_plan, work_dir));
    Ok(AnyObject::new(
        plans
            .into_iter()
            .map(|plan| AnyObject::new(LazyFrame::from(plan)))
            .collect::<Vec<_>>(),
    ))
    .into()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::Fallible;
    use crate::{
        ffi::any::{AnyObject, AnyQueryable, Downcast},
        interactive::{Answer, Query},
        measurements::expr_noisy_max::NoisyMaxPlugin,
        polars::{OnceFrame, apply_anonymous_function},
        transformations::expr_discrete_quantile_score::DiscreteQuantileScorePlugin,
    };
    use polars::prelude::*;
    use std::ffi::CString;

    #[test]
    fn test_onceframe_split_ffi_vector_and_consumption() -> Fallible<()> {
        let input = df!("A" => [1i32, 3, 6])?.lazy();
        let scores = apply_anonymous_function(
            vec![col("A")],
            DiscreteQuantileScorePlugin {
                candidates: Series::new("".into(), [1i32, 3, 6]),
                alpha: (1, 2),
                size_limit: 10,
            },
        );
        let release = apply_anonymous_function(
            vec![scores],
            NoisyMaxPlugin {
                replacement: false,
                scale: 0.,
                negate: true,
            },
        );
        let private = input
            .clone()
            .select([release.clone().alias("a"), release.alias("b")]);
        let extra_source =
            private
                .clone()
                .join(input.clone(), [col("a")], [col("A")], JoinType::Left.into());
        let work_dir = CString::new("unused").unwrap();
        // Success, unsupported plans/sources, and invalid UTF-8 all exhaust the
        // same shared state before returning across the FFI.
        for (plan, path, succeeds) in [
            (private, work_dir.clone(), true),
            (extra_source, work_dir.clone(), false),
            (input.clone(), work_dir, false),
            (input, CString::new(vec![0xff]).unwrap(), false),
        ] {
            let mut onceframe = OnceFrame::from(plan);
            let mut alias = onceframe.clone();
            let queryable = AnyQueryable::new_raw(move |_, query: Query<AnyObject>| {
                Ok(match query {
                    Query::External(q) => {
                        Answer::External(AnyObject::new(onceframe.eval(q.downcast_ref()?)?))
                    }
                    Query::Internal(q) => {
                        let Answer::Internal(a) = onceframe.eval_query(Query::Internal(q))? else {
                            unreachable!()
                        };
                        Answer::Internal(a)
                    }
                })
            });
            let mut object = AnyObject::new(queryable);
            let result: Fallible<AnyObject> =
                opendp_data__onceframe_split(&mut object, path.as_ptr()).into();
            if succeeds {
                let plans: Vec<AnyObject> = result?.downcast()?;
                assert_eq!(
                    plans.len(),
                    3,
                    "two deterministic heads, final continuation"
                );
                let plans = plans
                    .into_iter()
                    .map(|p| p.downcast::<LazyFrame>())
                    .collect::<Fallible<Vec<_>>>()?;
                assert!(matches!(plans[0].logical_plan, DslPlan::GroupBy { .. }));
                assert!(matches!(plans[1].logical_plan, DslPlan::GroupBy { .. }));
                assert!(matches!(plans[2].logical_plan, DslPlan::Select { .. }));
            } else {
                assert!(result.is_err());
            }
            assert!(alias.eval(&OnceFrameQuery::Collect).is_err());
            assert!(alias.eval_internal::<LazyFrame>(&ExtractLazyFrame).is_err());
            assert!(alias.eval_internal::<LazyFrame>(&TakeLazyFrame).is_err());
            let retry: Fallible<AnyObject> =
                opendp_data__onceframe_split(&mut object, path.as_ptr()).into();
            assert!(retry.is_err());
        }
        Ok(())
    }
}
