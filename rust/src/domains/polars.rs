use std::any::Any;
use std::fmt::Debug;
use std::rc::Rc;

use crate::core::Domain;
use crate::error::Fallible;

use polars::{export::ahash::HashSet, prelude::*};

use crate::domains::Bounds;

#[derive(PartialEq, Clone)]
pub struct LazyFrameDomain {
    pub series: Vec<SeriesDomain>,
    pub counts: Vec<(Vec<String>, Vec<usize>)>,
    pub user_id: Option<String>,
}
impl Domain for LazyFrameDomain {
    type Carrier = LazyFrame;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        let val = val.collect()?;

        if val.schema().len() != self.series.len() {
            return Ok(false);
        }
        if val.get_columns().iter().any(|s| {
            self.column(s.name())
                .map(|dom| dom.member(s).unwrap_or(false))
                .unwrap_or(false)
        }) {
            return Ok(false);
        }

        let count_doesnt_match = self
            .counts
            .iter()
            .find(|(col_names, counts)| unimplemented!())
            .is_some();

        if count_doesnt_match {
            return Ok(false);
        }

        // check that user id column exists
        if let Some(ref user_id) = self.user_id {
            if val.column(user_id.as_str()).is_err() {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl LazyFrameDomain {
    pub fn new(series: Vec<SeriesDomain>) -> Fallible<Self> {
        if HashSet::from_iter(series.iter().map(|s| s.field.name.clone())).len() != series.len() {
            return fallible!(MakeDomain, "duplicate column names");
        }
        Ok(LazyFrameDomain {
            series,
            counts: Vec::new(),
            user_id: None,
        })
    }
    pub fn with_counts(mut self, col_names: Vec<String>, counts: Vec<usize>) -> Fallible<Self> {
        if !self
            .counts
            .iter()
            .any(|(dom_col_names, _)| dom_col_names == &col_names)
        {
            return fallible!(MakeDomain, "counts already specified for these columns");
        }

        let col_cats = col_names
            .iter()
            .map(|col_name| {
                let series_domain = self
                    .series
                    .iter()
                    .find(|s| s.field.name.as_str() == col_name.as_str())
                    .ok_or_else(|| err!(MakeDomain, "column {} not in series", col_name))?;
                match series_domain.bounds {
                    SeriesBounds::Categorical(categories) => Ok(categories.len()),
                    _ => fallible!(MakeDomain, "column {} is not categorical", col_name),
                }
            })
            .collect::<Fallible<Vec<usize>>>()?;

        if col_cats.iter().product::<usize>() != counts.len() - 1 {
            return fallible!(
                MakeDomain,
                "counts length does not match product of categories. Make sure to include one null category."
            );
        }

        self.counts.push((col_names, counts));
        Ok(self)
    }
    pub fn with_user_id(mut self, user_id: String) -> Fallible<Self> {
        if !self.series.iter().any(|s| s.field.name == user_id) {
            return fallible!(MakeDomain, "user_id column must be in series");
        }
        self.user_id = Some(user_id);
        Ok(self)
    }

    pub fn column(&self, name: &str) -> Option<&SeriesDomain> {
        self.series.iter().find(|s| s.field.name == name)
    }

    pub fn len(&self) -> Option<usize> {
        self.counts
            .iter()
            .find(|(col_names, _)| col_names.len() == 0)
            .map(|(_, counts)| counts[0])
    }
}

impl Debug for LazyFrameDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LazyFrameDomain(num_cols={:?}, ...)", self.series.len())
    }
}

#[derive(Clone)]
enum SeriesBounds {
    Categorical(Series),
    Interval(Rc<dyn Any>),
    None,
}

impl PartialEq for SeriesBounds {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SeriesBounds::Categorical(s1), SeriesBounds::Categorical(s2)) => s1 == s2,
            (SeriesBounds::Interval(b1), SeriesBounds::Interval(b2)) => {
                macro_rules! bounds_eq {
                    ($ty:ty) => {{
                        if let (Some(b1), Some(b2)) = (
                            b1.downcast_ref::<Bounds<$ty>>(),
                            b2.downcast_ref::<Bounds<$ty>>(),
                        ) {
                            return b1 == b2;
                        }
                    }};
                }
                bounds_eq!(u8);
                bounds_eq!(u16);
                bounds_eq!(u32);
                bounds_eq!(u64);
                bounds_eq!(i8);
                bounds_eq!(i16);
                bounds_eq!(i32);
                bounds_eq!(i64);
                bounds_eq!(f32);
                bounds_eq!(f64);
                false
            }
            (SeriesBounds::None, SeriesBounds::None) => true,
            _ => false,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct SeriesDomain {
    pub field: Field,
    bounds: SeriesBounds,
    pub nullable: bool,
}

impl SeriesDomain {
    pub fn new<T: DataTypeFrom>(name: &str) -> Self {
        SeriesDomain {
            field: Field::new(name, T::dtype()),
            bounds: SeriesBounds::None,
            nullable: false,
        }
    }
    pub fn new_bounded<T: DataTypeFrom>(name: &str, bounds: Bounds<T>) -> Self {
        SeriesDomain {
            field: Field::new(name, T::dtype()),
            bounds: SeriesBounds::Interval(Rc::new(bounds)),
            nullable: false,
        }
    }
    pub fn new_categorical<T: DataTypeFrom>(name: &str, categories: Series) -> Fallible<Self> {
        if categories.dtype() != &T::dtype() {
            return fallible!(
                MakeDomain,
                "SeriesDomain categories must be of the same type as the values"
            );
        }
        if categories.n_unique()? != categories.len() {
            return fallible!(MakeDomain, "SeriesDomain categories must be unique");
        }
        Ok(SeriesDomain {
            field: Field::new(name, T::dtype()),
            bounds: SeriesBounds::Categorical(categories),
            nullable: false,
        })
    }
    pub fn new_nullable<T: DataTypeFrom>(name: &str) -> Self {
        SeriesDomain {
            field: Field::new(name, T::dtype()),
            bounds: SeriesBounds::None,
            nullable: true,
        }
    }
    pub fn categories(self) -> Fallible<Series> {
        match self.bounds {
            SeriesBounds::Categorical(categories) => Ok(categories),
            _ => fallible!(MakeDomain, "SeriesDomain does not have categorical bounds"),
        }
    }
}

trait DataTypeFrom {
    fn dtype() -> DataType;
}

macro_rules! impl_dtype_from {
    ($ty:ty, $dt:expr) => {
        impl DataTypeFrom for $ty {
            fn dtype() -> DataType {
                $dt
            }
        }
    };
}
impl_dtype_from!(bool, DataType::Boolean);
impl_dtype_from!(u8, DataType::UInt8);
impl_dtype_from!(u16, DataType::UInt16);
impl_dtype_from!(u32, DataType::UInt32);
impl_dtype_from!(u64, DataType::UInt64);
impl_dtype_from!(i8, DataType::Int8);
impl_dtype_from!(i16, DataType::Int16);
impl_dtype_from!(i32, DataType::Int32);
impl_dtype_from!(i64, DataType::Int64);
impl_dtype_from!(f32, DataType::Float32);
impl_dtype_from!(f64, DataType::Float64);
impl_dtype_from!(String, DataType::Utf8);
impl_dtype_from!(str, DataType::Utf8);

impl Domain for SeriesDomain {
    type Carrier = Series;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        if self.field.name != val.name() {
            return Ok(false);
        }
        if &self.field.dtype != val.dtype() {
            return Ok(false);
        }
        if !self.nullable && val.null_count() > 0 {
            return Ok(false);
        }
        if let SeriesBounds::Categorical(categories) = &self.bounds {
            if !val.0.is_in(categories)?.all() {
                return Ok(false);
            }
        }
        if let SeriesBounds::Interval(bounds) = self.bounds {
            macro_rules! bounds_member {
                ($ty:ty) => {{
                    let bounds = bounds
                        .downcast_ref::<Bounds<$ty>>()
                        .ok_or_else(|| err!(FailedCast, "bounds downcast failed"))?;
                    let Some(min) = val.min::<$ty>() else {
                                        return Ok(false);
                                    };
                    let Some(max) = val.max::<$ty>() else {
                                        return Ok(false);
                                    };
                    if bounds.member(&min)? || bounds.member(&max)? {
                        return Ok(false);
                    }
                }};
            }
            match self.field.dtype {
                DataType::UInt8 => bounds_member!(u8),
                DataType::UInt16 => bounds_member!(u16),
                DataType::UInt32 => bounds_member!(u32),
                DataType::UInt64 => bounds_member!(u64),
                DataType::Int8 => bounds_member!(i8),
                DataType::Int16 => bounds_member!(i16),
                DataType::Int32 => bounds_member!(i32),
                DataType::Int64 => bounds_member!(i64),
                DataType::Float32 => bounds_member!(f32),
                DataType::Float64 => bounds_member!(f64),
                _ => {
                    return fallible!(
                        NotImplemented,
                        "Bounds checks are not implemented for {:?}",
                        self.field.dtype
                    )
                }
            }
        }
        Ok(true)
    }
}

impl Debug for SeriesDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field.name, self.field.dtype)
    }
}
