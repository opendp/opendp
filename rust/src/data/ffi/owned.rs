//! Ownership-safe scratch views. The legacy slice API remains unchanged.

use std::any::Any;

use super::*;

#[cfg(feature = "polars")]
#[path = "owned_polars.rs"]
mod owned_polars;

/// Opaque owner of an FfiSlice header and its scratch allocations.
///
/// The source AnyObject is borrowed, not retained: it must outlive this owner
/// and every use of the view. View pointers are read-only, except for the
/// owner-allocated ArrowArray/ArrowSchema scratch headers: Arrow C consumers may
/// move/release their payloads and mark the headers released. These headers use
/// interior mutability; the borrowed source is never mutated. The owner must
/// still free the header boxes, even after payload consumption. Child AnyObjects
/// and pointer arrays belong to this owner and must not be freed separately.
pub struct FfiSliceOwner {
    view: FfiSlice,
    keepalives: Vec<Box<dyn Any>>,
}

impl FfiSliceOwner {
    fn new() -> Self {
        Self {
            view: FfiSlice::new(std::ptr::null_mut(), 0),
            keepalives: Vec::new(),
        }
    }

    fn keep<T: 'static>(&mut self, value: T) -> *const T {
        let value = Box::new(value);
        let ptr = &*value as *const T;
        self.keepalives.push(value);
        ptr
    }

    fn pointers<T: 'static>(&mut self, pointers: Vec<*const T>) {
        self.view = FfiSlice::new(pointers.as_ptr() as *mut c_void, pointers.len());
        self.keep(pointers);
    }

    fn string(&mut self, value: &str) -> Fallible<*const c_char> {
        let value = CString::new(value.as_bytes())
            .map_err(|e| err!(FFI, "Unable to represent string as a C string: {}", e))?;
        let ptr = value.as_ptr();
        self.keep(value);
        Ok(ptr)
    }

    fn from_object(obj: &AnyObject) -> Fallible<Self> {
        let mut owner = Self::new();
        match &obj.type_.contents {
            TypeContents::PLAIN("String") => {
                let ptr = owner.string(obj.downcast_ref::<String>()?)?;
                owner.pointers(vec![ptr]);
            }
            TypeContents::VEC(element_id) => {
                let element = Type::of_id(element_id)?;
                #[cfg(feature = "polars")]
                if element == Type::of::<Expr>() {
                    owner.polars_vector(obj.downcast_ref::<Vec<Expr>>()?);
                    return Ok(owner);
                } else if element == Type::of::<Bound>() {
                    owner.polars_vector(obj.downcast_ref::<Vec<Bound>>()?);
                    return Ok(owner);
                }
                if element == Type::of::<String>() {
                    let pointers = obj
                        .downcast_ref::<Vec<String>>()?
                        .iter()
                        .map(|s| owner.string(s))
                        .collect::<Fallible<Vec<_>>>()?;
                    owner.pointers(pointers);
                } else if element == Type::of::<AnyObject>() {
                    owner.pointers(
                        obj.downcast_ref::<Vec<AnyObject>>()?
                            .iter()
                            .map(|v| v as *const AnyObject)
                            .collect(),
                    );
                } else {
                    // Numeric vectors borrow the source allocation directly.
                    fn borrowed_vec<T: 'static>(obj: &AnyObject) -> Fallible<FfiSlice> {
                        let values = obj.downcast_ref::<Vec<T>>()?;
                        Ok(FfiSlice::new(values.as_ptr() as *mut c_void, values.len()))
                    }
                    owner.view = dispatch!(
                        borrowed_vec,
                        [(
                            element,
                            [u32, u64, i32, i64, usize, f32, f64, bool, ExtrinsicObject]
                        )],
                        (obj)
                    )?;
                }
            }
            TypeContents::TUPLE(ids) => {
                let types = ids.iter().map(Type::of_id).collect::<Fallible<Vec<_>>>()?;
                match types.len() {
                    2 => {
                        if types == vec![Type::of::<PrivacyProfile>(), Type::of::<f64>()] {
                            let (profile, delta) = obj.downcast_ref::<(PrivacyProfile, f64)>()?;
                            let profile = owner.keep(AnyObject::new(profile.clone()));
                            owner.pointers(vec![
                                profile.cast::<c_void>(),
                                (delta as *const f64).cast(),
                            ]);
                        } else if types == vec![Type::of::<f64>(), Type::of::<ExtrinsicObject>()] {
                            // Legacy erased output payload: the candidate is an AnyObject,
                            // despite the public type descriptor saying ExtrinsicObject.
                            tuple2::<f64, AnyObject>(obj, &mut owner)?;
                        } else {
                            dispatch!(tuple2, [(types[0], @primitives_plus), (types[1], @primitives_plus)], (obj, &mut owner))?;
                        }
                    }
                    3 => {
                        check_l0Pinf_distance_types(&types)?;
                        dispatch!(tuple3, [(types[1], @numbers)], (obj, &mut owner))?;
                    }
                    _ => return fallible!(FFI, "Only tuples of length 2 or 3 are supported"),
                }
            }
            TypeContents::GENERIC {
                name: "Option",
                args,
            } => {
                let [element] = parse_type_args(args, "Option")?;
                if element == Type::of::<AnyObject>() {
                    if let Some(value) = obj.downcast_ref::<Option<AnyObject>>()? {
                        owner.view = FfiSlice::new(value as *const AnyObject as *mut c_void, 1);
                    }
                } else if let TypeContents::TUPLE(ids) = &element.contents {
                    let types = ids.iter().map(Type::of_id).collect::<Fallible<Vec<_>>>()?;
                    if types.len() != 2 {
                        return fallible!(
                            FFI,
                            "Only Option-wrapped tuples of length 2 are supported"
                        );
                    }
                    dispatch!(option_tuple2, [(types[0], @primitives_plus), (types[1], @primitives_plus)], (obj, &mut owner))?;
                } else {
                    return fallible!(FFI, "unsupported object type: {}", obj.type_.to_string());
                }
            }
            TypeContents::GENERIC {
                name: "HashMap",
                args,
            } => {
                let [key, value] = parse_type_args(args, "HashMap")?;
                if value == Type::of::<ExtrinsicObject>() {
                    dispatch!(hashmap, [(key, @hashable), (value, [ExtrinsicObject])], (obj, &mut owner))?;
                } else {
                    dispatch!(hashmap, [(key, @hashable), (value, @primitives)], (obj, &mut owner))?;
                }
            }
            #[cfg(feature = "polars")]
            TypeContents::PLAIN(
                "Expr" | "LazyFrame" | "ExprPlan" | "Bound" | "Bounds" | "Margin" | "Series"
                | "DataFrame",
            ) => owner.polars_plain(obj)?,
            TypeContents::PLAIN("BitVector") => {
                let bits = obj.downcast_ref::<BitVector>()?;
                owner.view = FfiSlice::new(bits.as_bitptr().pointer() as *mut c_void, bits.len());
            }
            TypeContents::PLAIN("ExtrinsicObject") => {
                let value = obj.downcast_ref::<ExtrinsicObject>()?;
                owner.view = FfiSlice::new(value as *const ExtrinsicObject as *mut c_void, 1);
            }
            _ => {
                // Plain values borrow the original AnyObject without scratch allocation.
                fn borrowed_plain<T: 'static>(obj: &AnyObject) -> Fallible<FfiSlice> {
                    let value = obj.downcast_ref::<T>()?;
                    Ok(FfiSlice::new(value as *const T as *mut c_void, 1))
                }
                owner.view = dispatch!(
                    borrowed_plain,
                    [(
                        obj.type_,
                        [
                            u8,
                            u16,
                            u32,
                            u64,
                            u128,
                            i8,
                            i16,
                            i32,
                            i64,
                            i128,
                            usize,
                            f32,
                            f64,
                            bool,
                            AnyMeasurement,
                            AnyQueryable
                        ]
                    )],
                    (obj)
                )?;
            }
        }
        Ok(owner)
    }
}

fn tuple2<T0: 'static, T1: 'static>(obj: &AnyObject, owner: &mut FfiSliceOwner) -> Fallible<()> {
    let (a, b) = obj.downcast_ref::<(T0, T1)>()?;
    owner.pointers(vec![
        (a as *const T0).cast::<c_void>(),
        (b as *const T1).cast(),
    ]);
    Ok(())
}

fn option_tuple2<T0: 'static, T1: 'static>(
    obj: &AnyObject,
    owner: &mut FfiSliceOwner,
) -> Fallible<()> {
    if let Some((a, b)) = obj.downcast_ref::<Option<(T0, T1)>>()? {
        owner.pointers(vec![
            (a as *const T0).cast::<c_void>(),
            (b as *const T1).cast(),
        ]);
    }
    Ok(())
}

fn tuple3<T: 'static>(obj: &AnyObject, owner: &mut FfiSliceOwner) -> Fallible<()> {
    let (a, b, c) = obj.downcast_ref::<(IntDistance, T, T)>()?;
    owner.pointers(vec![
        (a as *const IntDistance).cast::<c_void>(),
        (b as *const T).cast(),
        (c as *const T).cast(),
    ]);
    Ok(())
}

fn hashmap<K: 'static + Clone + Hash + Eq, V: 'static + Clone>(
    obj: &AnyObject,
    owner: &mut FfiSliceOwner,
) -> Fallible<()> {
    let map = obj.downcast_ref::<HashMap<K, V>>()?;
    let keys = owner.keep(AnyObject::new(map.keys().cloned().collect::<Vec<_>>()));
    let values = owner.keep(AnyObject::new(map.values().cloned().collect::<Vec<_>>()));
    owner.pointers(vec![keys, values]);
    Ok(())
}

/// Transfer an independent AnyFunction clone to the binding that receives it.
/// Functions are owned handles, not copied values, so no scratch view is involved.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__clone_function(
    obj: *const AnyObject,
) -> FfiResult<*mut AnyFunction> {
    let obj = try_as_ref!(obj);
    let TypeContents::GENERIC {
        name: "Function",
        args,
    } = &obj.type_.contents
    else {
        return err!(FFI, "expected Function object").into();
    };
    let [input, output] = try_!(parse_type_args(args, "Function"));
    fn clone_function<I: 'static + Clone, O: 'static>(
        obj: &AnyObject,
    ) -> Fallible<*mut AnyFunction> {
        Ok(util::into_raw(
            obj.downcast_ref::<Function<I, O>>()?.clone().into_any(),
        ))
    }
    dispatch!(
        clone_function,
        [(input, @primitives), (output, @primitives)],
        (obj)
    )
    .into()
}

/// Create a scratch view owner, borrowing obj until the owner is freed.
/// Strings use C strings (embedded NUL is an error); tuples use field pointer
/// arrays. Maps use pointers to owned key/value vector AnyObjects.
/// Vec<AnyObject> uses an array of borrowed AnyObject pointers, NOT contiguous
/// AnyObjects. With Polars enabled, Vec<Expr>, Vec<Bound>, and Bounds likewise
/// use pointer arrays, but their child AnyObjects are owned scratch clones.
/// Polars fields, serialization, and Arrow views retain the legacy layouts.
/// Other primitive data is borrowed directly from obj.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__object_as_slice_owned(
    obj: *const AnyObject,
) -> FfiResult<*mut FfiSliceOwner> {
    FfiSliceOwner::from_object(try_as_ref!(obj)).into()
}

/// Borrow the owner's read-only header. Do not pass it to slice_free.
/// Arrow payload consumers may mutate only the scratch Arrow headers described
/// on FfiSliceOwner; do not pass these views to legacy arrow_array_free.
/// The header and everything it references expire when owner is freed;
/// the source AnyObject must also remain alive throughout their use.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__slice_owner_view(
    owner: *const FfiSliceOwner,
) -> FfiResult<*const FfiSlice> {
    FfiResult::Ok(&try_as_ref!(owner).view as *const FfiSlice)
}

/// Free the owner, its header, and all typed scratch allocations exactly once.
/// Does not free the borrowed source AnyObject. Null is an error.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__slice_owner_free(owner: *mut FfiSliceOwner) -> FfiResult<*mut ()> {
    util::into_owned(owner).map(|_| ()).into()
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::ffi::CStr;
    use std::rc::Rc;

    use super::*;

    struct TestOwner(*mut FfiSliceOwner);

    impl TestOwner {
        fn new(obj: &AnyObject) -> Fallible<Self> {
            match opendp_data__object_as_slice_owned(obj) {
                FfiResult::Ok(owner) => Ok(Self(owner)),
                FfiResult::Err(err) => {
                    let result: Fallible<()> = FfiResult::<*mut ()>::Err(err).into();
                    Err(result.unwrap_err())
                }
            }
        }

        fn view(&self) -> &FfiSlice {
            match opendp_data__slice_owner_view(self.0) {
                FfiResult::Ok(view) => unsafe { &*view },
                FfiResult::Err(_) => panic!("view failed"),
            }
        }

        fn pointers<T>(&self) -> &[*const T] {
            let view = self.view();
            if view.len == 0 {
                return &[];
            }
            unsafe { slice::from_raw_parts(view.ptr.cast(), view.len) }
        }
    }

    impl Drop for TestOwner {
        fn drop(&mut self) {
            let result: Fallible<()> = opendp_data__slice_owner_free(self.0).into();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn owned_function_clone_survives_source_destruction() -> Fallible<()> {
        let source = AnyObject::new(Function::new(|value: &i32| value + 1));
        let function: AnyFunction = Fallible::from(opendp_data__clone_function(&source))?;
        drop(source);
        let answer = function.eval(&AnyObject::new(41_i32))?;
        assert_eq!(*answer.downcast_ref::<i32>()?, 42);
        Ok(())
    }

    #[test]
    fn owner_free_drops_scratch_and_leaves_source() -> Fallible<()> {
        struct Tracked(Rc<Cell<usize>>);
        impl Drop for Tracked {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let drops = Rc::new(Cell::new(0));
        let obj = AnyObject::new(42_i32);
        let owner = TestOwner::new(&obj)?;
        unsafe { &mut *owner.0 }.keep(Tracked(drops.clone()));
        assert_eq!(unsafe { *owner.view().ptr.cast::<i32>() }, 42);
        assert!(std::ptr::eq(owner.view(), owner.view()));
        drop(owner);
        assert_eq!(drops.get(), 1);
        assert_eq!(*obj.downcast_ref::<i32>()?, 42);

        let result: Fallible<()> = opendp_data__slice_owner_free(std::ptr::null_mut()).into();
        assert!(result.is_err());
        let result: Fallible<FfiSliceOwner> = opendp_data__object_as_slice_owned(null()).into();
        assert!(result.is_err());
        match opendp_data__slice_owner_view(null()) {
            FfiResult::Err(err) => {
                let result: Fallible<()> = FfiResult::<*mut ()>::Err(err).into();
                assert!(result.is_err());
            }
            _ => panic!("null view must fail"),
        }
        Ok(())
    }

    #[test]
    fn strings_empty_utf8_and_nul_errors() -> Fallible<()> {
        for value in ["", "hello", "λ"] {
            let obj = AnyObject::new(value.to_owned());
            let owner = TestOwner::new(&obj)?;
            assert_eq!(owner.view().len, 1);
            assert_eq!(
                unsafe { CStr::from_ptr(owner.pointers::<c_char>()[0]) }
                    .to_str()
                    .unwrap(),
                value
            );
        }
        for values in [vec![], vec![""], vec!["first", "", "λ"]] {
            let obj = AnyObject::new(values.iter().map(|s| s.to_string()).collect::<Vec<_>>());
            let owner = TestOwner::new(&obj)?;
            let actual = owner
                .pointers::<c_char>()
                .iter()
                .map(|&p| unsafe { CStr::from_ptr(p) }.to_str().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(actual, values);
        }
        // The last conversion fails after several scratch allocations already exist.
        // Repetition also exercises the cleanup path under leak checkers.
        for _ in 0..32 {
            assert!(TestOwner::new(&AnyObject::new("a\0b".to_string())).is_err());
            assert!(
                TestOwner::new(&AnyObject::new(vec![
                    "valid".to_string(),
                    "a\0b".to_string()
                ]))
                .is_err()
            );
        }
        Ok(())
    }

    #[test]
    fn nested_vectors_and_tuple_views() -> Fallible<()> {
        let obj = AnyObject::new(vec![
            AnyObject::new("nested".to_owned()),
            AnyObject::new(vec![1_i32, 2]),
        ]);
        let outer = TestOwner::new(&obj)?;
        let pointers = outer.pointers::<AnyObject>();
        let source = obj.downcast_ref::<Vec<AnyObject>>()?;
        assert_eq!(pointers[0], &source[0] as *const AnyObject);
        let inner = TestOwner::new(unsafe { &*pointers[0] })?;
        assert_eq!(
            unsafe { CStr::from_ptr(inner.pointers::<c_char>()[0]) }.to_bytes(),
            b"nested"
        );
        drop(inner); // Child scratch owner must be freed before the parent/source.
        let inner = TestOwner::new(unsafe { &*pointers[1] })?;
        assert_eq!(
            unsafe { slice::from_raw_parts(inner.view().ptr.cast::<i32>(), 2) },
            &[1, 2]
        );
        drop(inner);

        let tuple = AnyObject::new((AnyObject::new(5_i32), AnyObject::new(6_f64)));
        let owner = TestOwner::new(&tuple)?;
        let pointers = owner.pointers::<AnyObject>();
        assert_eq!(*unsafe { &*pointers[0] }.downcast_ref::<i32>()?, 5);
        assert_eq!(*unsafe { &*pointers[1] }.downcast_ref::<f64>()?, 6.0);

        let tuple = AnyObject::new((7_i32, 8_i32));
        let owner = TestOwner::new(&tuple)?;
        assert_eq!(
            owner
                .pointers::<i32>()
                .iter()
                .map(|&p| unsafe { *p })
                .collect::<Vec<_>>(),
            vec![7, 8]
        );
        let tuple = AnyObject::new((1_u32, 2_f64, 3_f64));
        let owner = TestOwner::new(&tuple)?;
        let pointers = owner.pointers::<c_void>();
        assert_eq!(unsafe { *pointers[0].cast::<u32>() }, 1);
        assert_eq!(unsafe { *pointers[2].cast::<f64>() }, 3.0);

        let empty = AnyObject::new(Vec::<AnyObject>::new());
        assert_eq!(TestOwner::new(&empty)?.view().len, 0);
        Ok(())
    }

    #[test]
    fn option_and_erased_candidate_views() -> Fallible<()> {
        let none = AnyObject::new(None::<(f64, f64)>);
        let owner = TestOwner::new(&none)?;
        assert_eq!(owner.view().len, 0);
        assert!(owner.view().ptr.is_null());
        let none = AnyObject::new(None::<AnyObject>);
        assert_eq!(TestOwner::new(&none)?.view().len, 0);
        let none = AnyObject::new(None::<(f64, ExtrinsicObject)>);
        assert_eq!(TestOwner::new(&none)?.view().len, 0);
        let some = AnyObject::new(Some((2_f64, 3_f64)));
        let owner = TestOwner::new(&some)?;
        assert_eq!(unsafe { *owner.pointers::<f64>()[1] }, 3.0);
        let some = AnyObject::new(Some((2_f64, AnyObject::new(3_i32))));
        let owner = TestOwner::new(&some)?;
        assert_eq!(
            *unsafe { &*owner.pointers::<c_void>()[1].cast::<AnyObject>() }
                .downcast_ref::<i32>()?,
            3
        );
        let some = AnyObject::new(Some(AnyObject::new(9_i32)));
        let owner = TestOwner::new(&some)?;
        assert_eq!(
            *unsafe { &*owner.view().ptr.cast::<AnyObject>() }.downcast_ref::<i32>()?,
            9
        );

        let erased = AnyObject::new_type(
            (0.5_f64, AnyObject::new("candidate".to_owned())),
            Type::of::<(f64, ExtrinsicObject)>(),
        );
        let owner = TestOwner::new(&erased)?;
        let pointers = owner.pointers::<c_void>();
        assert_eq!(unsafe { *pointers[0].cast::<f64>() }, 0.5);
        assert_eq!(
            unsafe { &*pointers[1].cast::<AnyObject>() }.downcast_ref::<String>()?,
            "candidate"
        );
        Ok(())
    }

    #[test]
    fn map_child_objects_and_nested_string_views() -> Fallible<()> {
        for map in [
            HashMap::<String, String>::new(),
            HashMap::from([("k".to_owned(), "v".to_owned())]),
        ] {
            let obj = AnyObject::new(map.clone());
            let owner = TestOwner::new(&obj)?;
            let pointers = owner.pointers::<AnyObject>();
            assert_eq!(pointers.len(), 2);
            let keys = unsafe { &*pointers[0] };
            let values = unsafe { &*pointers[1] };
            let reconstructed = keys
                .downcast_ref::<Vec<String>>()?
                .iter()
                .cloned()
                .zip(values.downcast_ref::<Vec<String>>()?.iter().cloned())
                .collect::<HashMap<_, _>>();
            assert_eq!(reconstructed, map);
            let key_view = TestOwner::new(keys)?;
            let value_view = TestOwner::new(values)?;
            assert_eq!(key_view.view().len, map.len());
            assert_eq!(value_view.view().len, map.len());
            // Reverse declaration order frees nested scratch before the map owner.
        }
        Ok(())
    }

    #[test]
    fn profile_tuple_owns_child_and_borrows_delta() -> Fallible<()> {
        let obj = AnyObject::new((PrivacyProfile::new(|_| Ok(0.25)), 1e-7_f64));
        let owner = TestOwner::new(&obj)?;
        let pointers = owner.pointers::<c_void>();
        let child = unsafe { &*pointers[0].cast::<AnyObject>() };
        assert_eq!(child.downcast_ref::<PrivacyProfile>()?.delta(0.)?, 0.25);
        let (profile, delta) = obj.downcast_ref::<(PrivacyProfile, f64)>()?;
        assert_eq!(pointers[1], (delta as *const f64).cast());
        drop(owner);
        assert_eq!(profile.delta(0.)?, 0.25);
        Ok(())
    }

    #[test]
    fn borrowed_bits_and_unsupported_allocating_types() -> Fallible<()> {
        let bits = BitVector::from_bitslice(BitSlice::from_slice(&[0b10101010_u8]));
        let obj = AnyObject::new(bits);
        let owner = TestOwner::new(&obj)?;
        assert_eq!(owner.view().len, 8);
        assert_eq!(unsafe { *owner.view().ptr.cast::<u8>() }, 0b10101010);
        let func = AnyObject::new(Function::new(|v: &f64| *v));
        assert!(TestOwner::new(&func).is_err());
        let vector = AnyObject::new(vec![(1_f64, 2_f64)]);
        assert!(TestOwner::new(&vector).is_err());
        Ok(())
    }
}
