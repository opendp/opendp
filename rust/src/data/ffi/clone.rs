use super::*;
use crate::ffi::any::AnyOdometerQueryable;

/// Clone an opaque object that must outlive a borrowed callback argument.
/// Unlike ordinary values, these objects cannot be copied into R vectors.
/// The caller owns the returned object; the source remains borrowed.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__clone_opaque_object(
    obj: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let obj = try_as_ref!(obj);
    fn clone_plain<T: 'static + Clone>(obj: &AnyObject) -> Fallible<AnyObject> {
        Ok(AnyObject::new(obj.downcast_ref::<T>()?.clone()))
    }
    dispatch!(
        clone_plain,
        [(
            obj.type_,
            [PrivacyProfile, AnyQueryable, AnyOdometerQueryable]
        )],
        (obj)
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_profile_clone_outlives_source() -> Fallible<()> {
        let source = AnyObject::new(PrivacyProfile::new(|epsilon| Ok((-epsilon).exp())));
        let cloned: AnyObject = Fallible::from(opendp_data__clone_opaque_object(&source))?;
        drop(source);
        assert_eq!(cloned.downcast_ref::<PrivacyProfile>()?.delta(0.)?, 1.);
        Ok(())
    }

    #[test]
    fn unsupported_clone_is_fallible() {
        let source = AnyObject::new(42_i32);
        let cloned: Fallible<AnyObject> = opendp_data__clone_opaque_object(&source).into();
        assert!(cloned.is_err());
    }
}
