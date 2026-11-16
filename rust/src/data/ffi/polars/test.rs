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

#[test]
fn test_onceframe_collector_ffi_protocol() -> Fallible<()> {
    use crate::ffi::util::{_set_ref_count, ExtrinsicObject, into_raw};
    use std::{cell::Cell, ffi::c_void, rc::Rc};

    struct State {
        calls: Cell<usize>,
        fail: bool,
    }
    extern "C" fn ref_count(ptr: *const c_void, increment: bool) -> bool {
        unsafe {
            if increment {
                Rc::increment_strong_count(ptr.cast::<State>());
            } else {
                Rc::decrement_strong_count(ptr.cast::<State>());
            }
        }
        true
    }
    extern "C" fn callback(
        arg: *const AnyObject,
        userdata: *const c_void,
    ) -> *mut FfiResult<*mut AnyObject> {
        let result = (|| -> Fallible<AnyObject> {
            let state = unsafe { &*userdata.cast::<State>() };
            state.calls.set(state.calls.get() + 1);
            if *unsafe { &*arg }.downcast_ref::<bool>()? {
                return fallible!(FailedFunction, "expected false callback argument");
            }
            if state.fail {
                return fallible!(FailedFunction, "external collector failure detail");
            }
            Ok(AnyObject::new(df!("A" => [3, 1, 2])?))
        })();
        into_raw(result.into())
    }

    _set_ref_count(ref_count);
    for fail in [false, true] {
        let state = Rc::new(State {
            calls: Cell::new(0),
            fail,
        });
        let weak = Rc::downgrade(&state);
        let collector = CallbackFn {
            callback,
            userdata: ExtrinsicObject(Rc::into_raw(state).cast()),
        };
        let object: Fallible<AnyObject> = opendp_data__onceframe_with_collector(&collector).into();
        let mut object = object?;
        drop(collector);
        assert_eq!(weak.upgrade().unwrap().calls.get(), 0);
        let mut alias = object.downcast_ref::<AnyQueryable>()?.clone();
        assert_eq!(
            alias.eval_internal::<Type>(&QueryType)?,
            Type::of::<OnceFrameQuery>()
        );
        let lazy: Fallible<AnyObject> = opendp_data__onceframe_lazy(&mut object).into();
        assert!(lazy.is_err());
        let path = CString::new("unused").unwrap();
        let split: Fallible<AnyObject> =
            opendp_data__onceframe_split(&mut object, path.as_ptr()).into();
        assert!(split.is_err());
        assert_eq!(weak.upgrade().unwrap().calls.get(), 0);
        let result: Fallible<AnyObject> = opendp_data__onceframe_collect(&mut object).into();
        if fail {
            assert_eq!(
                result.err().unwrap().message.as_deref(),
                Some("external collector failure detail")
            );
        } else {
            assert_eq!(result?.downcast::<DataFrame>()?, df!("A" => [3, 1, 2])?);
        }
        assert!(
            weak.upgrade().is_none(),
            "callback released after invocation"
        );
        assert!(
            alias
                .eval(&AnyObject::new(OnceFrameQuery::Collect))
                .is_err()
        );
        assert!(alias.eval_internal::<LazyFrame>(&ExtractLazyFrame).is_err());
        assert!(alias.eval_internal::<LazyFrame>(&TakeLazyFrame).is_err());
        let retry: Fallible<AnyObject> = opendp_data__onceframe_collect(&mut object).into();
        assert!(retry.is_err());
    }
    Ok(())
}
