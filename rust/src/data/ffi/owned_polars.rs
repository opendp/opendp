//! Typed Polars scratch allocations for the existing slice owner.

use std::cell::UnsafeCell;

use crate::domains::{ExprPlan, Invariant, Margin};

use super::*;

// Polars' Arrow schema exporter unwraps CString construction recursively.
// Validate first so a nested field name cannot abort an extern-C conversion.
fn validate_arrow_field(field: &ArrowField) -> Fallible<()> {
    if field.name.as_bytes().contains(&0) {
        return fallible!(FFI, "Arrow field names cannot contain NUL bytes");
    }
    validate_arrow_dtype(field.dtype())
}

fn validate_arrow_dtype(dtype: &ArrowDataType) -> Fallible<()> {
    use ArrowDataType::*;
    match dtype {
        List(field) | LargeList(field) | FixedSizeList(field, _) | Map(field, _) => {
            validate_arrow_field(field)
        }
        Struct(fields) => fields.iter().try_for_each(validate_arrow_field),
        Union(union) => union.fields.iter().try_for_each(validate_arrow_field),
        Dictionary(_, values, _) => validate_arrow_dtype(values),
        Extension(extension) => validate_arrow_dtype(&extension.inner),
        Timestamp(_, Some(zone)) if zone.as_bytes().contains(&0) => {
            fallible!(FFI, "Arrow time zones cannot contain NUL bytes")
        }
        _ => Ok(()),
    }
}

impl FfiSliceOwner {
    // Retain the actual Vec, including its capacity, rather than reconstructing
    // ownership from the exported data pointer and length.
    fn kept_slice<T: 'static>(&mut self, values: Vec<T>) -> FfiSlice {
        let view = FfiSlice::new(values.as_ptr() as *mut c_void, values.len());
        self.keep(values);
        view
    }

    pub(super) fn polars_vector<T: 'static + Clone>(&mut self, values: &[T]) {
        let pointers = values
            .iter()
            .map(|value| self.keep(AnyObject::new(value.clone())))
            .collect();
        self.pointers(pointers);
    }

    fn optional_u32(&mut self, value: Option<u32>) -> *const c_void {
        value.map(|v| self.keep(v).cast()).unwrap_or_else(null)
    }

    fn serialized_expr(&mut self, expr: &Expr) -> Fallible<FfiSlice> {
        let mut bytes = Vec::new();
        polars_utils::pl_serialize::SerializeOptions::default()
            // Match the legacy/Python format (no forward compatibility).
            .serialize_into_writer::<_, _, false>(&mut bytes, expr)
            .map_err(|e| err!(FFI, "failed to serialize Expr: {}", e))?;
        Ok(self.kept_slice(bytes))
    }

    fn serialized_plan(&mut self, plan: &DslPlan) -> Fallible<FfiSlice> {
        let mut bytes = Vec::new();
        plan.serialize_versioned(&mut bytes, Default::default())?;
        Ok(self.kept_slice(bytes))
    }

    /// Create mutable Arrow C headers for a Python producer. All allocations
    /// remain owned by this object, even if export fails or only some input
    /// columns are consumed. The pointer triple has interior mutability because
    /// the Rust importer clears the consumed array slot.
    fn new_arrow_input(name: &str) -> Fallible<Self> {
        let mut owner = Self::new();
        let name = owner.string(name)?;
        let array = UnsafeCell::raw_get(owner.keep(UnsafeCell::new(ArrowArray::empty())));
        let schema = UnsafeCell::raw_get(owner.keep(UnsafeCell::new(ArrowSchema::empty())));
        let pointers = UnsafeCell::raw_get(owner.keep(UnsafeCell::new([
            array.cast::<c_void>().cast_const(),
            schema.cast::<c_void>().cast_const(),
            name.cast::<c_void>(),
        ])));
        owner.view = FfiSlice::new(pointers.cast::<c_void>(), 3);
        Ok(owner)
    }

    fn polars_series(&mut self, series: &Series) -> Fallible<FfiSlice> {
        // Validate before Arrow schema construction (which also needs a CString).
        let name = self.string(series.name().as_str())?;
        let array = series.rechunk().to_arrow(0, CompatLevel::newest());
        let field = ArrowField::new(series.name().clone(), array.dtype().clone(), true);
        validate_arrow_field(&field)?;
        let schema = export_field_to_c(&field);
        let array = export_array_to_c(array);

        // A consumer may release/move the payload and clear the release callback,
        // but must not free these headers. UnsafeCell permits that mutation even
        // while the owner is shared. Dropping the typed boxes frees the headers
        // and releases any payload the consumer did not consume.
        let array = UnsafeCell::raw_get(self.keep(UnsafeCell::new(array)));
        let schema = UnsafeCell::raw_get(self.keep(UnsafeCell::new(schema)));
        Ok(self.kept_slice(vec![
            array.cast::<c_void>().cast_const(),
            schema.cast::<c_void>().cast_const(),
            name.cast::<c_void>(),
        ]))
    }

    pub(super) fn polars_plain(&mut self, obj: &AnyObject) -> Fallible<()> {
        match &obj.type_.contents {
            TypeContents::PLAIN("Expr") => {
                self.view = self.serialized_expr(obj.downcast_ref::<Expr>()?)?;
            }
            TypeContents::PLAIN("LazyFrame") => {
                self.view = self.serialized_plan(&obj.downcast_ref::<LazyFrame>()?.logical_plan)?;
            }
            TypeContents::PLAIN("ExprPlan") => {
                let value = obj.downcast_ref::<ExprPlan>()?;
                let plan = self.serialized_plan(&value.plan)?;
                let expr = self.serialized_expr(&value.expr)?;
                let mut pointers = vec![self.keep(plan), self.keep(expr)];
                if let Some(fill) = &value.fill {
                    let fill = self.serialized_expr(fill)?;
                    pointers.push(self.keep(fill));
                }
                self.pointers(pointers);
            }
            TypeContents::PLAIN("Bound") => {
                let value = obj.downcast_ref::<Bound>()?;
                let by = self.keep(AnyObject::new(value.by.iter().cloned().collect::<Vec<_>>()));
                let per_group = self.optional_u32(value.per_group);
                let num_groups = self.optional_u32(value.num_groups);
                self.pointers(vec![by.cast::<c_void>(), per_group, num_groups]);
            }
            TypeContents::PLAIN("Bounds") => {
                self.polars_vector(&obj.downcast_ref::<Bounds>()?.0);
            }
            TypeContents::PLAIN("Margin") => {
                let value = obj.downcast_ref::<Margin>()?;
                let by = self.keep(AnyObject::new(value.by.iter().cloned().collect::<Vec<_>>()));
                let max_length = self.optional_u32(value.max_length);
                let max_groups = self.optional_u32(value.max_groups);
                let invariant = match value.invariant {
                    Some(Invariant::Keys) => self.string("keys")?,
                    Some(Invariant::Lengths) => self.string("lengths")?,
                    None => null(),
                };
                self.pointers(vec![
                    by.cast::<c_void>(),
                    max_length,
                    max_groups,
                    invariant.cast(),
                ]);
            }
            TypeContents::PLAIN("Series") => {
                self.view = self.polars_series(obj.downcast_ref::<Series>()?)?;
            }
            TypeContents::PLAIN("DataFrame") => {
                let columns = obj
                    .downcast_ref::<DataFrame>()?
                    .get_columns()
                    .iter()
                    .map(|column| self.polars_series(column.as_materialized_series()))
                    .collect::<Fallible<Vec<_>>>()?;
                // Existing output ABI is contiguous FfiSlice headers, not pointers.
                self.view = self.kept_slice(columns);
            }
            _ => return fallible!(FFI, "unsupported object type: {}", obj.type_.to_string()),
        }
        Ok(())
    }
}

/// Allocate a Rust-owned Arrow input view. The caller borrows the headers
/// until slice_owner_free, including on export failure or partial consumption.
#[unsafe(no_mangle)]
pub extern "C" fn opendp_data__new_arrow_array_owned(
    name: *const c_char,
) -> FfiResult<*mut FfiSliceOwner> {
    FfiSliceOwner::new_arrow_input(try_!(util::to_str(name))).into()
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::ffi::CStr;
    use std::rc::Rc;

    use super::*;

    fn pointers<T>(view: &FfiSlice) -> &[*const T] {
        if view.len == 0 {
            return &[];
        }
        unsafe { slice::from_raw_parts(view.ptr.cast(), view.len) }
    }

    fn bytes(view: &FfiSlice) -> &[u8] {
        if view.len == 0 {
            return &[];
        }
        unsafe { slice::from_raw_parts(view.ptr.cast(), view.len) }
    }

    fn read_expr(view: &FfiSlice) -> Fallible<Expr> {
        polars_utils::pl_serialize::SerializeOptions::default()
            .deserialize_from_reader::<Expr, &[u8], false>(bytes(view))
            .map_err(|e| err!(FFI, "failed to deserialize Expr: {}", e))
    }

    #[test]
    fn expr_bound_vectors_and_bounds_are_anyobject_pointer_arrays() -> Fallible<()> {
        for count in [0, 1, 3] {
            let exprs = (0..count).map(|i| col(format!("c{i}"))).collect::<Vec<_>>();
            let obj = AnyObject::new(exprs.clone());
            let owner = FfiSliceOwner::from_object(&obj)?;
            assert_eq!(owner.view.len, count);
            for (&ptr, expected) in pointers::<AnyObject>(&owner.view).iter().zip(&exprs) {
                let child = unsafe { &*ptr };
                assert_eq!(child.downcast_ref::<Expr>()?, expected);
                let nested = FfiSliceOwner::from_object(child)?;
                assert_eq!(read_expr(&nested.view)?, *expected);
                drop(nested);
            }
            drop(owner);
            assert_eq!(obj.downcast_ref::<Vec<Expr>>()?, &exprs);

            let bounds = (0..count)
                .map(|i| Bound::by([col(format!("c{i}"))]).with_per_group(i as u32))
                .collect::<Vec<_>>();
            for obj in [
                AnyObject::new(bounds.clone()),
                AnyObject::new(Bounds(bounds.clone())),
            ] {
                let owner = FfiSliceOwner::from_object(&obj)?;
                assert_eq!(owner.view.len, count);
                for (&ptr, expected) in pointers::<AnyObject>(&owner.view).iter().zip(&bounds) {
                    let child = unsafe { &*ptr };
                    assert_eq!(child.downcast_ref::<Bound>()?, expected);
                    let nested = FfiSliceOwner::from_object(child)?;
                    assert_eq!(nested.view.len, 3);
                    drop(nested);
                }
                drop(owner);
                if obj.type_ == Type::of::<Bounds>() {
                    assert_eq!(&obj.downcast_ref::<Bounds>()?.0, &bounds);
                } else {
                    assert_eq!(obj.downcast_ref::<Vec<Bound>>()?, &bounds);
                }
            }
        }
        Ok(())
    }

    fn check_optional(ptr: *const c_void, expected: Option<u32>) {
        match expected {
            Some(value) => assert_eq!(unsafe { *ptr.cast::<u32>() }, value),
            None => assert!(ptr.is_null()),
        }
    }

    fn check_by(ptr: *const c_void, expected: &std::collections::HashSet<Expr>) -> Fallible<()> {
        let child = unsafe { &*ptr.cast::<AnyObject>() };
        let exprs = child.downcast_ref::<Vec<Expr>>()?;
        assert_eq!(
            &exprs
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<_>>(),
            expected
        );
        let nested = FfiSliceOwner::from_object(child)?;
        assert_eq!(nested.view.len, expected.len());
        drop(nested);
        Ok(())
    }

    #[test]
    fn bound_and_margin_fields_cover_all_optional_combinations() -> Fallible<()> {
        for by in [vec![], vec![col("a"), col("b")]] {
            for first in [None, Some(0), Some(7)] {
                for second in [None, Some(0), Some(11)] {
                    let bound = Bound {
                        by: by.iter().cloned().collect(),
                        per_group: first,
                        num_groups: second,
                    };
                    let obj = AnyObject::new(bound.clone());
                    let owner = FfiSliceOwner::from_object(&obj)?;
                    assert_eq!(owner.view.len, 3);
                    let fields = pointers::<c_void>(&owner.view);
                    check_by(fields[0], &bound.by)?;
                    check_optional(fields[1], first);
                    check_optional(fields[2], second);
                    drop(owner);
                    assert_eq!(obj.downcast_ref::<Bound>()?, &bound);

                    for invariant in [None, Some(Invariant::Keys), Some(Invariant::Lengths)] {
                        let margin = Margin {
                            by: bound.by.clone(),
                            max_length: first,
                            max_groups: second,
                            invariant,
                        };
                        let obj = AnyObject::new(margin.clone());
                        let owner = FfiSliceOwner::from_object(&obj)?;
                        assert_eq!(owner.view.len, 4);
                        let fields = pointers::<c_void>(&owner.view);
                        check_by(fields[0], &margin.by)?;
                        check_optional(fields[1], first);
                        check_optional(fields[2], second);
                        if let Some(invariant) = invariant {
                            let expected = if invariant == Invariant::Keys {
                                "keys"
                            } else {
                                "lengths"
                            };
                            assert_eq!(
                                unsafe { CStr::from_ptr(fields[3].cast()) }
                                    .to_str()
                                    .unwrap(),
                                expected
                            );
                        } else {
                            assert!(fields[3].is_null());
                        }
                        drop(owner);
                        assert_eq!(obj.downcast_ref::<Margin>()?, &margin);
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn serialized_expr_lazyframe_and_exprplan_children() -> Fallible<()> {
        let frame = DataFrame::new(vec![Series::new("a".into(), [1_i32, 2]).into_column()])?;
        let lf = frame.lazy();
        let expr = col("a") + lit(1_i32);
        let obj = AnyObject::new(expr.clone());
        let owner = FfiSliceOwner::from_object(&obj)?;
        assert_eq!(read_expr(&owner.view)?, expr);
        drop(owner);
        assert_eq!(obj.downcast_ref::<Expr>()?, &expr);

        let obj = AnyObject::new(lf.clone());
        let owner = FfiSliceOwner::from_object(&obj)?;
        let plan = DslPlan::deserialize_versioned(bytes(&owner.view))?;
        drop(owner);
        assert_eq!(LazyFrame::from(plan).collect()?.height(), 2);
        assert_eq!(
            obj.downcast_ref::<LazyFrame>()?.clone().collect()?.height(),
            2
        );

        for fill in [None, Some(lit(0_i32))] {
            let obj = AnyObject::new(ExprPlan {
                plan: lf.logical_plan.clone(),
                expr: expr.clone(),
                fill: fill.clone(),
            });
            let owner = FfiSliceOwner::from_object(&obj)?;
            let children = pointers::<FfiSlice>(&owner.view);
            assert_eq!(children.len(), if fill.is_some() { 3 } else { 2 });
            let plan = DslPlan::deserialize_versioned(bytes(unsafe { &*children[0] }))?;
            assert_eq!(read_expr(unsafe { &*children[1] })?, expr);
            if let Some(fill) = fill {
                assert_eq!(read_expr(unsafe { &*children[2] })?, fill);
            }
            drop(owner);
            assert_eq!(LazyFrame::from(plan).collect()?.height(), 2);
            assert_eq!(obj.downcast_ref::<ExprPlan>()?.expr, expr);
        }

        // Verify retention of the actual capacity-bearing allocation, not merely
        // its exposed length. Serializers are allowed to overallocate too.
        let mut owner = FfiSliceOwner::new();
        let mut buffer = Vec::with_capacity(128);
        buffer.extend_from_slice(b"serialized scratch");
        let ptr = buffer.as_ptr();
        let view = owner.kept_slice(buffer);
        assert_eq!(view.ptr, ptr.cast::<c_void>());
        let retained = owner.keepalives[0].downcast_ref::<Vec<u8>>().unwrap();
        assert!(retained.capacity() > retained.len());
        assert_eq!(bytes(&view), b"serialized scratch");
        Ok(())
    }

    // Test-only mirrors of the standard Arrow C ABI let us instrument callbacks
    // without reaching into Polars' private fields. No production layout casts.
    #[repr(C)]
    struct CArray {
        length: i64,
        null_count: i64,
        offset: i64,
        n_buffers: i64,
        n_children: i64,
        buffers: *mut *const c_void,
        children: *mut *mut ArrowArray,
        dictionary: *mut ArrowArray,
        release: Option<unsafe extern "C" fn(*mut ArrowArray)>,
        private_data: *mut c_void,
    }

    #[repr(C)]
    struct CSchema {
        format: *const c_char,
        name: *const c_char,
        metadata: *const c_char,
        flags: i64,
        n_children: i64,
        children: *mut *mut ArrowSchema,
        dictionary: *mut ArrowSchema,
        release: Option<unsafe extern "C" fn(*mut ArrowSchema)>,
        private_data: *mut c_void,
    }

    struct ReleaseState<T> {
        original: unsafe extern "C" fn(*mut T),
        private_data: *mut c_void,
        calls: Rc<Cell<usize>>,
    }

    unsafe extern "C" fn release_array(ptr: *mut ArrowArray) {
        let header = unsafe { &mut *ptr.cast::<CArray>() };
        let state =
            unsafe { Box::from_raw(header.private_data.cast::<ReleaseState<ArrowArray>>()) };
        header.private_data = state.private_data;
        header.release = Some(state.original);
        state.calls.set(state.calls.get() + 1);
        unsafe { (state.original)(ptr) };
    }

    unsafe extern "C" fn release_schema(ptr: *mut ArrowSchema) {
        let header = unsafe { &mut *ptr.cast::<CSchema>() };
        let state =
            unsafe { Box::from_raw(header.private_data.cast::<ReleaseState<ArrowSchema>>()) };
        header.private_data = state.private_data;
        header.release = Some(state.original);
        state.calls.set(state.calls.get() + 1);
        unsafe { (state.original)(ptr) };
    }

    fn track_releases(view: &FfiSlice) -> (Rc<Cell<usize>>, Rc<Cell<usize>>) {
        assert_eq!(
            std::mem::size_of::<CArray>(),
            std::mem::size_of::<ArrowArray>()
        );
        assert_eq!(
            std::mem::align_of::<CArray>(),
            std::mem::align_of::<ArrowArray>()
        );
        assert_eq!(
            std::mem::size_of::<CSchema>(),
            std::mem::size_of::<ArrowSchema>()
        );
        assert_eq!(
            std::mem::align_of::<CSchema>(),
            std::mem::align_of::<ArrowSchema>()
        );
        assert_eq!(view.len, 3);
        let triple = pointers::<c_void>(view);
        let arrays = Rc::new(Cell::new(0));
        let schemas = Rc::new(Cell::new(0));
        unsafe {
            let array = &mut *triple[0].cast_mut().cast::<CArray>();
            let state = ReleaseState {
                original: array.release.unwrap(),
                private_data: array.private_data,
                calls: arrays.clone(),
            };
            array.private_data = Box::into_raw(Box::new(state)).cast();
            array.release = Some(release_array);
            let schema = &mut *triple[1].cast_mut().cast::<CSchema>();
            let state = ReleaseState {
                original: schema.release.unwrap(),
                private_data: schema.private_data,
                calls: schemas.clone(),
            };
            schema.private_data = Box::into_raw(Box::new(state)).cast();
            schema.release = Some(release_schema);
        }
        (arrays, schemas)
    }

    fn consume_series(view: &FfiSlice) -> Fallible<Series> {
        let triple = pointers::<c_void>(view);
        // Model an Arrow C importer moving both payloads and marking the source
        // headers released. It must NOT take ownership of the header boxes.
        let (array, schema) = unsafe {
            (
                std::ptr::replace(
                    triple[0].cast_mut().cast::<ArrowArray>(),
                    ArrowArray::empty(),
                ),
                std::ptr::replace(
                    triple[1].cast_mut().cast::<ArrowSchema>(),
                    ArrowSchema::empty(),
                ),
            )
        };
        let field = unsafe { import_field_from_c(&schema) }?;
        let array = unsafe { import_array_from_c(array, field.dtype) }?;
        let name = unsafe { CStr::from_ptr(triple[2].cast()) }
            .to_str()
            .unwrap();
        Ok(Series::try_from((PlSmallStr::from_str(name), array))?)
    }

    #[test]
    fn arrow_input_owner_cleans_empty_headers_and_invalid_names() -> Fallible<()> {
        for name in ["", "column λ", "long name"] {
            let name = CString::new(name).unwrap();
            let owner: FfiSliceOwner =
                Fallible::from(opendp_data__new_arrow_array_owned(name.as_ptr()))?;
            let pointers = pointers::<c_void>(&owner.view);
            assert_eq!(pointers.len(), 3);
            assert_eq!(
                unsafe { CStr::from_ptr(pointers[2].cast()) }
                    .to_str()
                    .unwrap(),
                name.to_str().unwrap()
            );
            assert!(
                owner
                    .keepalives
                    .iter()
                    .any(|v| v.is::<UnsafeCell<ArrowArray>>())
            );
            assert!(
                owner
                    .keepalives
                    .iter()
                    .any(|v| v.is::<UnsafeCell<ArrowSchema>>())
            );
            drop(owner);
        }
        let invalid = CString::new(vec![0xff]).unwrap();
        let result: Fallible<FfiSliceOwner> =
            opendp_data__new_arrow_array_owned(invalid.as_ptr()).into();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn arrow_headers_release_once_when_abandoned_or_consumed() -> Fallible<()> {
        for name in ["", "λ", "a long column name"] {
            for consumed in [false, true] {
                let mut series = Series::new(name.into(), [Some(1_i32), None]);
                series.append(&Series::new(name.into(), [Some(3_i32)]))?;
                let obj = AnyObject::new(series);
                let owner = FfiSliceOwner::from_object(&obj)?;
                assert!(
                    owner
                        .keepalives
                        .iter()
                        .any(|v| v.is::<UnsafeCell<ArrowArray>>())
                );
                assert!(
                    owner
                        .keepalives
                        .iter()
                        .any(|v| v.is::<UnsafeCell<ArrowSchema>>())
                );
                let (arrays, schemas) = track_releases(&owner.view);
                let imported = if consumed {
                    Some(consume_series(&owner.view)?)
                } else {
                    None
                };
                drop(owner); // Boxes are freed even if their payloads were moved.
                assert_eq!(schemas.get(), 1);
                if let Some(imported) = imported {
                    assert_eq!(arrays.get(), 0);
                    assert_eq!(imported.name().as_str(), name);
                    assert_eq!(
                        imported.i32()?.into_iter().collect::<Vec<_>>(),
                        vec![Some(1), None, Some(3)]
                    );
                    drop(imported);
                }
                assert_eq!(arrays.get(), 1);
                assert_eq!(obj.downcast_ref::<Series>()?.len(), 3);
            }
        }
        let obj = AnyObject::new(Series::new("empty".into(), Vec::<i32>::new()));
        let owner = FfiSliceOwner::from_object(&obj)?;
        assert_eq!(consume_series(&owner.view)?.len(), 0);
        Ok(())
    }

    #[test]
    fn arrow_release_in_place_leaves_owner_to_free_header_boxes() -> Fallible<()> {
        let obj = AnyObject::new(Series::new("released".into(), [1_i32]));
        let owner = FfiSliceOwner::from_object(&obj)?;
        let (arrays, schemas) = track_releases(&owner.view);
        let triple = pointers::<c_void>(&owner.view);
        unsafe {
            let array = triple[0].cast_mut().cast::<ArrowArray>();
            let schema = triple[1].cast_mut().cast::<ArrowSchema>();
            ((*array.cast::<CArray>()).release.unwrap())(array);
            ((*schema.cast::<CSchema>()).release.unwrap())(schema);
            assert!((*array.cast::<CArray>()).release.is_none());
            assert!((*schema.cast::<CSchema>()).release.is_none());
        }
        assert_eq!((arrays.get(), schemas.get()), (1, 1));
        drop(owner);
        assert_eq!((arrays.get(), schemas.get()), (1, 1));
        assert_eq!(obj.downcast_ref::<Series>()?.i32()?.get(0), Some(1));
        Ok(())
    }

    #[test]
    fn dataframe_columns_are_contiguous_headers_and_partial_consumption_is_safe() -> Fallible<()> {
        let empty = AnyObject::new(DataFrame::empty());
        assert_eq!(FfiSliceOwner::from_object(&empty)?.view.len, 0);
        let obj = AnyObject::new(DataFrame::new(vec![
            Series::new("number".into(), [Some(1_i32), None]).into_column(),
            Series::new("text".into(), [Some("λ"), None]).into_column(),
        ])?);
        let owner = FfiSliceOwner::from_object(&obj)?;
        assert_eq!(owner.view.len, 2);
        let columns =
            unsafe { slice::from_raw_parts(owner.view.ptr.cast::<FfiSlice>(), owner.view.len) };
        let first = track_releases(&columns[0]);
        let second = track_releases(&columns[1]);
        let imported = consume_series(&columns[0])?;
        drop(owner); // One consumed column, one abandoned column.
        assert_eq!((first.0.get(), first.1.get()), (0, 1));
        assert_eq!((second.0.get(), second.1.get()), (1, 1));
        drop(obj);
        assert_eq!(imported.i32()?.get(0), Some(1));
        drop(imported);
        assert_eq!((first.0.get(), first.1.get()), (1, 1));
        Ok(())
    }

    #[test]
    fn nul_column_names_fail_and_clean_up_prior_scratch() -> Fallible<()> {
        for _ in 0..16 {
            let bad = Series::new("a\0b".into(), [1_i32]);
            let obj = AnyObject::new(bad.clone());
            assert!(FfiSliceOwner::from_object(&obj).is_err());
            assert_eq!(obj.downcast_ref::<Series>()?.len(), 1);
            let good = Series::new("good".into(), [2_i32]);
            let obj = AnyObject::new(DataFrame::new(vec![
                good.clone().into_column(),
                bad.clone().into_column(),
            ])?);
            assert!(FfiSliceOwner::from_object(&obj).is_err());
            assert_eq!(obj.downcast_ref::<DataFrame>()?.width(), 2);

            let mut owner = FfiSliceOwner::new();
            let view = owner.polars_series(&good)?;
            let (arrays, schemas) = track_releases(&view);
            assert!(owner.polars_series(&bad).is_err());
            drop(owner);
            assert_eq!((arrays.get(), schemas.get()), (1, 1));

            let child = Series::new("bad\0field".into(), [1_i32]);
            let nested =
                StructChunked::from_series("outer".into(), 1, [&child].into_iter())?.into_series();
            let obj = AnyObject::new(nested.clone());
            assert!(FfiSliceOwner::from_object(&obj).is_err());
            let obj = AnyObject::new(DataFrame::new(vec![
                good.into_column(),
                nested.into_column(),
            ])?);
            assert!(FfiSliceOwner::from_object(&obj).is_err());
        }
        Ok(())
    }
}
