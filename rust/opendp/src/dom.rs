//! Various implementations of Domain.
//!
//! These different versions of [`Domain`] provide a general set of models used throughout OpenDP.
//! Most of the implementations are generic, with the type parameter setting the underlying [`Domain::Carrier`]
//! type.

use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Bound;

use crate::core::Domain;
use crate::error::Fallible;
use crate::traits::{CheckNull, TotalOrd};
use std::fmt::{Debug, Formatter};

/// A Domain that contains all non-null members of the carrier type.
pub struct AllDomain<T> {
    _marker: PhantomData<T>,
}
impl<T> Debug for AllDomain<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "AllDomain()")
    }
}
impl<T> Default for AllDomain<T> {
    fn default() -> Self { Self::new() }
}
impl<T> AllDomain<T> {
    pub fn new() -> Self {
        AllDomain { _marker: PhantomData }
    }
}
// Auto-deriving Clone would put the same trait bound on T, so we implement it manually.
impl<T> Clone for AllDomain<T> {
    fn clone(&self) -> Self { Self::new() }
}
// Auto-deriving PartialEq would put the same trait bound on T, so we implement it manually.
impl<T> PartialEq for AllDomain<T> {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl<T: CheckNull> Domain for AllDomain<T> {
    type Carrier = T;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> { Ok(!val.is_null()) }
}

/// A Domain that carries an underlying Domain in a Box.
#[derive(Clone, PartialEq, Debug)]
pub struct BoxDomain<D: Domain> {
    element_domain: Box<D>
}
impl<D: Domain> BoxDomain<D> {
    pub fn new(element_domain: Box<D>) -> Self {
        BoxDomain { element_domain }
    }
}
impl<D: Domain> Domain for BoxDomain<D> {
    type Carrier = Box<D::Carrier>;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.element_domain.member(val)
    }
}


/// A Domain that unwraps a Data wrapper.
#[derive(Clone, PartialEq, Debug)]
pub struct DataDomain<D: Domain> {
    pub form_domain: D,
}
impl<D: Domain> DataDomain<D> {
    pub fn new(form_domain: D) -> Self {
        DataDomain { form_domain }
    }
}
impl<D: Domain> Domain for DataDomain<D> where
    D::Carrier: 'static + Any {
    type Carrier = Box<dyn Any>;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        let val = val.downcast_ref::<D::Carrier>()
            .ok_or_else(|| err!(FailedCast, "failed to downcast to carrier type"))?;
        self.form_domain.member(val)
    }
}


/// A Domain that contains all the values bounded by an interval.
#[derive(Clone, PartialEq)]
pub struct BoundedDomain<T> {
    lower: Bound<T>,
    upper: Bound<T>,
}
impl<T: TotalOrd> BoundedDomain<T> {
    pub fn new_closed(bounds: (T, T)) -> Fallible<Self> {
        Self::new((Bound::Included(bounds.0), Bound::Included(bounds.1)))
    }
    pub fn new(bounds: (Bound<T>, Bound<T>)) -> Fallible<Self> {
        let (lower, upper) = bounds;
        fn get<T>(value: &Bound<T>) -> Option<&T> {
            match value {
                Bound::Included(value) => Some(value),
                Bound::Excluded(value) => Some(value),
                Bound::Unbounded => None
            }
        }
        if let Some((v_lower, v_upper)) = get(&lower).zip(get(&upper)) {
            if v_lower > v_upper {
                return fallible!(MakeDomain, "lower bound may not be greater than upper bound")
            }
            if v_lower == v_upper {
                match (&lower, &upper) {
                    (Bound::Included(_l), Bound::Excluded(_u)) =>
                        return fallible!(MakeDomain, "upper bound excludes inclusive lower bound"),
                    (Bound::Excluded(_l), Bound::Included(_u)) =>
                        return fallible!(MakeDomain, "lower bound excludes inclusive upper bound"),
                    _ => ()
                }
            }
        }
        Ok(BoundedDomain { lower, upper })
    }
}
impl<T> Debug for BoundedDomain<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "BoundedDomain()")
    }
}
impl<T: Clone + TotalOrd> Domain for BoundedDomain<T> {
    type Carrier = T;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        Ok(match &self.lower {
            Bound::Included(bound) => { val >= bound }
            Bound::Excluded(bound) => { val > bound }
            Bound::Unbounded => { true }
        } && match &self.upper {
            Bound::Included(bound) => { val <= bound }
            Bound::Excluded(bound) => { val < bound }
            Bound::Unbounded => { true }
        })
    }
}


/// A Domain that contains pairs of values.
#[derive(Clone, PartialEq, Debug)]
pub struct PairDomain<D0: Domain, D1: Domain>(pub D0, pub D1);
impl<D0: Domain, D1: Domain> PairDomain<D0, D1> {
    pub fn new(element_domain0: D0, element_domain1: D1) -> Self {
        PairDomain(element_domain0, element_domain1)
    }
}
impl<D0: Domain, D1: Domain> Domain for PairDomain<D0, D1> {
    type Carrier = (D0::Carrier, D1::Carrier);
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        Ok(self.0.member(&val.0)? && self.1.member(&val.1)?)
    }
}


/// A Domain that contains maps of (homogeneous) values.
#[derive(Clone, PartialEq, Debug)]
pub struct MapDomain<DK: Domain, DV: Domain> where DK::Carrier: Eq + Hash {
    pub key_domain: DK,
    pub value_domain: DV
}
impl<DK: Domain, DV: Domain> MapDomain<DK, DV> where DK::Carrier: Eq + Hash {
    pub fn new(key_domain: DK, element_domain: DV) -> Self {
        MapDomain { key_domain, value_domain: element_domain }
    }
}
impl<K: CheckNull, V: CheckNull> MapDomain<AllDomain<K>, AllDomain<V>> where K: Eq + Hash {
    pub fn new_all() -> Self {
        Self::new(AllDomain::<K>::new(), AllDomain::<V>::new())
    }
}
impl<DK: Domain, DV: Domain> Domain for MapDomain<DK, DV> where DK::Carrier: Eq + Hash {
    type Carrier = HashMap<DK::Carrier, DV::Carrier>;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        for (k, v) in val {
            if !self.key_domain.member(k)? || !self.value_domain.member(v)? {
                return Ok(false)
            }
        }
        Ok(true)
    }
}


/// A Domain that contains vectors of (homogeneous) values.
#[derive(Clone, PartialEq)]
pub struct VectorDomain<D: Domain> {
    pub element_domain: D,
}
impl<D: Domain> Debug for VectorDomain<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "VectorDomain({:?})", self.element_domain)
    }
}
impl<D: Domain + Default> Default for VectorDomain<D> {
    fn default() -> Self { Self::new(D::default()) }
}
impl<D: Domain> VectorDomain<D> {
    pub fn new(element_domain: D) -> Self {
        VectorDomain { element_domain }
    }
}
impl<T: CheckNull> VectorDomain<AllDomain<T>> {
    pub fn new_all() -> Self {
        Self::new(AllDomain::<T>::new())
    }
}
impl<D: Domain> Domain for VectorDomain<D> {
    type Carrier = Vec<D::Carrier>;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        for e in val {
            if !self.element_domain.member(e)? {return Ok(false)}
        }
        Ok(true)
    }
}

/// A Domain that specifies the length of the enclosed domain
#[derive(Clone, PartialEq)]
pub struct SizedDomain<D: Domain> {
    pub element_domain: D,
    pub size: usize
}
impl<D: Domain> SizedDomain<D> {
    pub fn new(member_domain: D, size: usize) -> Self {
        SizedDomain { element_domain: member_domain, size }
    }
}
impl<D: Domain> Debug for SizedDomain<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "SizedDomain({:?}, size={})", self.element_domain, self.size)
    }
}
impl<D: Domain> Domain for SizedDomain<D> {
    type Carrier = D::Carrier;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.element_domain.member(val)
    }
}

/// A domain with a built-in representation of nullity, that may take on null values at runtime
#[derive(Clone, PartialEq)]
pub struct InherentNullDomain<D: Domain>
    where D::Carrier: InherentNull {
    pub element_domain: D,
}
impl<D: Domain + Default> Default for InherentNullDomain<D>
    where D::Carrier: InherentNull {
    fn default() -> Self { Self::new(D::default()) }
}
impl<D: Domain> InherentNullDomain<D> where D::Carrier: InherentNull {
    pub fn new(member_domain: D) -> Self {
        InherentNullDomain { element_domain: member_domain }
    }
}
impl<D: Domain> Debug for InherentNullDomain<D> where D::Carrier: InherentNull {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "InherentNullDomain({:?})", self.element_domain)
    }
}
impl<D: Domain> Domain for InherentNullDomain<D> where D::Carrier: InherentNull {
    type Carrier = D::Carrier;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        if val.is_null() {return Ok(true)}
        self.element_domain.member(val)
    }
}
pub trait InherentNull: CheckNull {
    const NULL: Self;
}
macro_rules! impl_inherent_null_float {
    ($($ty:ty),+) => ($(impl InherentNull for $ty {
        const NULL: Self = Self::NAN;
    })+)
}
impl_inherent_null_float!(f64, f32);

/// A domain that represents nullity via the Option type.
/// The value inside is non-null by definition.
/// Transformations should not emit data that can take on null-values at runtime.
/// For example, it is fine to have an OptionDomain<AllDomain<f64>>, but the f64 should never be nan
#[derive(Clone, PartialEq)]
pub struct OptionNullDomain<D: Domain> {
    pub element_domain: D,
}
impl<D: Domain + Default> Default for OptionNullDomain<D> {
    fn default() -> Self { Self::new(D::default()) }
}
impl<D: Domain> OptionNullDomain<D> {
    pub fn new(member_domain: D) -> Self {
        OptionNullDomain { element_domain: member_domain }
    }
}
impl<D: Domain> Debug for OptionNullDomain<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "OptionNullDomain({:?})", self.element_domain)
    }
}
impl<D: Domain> Domain for OptionNullDomain<D> {
    type Carrier = Option<D::Carrier>;
    fn member(&self, value: &Self::Carrier) -> Fallible<bool> {
        value.as_ref()
            .map(|v| self.element_domain.member(v))
            .unwrap_or(Ok(true))
    }
}
