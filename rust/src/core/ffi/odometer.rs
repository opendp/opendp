use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::{OdometerAnswer, OdometerQuery},
    ffi::{
        any::{
            AnyDomain, AnyMeasure, AnyMetric, AnyObject, AnyOdometer, AnyOdometerQueryable,
            Downcast, QueryOdometerInvokeType, QueryOdometerMapType,
        },
        util::{self, Type, into_c_char_p},
    },
};

use super::FfiResult;

pub type AnyOdometerQuery = OdometerQuery<AnyObject, AnyObject>;
pub type AnyOdometerAnswer = OdometerAnswer<AnyObject, AnyObject>;

#[bootstrap(
    name = "odometer_input_domain",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyDomain *>")
)]
/// Get the input domain from a `odometer`.
///
/// # Arguments
/// * `this` - The odometer to retrieve the value from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__odometer_input_domain(
    this: *mut AnyOdometer,
) -> FfiResult<*mut AnyDomain> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_domain.clone()))
}

#[bootstrap(
    name = "odometer_input_metric",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMetric *>")
)]
/// Get the input domain from a `odometer`.
///
/// # Arguments
/// * `this` - The odometer to retrieve the value from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__odometer_input_metric(
    this: *mut AnyOdometer,
) -> FfiResult<*mut AnyMetric> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).input_metric.clone()))
}

#[bootstrap(
    name = "odometer_output_measure",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<AnyMeasure *>")
)]
/// Get the output domain from a `odometer`.
///
/// # Arguments
/// * `this` - The odometer to retrieve the value from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__odometer_output_measure(
    this: *mut AnyOdometer,
) -> FfiResult<*mut AnyMeasure> {
    FfiResult::Ok(util::into_raw(try_as_ref!(this).output_measure.clone()))
}

#[bootstrap(
    name = "odometer_invoke",
    arguments(
        this(rust_type = b"null"),
        arg(rust_type = "$odometer_input_carrier_type(this)")
    )
)]
/// Invoke the `odometer` with `arg`. Returns a differentially private release.
///
/// # Arguments
/// * `this` - Odometer to invoke.
/// * `arg` - Input data to supply to the odometer. A member of the odometer's input domain.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__odometer_invoke(
    this: *const AnyOdometer,
    arg: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let this = try_as_ref!(this);
    let arg = try_as_ref!(arg);
    this.invoke(arg).into()
}

#[bootstrap(
    name = "odometer_input_carrier_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the input (carrier) data type of `this`.
///
/// # Arguments
/// * `this` - The odometer to retrieve the type from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__odometer_input_carrier_type(
    this: *mut AnyOdometer,
) -> FfiResult<*mut c_char> {
    let this = try_as_ref!(this);
    FfiResult::Ok(try_!(into_c_char_p(
        this.input_domain.carrier_type.descriptor.to_string()
    )))
}

#[bootstrap(
    name = "_odometer_free",
    arguments(this(do_not_convert = true)),
    returns(c_type = "FfiResult<void *>")
)]
/// Internal function. Free the memory associated with `this`.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core___odometer_free(this: *mut AnyOdometer) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[bootstrap(
    name = "odometer_queryable_invoke",
    arguments(
        queryable(rust_type = b"null"),
        query(rust_type = "$odometer_queryable_invoke_type(queryable)")
    )
)]
/// Eval the odometer `queryable` with an invoke `query`.
///
/// # Arguments
/// * `queryable` - Queryable to eval.
/// * `query` - Invoke query to supply to the queryable.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__odometer_queryable_invoke(
    queryable: *mut AnyObject,
    query: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(queryable).downcast_mut::<AnyOdometerQueryable>());
    let query = AnyOdometerQuery::Invoke(try_as_ref!(query).clone());

    let answer = try_!(queryable.eval(&query));
    if let AnyOdometerAnswer::Invoke(answer) = answer {
        Ok(answer).into()
    } else {
        err!(FailedCast, "return type is a d_out").into()
    }
}

#[bootstrap(
    name = "odometer_queryable_invoke_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the invoke query type of an odometer `queryable`.
///
/// # Arguments
/// * `this` - The queryable to retrieve the query type from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__odometer_queryable_invoke_type(
    this: *mut AnyObject,
) -> FfiResult<*mut c_char> {
    let this = try_as_mut_ref!(this);
    let this = try_!(this.downcast_mut::<AnyOdometerQueryable>());
    let answer: Type = try_!(this.eval_internal(&QueryOdometerInvokeType));
    FfiResult::Ok(try_!(into_c_char_p(answer.descriptor.to_string())))
}

#[bootstrap(
    name = "odometer_queryable_privacy_loss_type",
    arguments(this(rust_type = b"null")),
    returns(c_type = "FfiResult<char *>")
)]
/// Get the map query type of an odometer `queryable`.
///
/// # Arguments
/// * `this` - The queryable to retrieve the query type from.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__odometer_queryable_privacy_loss_type(
    this: *mut AnyObject,
) -> FfiResult<*mut c_char> {
    let this = try_as_mut_ref!(this);
    let this = try_!(this.downcast_mut::<AnyOdometerQueryable>());
    let answer: Type = try_!(this.eval_internal(&QueryOdometerMapType));
    FfiResult::Ok(try_!(into_c_char_p(answer.descriptor.to_string())))
}

#[bootstrap(
    name = "odometer_queryable_privacy_loss",
    arguments(
        queryable(rust_type = b"null"),
        d_in(rust_type = "$odometer_queryable_privacy_loss_type(queryable)")
    )
)]
/// Retrieve the privacy loss of an odometer `queryable`.
///
/// # Arguments
/// * `queryable` - Queryable to eval.
/// * `d_in` - Maximum distance between adjacent inputs in the input domain.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_core__odometer_queryable_privacy_loss(
    queryable: *mut AnyObject,
    d_in: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let queryable = try_!(try_as_mut_ref!(queryable).downcast_mut::<AnyOdometerQueryable>());
    let d_in = try_as_ref!(d_in).clone();
    queryable.privacy_loss(d_in).into()
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{Function, Odometer},
        domains::{AtomDomain, VectorDomain},
        error::{ErrorVariant, Fallible},
        ffi::any::Downcast,
        interactive::Queryable,
        measures::MaxDivergence,
        metrics::SymmetricDistance,
    };

    use super::*;

    fn make_test_any_odometer() -> Fallible<AnyOdometer> {
        Odometer::new(
            AnyDomain::new(VectorDomain::new(AtomDomain::<i32>::default())),
            AnyMetric::new(SymmetricDistance),
            AnyMeasure::new(MaxDivergence),
            Function::new_fallible(|arg: &AnyObject| {
                let data = arg.downcast_ref::<Vec<i32>>()?.clone();
                Queryable::new_external(move |query: &OdometerQuery<AnyObject, AnyObject>| {
                    Ok(match query {
                        OdometerQuery::Invoke(idx) => {
                            let idx = idx.downcast_ref::<usize>()?;
                            let res = data[*idx].clone();
                            OdometerAnswer::Invoke(AnyObject::new(res))
                        }
                        OdometerQuery::PrivacyLoss(_) => {
                            OdometerAnswer::PrivacyLoss(AnyObject::new(0.0))
                        }
                    })
                })
            }),
        )
    }

    #[test]
    fn test_odometer_invoke() -> Fallible<()> {
        let odometer = util::into_raw(make_test_any_odometer()?);
        let arg = util::into_raw(AnyObject::new(vec![999]));
        let res = opendp_core__odometer_invoke(odometer, arg);

        let mut res: AnyOdometerQueryable = Fallible::from(res)?.downcast()?;

        let query = AnyOdometerQuery::Invoke(AnyObject::new(0usize));
        let AnyOdometerAnswer::Invoke(res) = res.eval(&query)? else {
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
