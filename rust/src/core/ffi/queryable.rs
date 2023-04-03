use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::Fallible,
    ffi::{
        any::{AnyObject, AnyQueryable, Downcast, QueryOdometerMapType, QueryType},
        util::{self, c_bool, into_c_char_p, Type},
    },
    interactive::{Answer, Query, Queryable},
};

use super::FfiResult;

#[bootstrap(
    name = "queryable_eval",
    arguments(
        queryable(rust_type = b"null"),
        query(rust_type = "$queryable_query_type(queryable)")
    )
)]
/// Eval the `queryable` with `query`. Returns a differentially private release.
///
/// # Arguments
/// * `queryable` - Queryable to eval.
/// * `query` - The input to the queryable.
#[no_mangle]
pub extern "C" fn opendp_core__queryable_eval(
    queryable: *mut AnyObject,
    query: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_as_mut_ref!(queryable);
    let queryable = try_!(queryable.downcast_mut::<AnyQueryable>());
    let query = try_as_ref!(query);
    queryable.eval(query).into()
}

#[bootstrap(
    name = "queryable_query_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the query type of `queryable`.
///
/// # Arguments
/// * `this` - The queryable to retrieve the query type from.
#[no_mangle]
pub extern "C" fn opendp_core__queryable_query_type(
    this: *mut AnyObject,
) -> FfiResult<*mut c_char> {
    let this = try_as_mut_ref!(this);
    let this = try_!(this.downcast_mut::<AnyQueryable>());
    let answer: Type = try_!(this.eval_internal(&QueryType));
    FfiResult::Ok(try_!(into_c_char_p(answer.descriptor.to_string())))
}

#[bootstrap(
    name = "queryable_query_odometer_map_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the query odometer map type of `queryable`.
///
/// # Arguments
/// * `this` - The queryable to retrieve the type from.
#[no_mangle]
pub extern "C" fn opendp_core__queryable_query_odometer_map_type(
    this: *mut AnyObject,
) -> FfiResult<*mut c_char> {
    let this = try_as_mut_ref!(this);
    let this = try_!(this.downcast_mut::<AnyQueryable>());
    let answer: Type = try_!(this.eval_internal(&QueryOdometerMapType));
    FfiResult::Ok(try_!(into_c_char_p(answer.descriptor.to_string())))
}

type TransitionFn = extern "C" fn(*const AnyObject, c_bool) -> *mut FfiResult<*mut AnyObject>;

// wrap a TransitionFn in a closure, so that it can be used in Queryables
fn wrap_transition(
    transition: TransitionFn,
    Q: Type,
) -> impl FnMut(&AnyQueryable, Query<AnyObject>) -> Fallible<Answer<AnyObject>> {
    fn eval(transition: &TransitionFn, q: &AnyObject, is_internal: bool) -> Fallible<AnyObject> {
        util::into_owned(transition(
            q as *const AnyObject,
            util::from_bool(is_internal),
        ))?
        .into()
    }

    move |_self: &AnyQueryable, arg: Query<AnyObject>| -> Fallible<Answer<AnyObject>> {
        Ok(match arg {
            Query::External(q, wrapper) => {
                if wrapper.is_some() {
                    return fallible!(
                        FailedFunction,
                        "queryable wrapping is not supported across languages"
                    );
                }
                Answer::External(eval(&transition, q, false)?)
            }
            Query::Internal(q) => {
                if q.downcast_ref::<QueryType>().is_some() {
                    return Ok(Answer::internal(Q.clone()));
                }
                let q = q
                    .downcast_ref::<AnyObject>()
                    .ok_or_else(|| err!(FFI, "failed to downcast internal query to AnyObject"))?;

                Answer::Internal(Box::new(eval(&transition, q, true)?))
            }
        })
    }
}

#[bootstrap(
    name = "new_queryable",
    features("contrib"),
    arguments(transition(rust_type = "$pass_through(A)")),
    generics(Q(default = "ExtrinsicObject"), A(default = "ExtrinsicObject")),
    dependencies("c_transition")
)]
/// Construct a queryable from a user-defined transition function.
///
/// # Arguments
/// * `transition` - A transition function taking a reference to self, a query, and an internal/external indicator
///
/// # Generics
/// * `Q` - Query Type
/// * `A` - Output Type
#[allow(dead_code)]
fn new_queryable<Q, A>(transition: TransitionFn) -> Fallible<AnyObject> {
    let _ = transition;
    panic!("this signature only exists for code generation")
}

#[no_mangle]
pub extern "C" fn opendp_core__new_queryable(
    transition: TransitionFn,
    Q: *const c_char,
    A: *const c_char,
) -> FfiResult<*mut AnyObject> {
    let Q = try_!(Type::try_from(Q));
    let _A = A;
    FfiResult::Ok(util::into_raw(AnyObject::new(try_!(Queryable::new(
        wrap_transition(transition, Q),
        None
    )))))
}
