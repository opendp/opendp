use opendp_derive::bootstrap;

use crate::{
    combinators::ffi::{AnyOdometerAnswer, AnyOdometerQuery},
    ffi::{
        any::{
            AnyDomain, AnyFunction, AnyMeasure, AnyMetric, AnyObject, AnyOdometer, AnyQueryable,
            Downcast,
        },
        util,
    },
};

use super::FfiResult;

#[bootstrap(
    name = "odometer_input_domain",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>", do_not_convert = true)
)]
/// Get the input domain from a `odometer`.
///
/// # Arguments
/// * `this` - The odometer to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__odometer_input_domain(
    this: *mut AnyOdometer,
) -> FfiResult<*mut AnyDomain> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_domain.clone()))
}

#[bootstrap(
    name = "odometer_input_metric",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMetric *>", do_not_convert = true)
)]
/// Get the input domain from a `odometer`.
///
/// # Arguments
/// * `this` - The odometer to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__odometer_input_metric(
    this: *mut AnyOdometer,
) -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_metric.clone()))
}

#[bootstrap(
    name = "odometer_output_measure",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMeasure *>", do_not_convert = true)
)]
/// Get the output domain from a `odometer`.
///
/// # Arguments
/// * `this` - The odometer to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__odometer_output_measure(
    this: *mut AnyOdometer,
) -> FfiResult<*mut AnyMeasure> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).output_measure.clone()))
}

#[bootstrap(
    name = "odometer_function",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyFunction *>", do_not_convert = true)
)]
/// Get the function from a odometer.
///
/// # Arguments
/// * `this` - The odometer to retrieve the value from.
#[no_mangle]
pub extern "C" fn opendp_core__odometer_function(
    this: *mut AnyOdometer,
) -> FfiResult<*mut AnyFunction> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).function.clone()))
}

#[bootstrap(
    name = "odometer_invoke",
    arguments(
        this(rust_type = b"null"),
        arg(rust_type = "$get_carrier_type(get_input_domain(this))")
    )
)]
/// Invoke the `odometer` with `arg`. Returns a differentially private release.
///
/// # Arguments
/// * `this` - Odometer to invoke.
/// * `arg` - Input data to supply to the odometer. A member of the odometer's input domain.
#[no_mangle]
pub extern "C" fn opendp_core__odometer_invoke(
    this: *const AnyOdometer,
    arg: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let this = try_as_ref!(this);
    let arg = try_as_ref!(arg);
    this.invoke(arg).into()
}

#[bootstrap(
    name = "_odometer_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[no_mangle]
pub extern "C" fn opendp_core___odometer_free(this: *mut AnyOdometer) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "odometer_queryable_invoke",
    arguments(
        queryable(rust_type = b"null"),
        query(rust_type = "$queryable_query_odometer_invoke_type(queryable)")
    )
)]
/// Eval the odometer `queryable` with an invoke `query`. Returns a differentially private release.
///
/// # Arguments
/// * `queryable` - Queryable to eval.
/// * `query` - Invoke query to supply to the queryable.
#[no_mangle]
pub extern "C" fn opendp_core__odometer_queryable_invoke(
    queryable: *mut AnyObject,
    query: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(queryable).downcast_mut::<AnyQueryable>());
    let query = AnyObject::new(AnyOdometerQuery::Invoke(try_as_ref!(query).clone()));
    let answer = try_!(try_!(queryable.eval(&query)).downcast::<AnyOdometerAnswer>());
    if let AnyOdometerAnswer::Invoke(answer) = answer {
        Ok(answer).into()
    } else {
        fallible!(FailedCast, "return type is a d_out").into()
    }
}

#[bootstrap(
    name = "odometer_queryable_map",
    arguments(
        queryable(rust_type = b"null"),
        query(rust_type = "$queryable_query_odometer_map_type(queryable)")
    )
)]
/// Eval the odometer `queryable` with a map `query`. Returns the current d_out.
///
/// # Arguments
/// * `queryable` - Queryable to eval.
/// * `query` - Map query to supply to the queryable.
#[no_mangle]
pub extern "C" fn opendp_core__odometer_queryable_map(
    queryable: *mut AnyObject,
    query: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(queryable).downcast_mut::<AnyQueryable>());
    let query = AnyObject::new(AnyOdometerQuery::Map(try_as_ref!(query).clone()));
    let answer = try_!(try_!(queryable.eval(&query)).downcast::<AnyOdometerAnswer>());
    if let AnyOdometerAnswer::Map(answer) = answer {
        Ok(answer).into()
    } else {
        fallible!(FailedCast, "return type is not d_out").into()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        combinators::{ffi::AnyOdometerQuery, OdometerAnswer, OdometerQuery},
        core::{Function, Odometer},
        domains::{AtomDomain, VectorDomain},
        error::{ErrorVariant, Fallible},
        ffi::any::{AnyQueryable, Downcast},
        interactive::Queryable,
        measures::MaxDivergence,
        metrics::{IntDistance, SymmetricDistance},
    };

    use super::*;

    fn make_test_any_odometer() -> Fallible<AnyOdometer> {
        Odometer::new(
            AnyDomain::new(VectorDomain::new(AtomDomain::<i32>::default())),
            Function::new_fallible(|arg: &AnyObject| {
                let data = arg.downcast_ref::<Vec<i32>>()?.clone();
                Queryable::new_external(move |query: &OdometerQuery<AnyObject, AnyObject>| {
                    Ok(match query {
                        OdometerQuery::Invoke(idx) => {
                            let idx = idx.downcast_ref::<usize>()?;
                            let res = data[*idx].clone();
                            OdometerAnswer::Invoke(AnyObject::new(res))
                        }
                        OdometerQuery::Map(d_in) => {
                            let d_in = d_in.downcast_ref::<IntDistance>()?;
                            let res = *d_in as f64 + 1.;
                            OdometerAnswer::Map(AnyObject::new(res))
                        }
                    })
                })
            }),
            AnyMetric::new(SymmetricDistance::default()),
            AnyMeasure::new(MaxDivergence::<f64>::default()),
        )
        .map(Odometer::into_any_queryable)
        .map(Odometer::into_any_out)
    }

    #[test]
    fn test_odometer_invoke() -> Fallible<()> {
        let odometer = util::into_raw(make_test_any_odometer()?);
        let arg = util::into_raw(AnyObject::new(vec![999]));
        let res = opendp_core__odometer_invoke(odometer, arg);

        let mut res: AnyQueryable = Fallible::from(res)?.downcast()?;

        let query = AnyObject::new(AnyOdometerQuery::Invoke(AnyObject::new(0usize)));
        let AnyOdometerAnswer::Invoke(res) = res.eval(&query)?.downcast()? else {
            panic!("expecting invoke")
        };
        let res: i32 = res.downcast()?;
        assert_eq!(res, 999);
        Ok(())
    }

    #[test]
    fn test_odometer_invoke_wrong_type() -> Fallible<()> {
        let odometer = util::into_raw(make_test_any_odometer()?);
        let arg = util::into_raw(AnyObject::new(vec![999.0]));
        let res = Fallible::from(opendp_core__odometer_invoke(odometer, arg));

        assert_eq!(res.err().unwrap().variant, ErrorVariant::FailedCast);
        Ok(())
    }
}
