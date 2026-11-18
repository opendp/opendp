//! Scratch views for Polars expressions, plans, margins, and bounds.

use crate::domains::{ExprPlan, Invariant, Margin};

use super::*;

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
            _ => return fallible!(FFI, "unsupported object type: {}", obj.type_.to_string()),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

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

}
