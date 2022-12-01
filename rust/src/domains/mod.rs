//! Various implementations of Domain.
//!
//! These different versions of [`Domain`] provide a general set of models used throughout OpenDP.
//! Most of the implementations are generic, with the type parameter setting the underlying [`Domain::Carrier`]
//! type.
//! 
//! A data domain is a representation of the set of values on which the function associated with a transformation or measurement can operate.
//! Each metric (see [`crate::metrics`]) is associated with certain data domains. 
//! The [`Domain`] trait is implemented for all domains used in OpenDP.

use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Bound;

use crate::core::Domain;
use crate::error::Fallible;
use crate::interactive::Queryable;
use crate::traits::{CheckNull, TotalOrd, CollectionSize, InherentNull};
use std::fmt::{Debug, Formatter};

#[cfg(feature="contrib")]
mod poly;
#[cfg(feature="contrib")]
pub use poly::*;

pub type QueryableDomain = AllDomain<Queryable>;

// pub struct Hook<'s> {
//     pub inner: Box<dyn Any + 's>,
//     pub listener: Box<dyn Fn(&dyn Any, bool) -> Fallible<bool> + 's>
// }

// impl<'s> Hook<'s> {
//     pub fn new_queryable<Q, A>(state: S, transition: impl Fn(S, &dyn Query<Q>) -> (S, A) + 's) -> Queryable<Q, A> {
//         let listener = |v: &L, b: bool| {
//             Ok(true)
//         };
//         Queryable {
//             state: Some(Hook {
//                 inner: state,
//                 listener: Box::new(listener)
//             }),
//             transition: transition
//         }
//     }
// }


/// # Proof Definition
/// `AllDomain(T)` is the domain of all **non-null** values of type `T`.
/// 
/// # Example
/// ```
/// // Create a domain that includes all values `{0, 1, ..., 2^32 - 1}`.
/// use opendp::domains::AllDomain;
/// let i32_domain = AllDomain::<i32>::new();
/// 
/// // 1 is a member of the i32_domain
/// use opendp::core::Domain;
/// assert!(i32_domain.member(&1)?);
/// 
/// // Create a domain that includes all non-null 32-bit floats.
/// let f32_domain = AllDomain::<f32>::new();
/// 
/// // 1. is a member of the f32_domain
/// assert!(f32_domain.member(&1.)?);
/// // NAN is not a member of the f32_domain
/// assert!(!f32_domain.member(&f32::NAN)?);
/// # opendp::error::Fallible::Ok(())
/// ```
pub struct AllDomain<T> {
    _marker: PhantomData<T>,
}
impl<T> Debug for AllDomain<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "AllDomain({})", type_name!(T))
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

/// # Proof Definition
/// `BoundedDomain(lower, upper, T)` is the domain of all **non-null** values of type `T`
/// between some `lower` bound and `upper` bound.
/// 
/// # Notes
/// The bounds may be inclusive, exclusive, or unbounded (for half-open intervals).
/// For a type `T` to be valid, it must be totally ordered ([`crate::traits::TotalOrd`]).
/// 
/// It is impossible to construct an instance of `BoundedDomain` with inconsistent bounds.
/// The constructors for this struct return an error if `lower > upper`, or if the bounds both exclude and include a value.
/// 
/// # Example
/// ```
/// // Create a domain that includes the values `{1, 2, 3}`.
/// use opendp::domains::BoundedDomain;
/// let i32_bounded_domain = BoundedDomain::<i32>::new_closed((1, 3))?;
/// 
/// // 1 is a member of the i32_domain
/// use opendp::core::Domain;
/// assert!(i32_bounded_domain.member(&1)?);
/// 
/// // 4 is not a member of the i32_domain
/// assert!(!i32_bounded_domain.member(&4)?);
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone, PartialEq)]
pub struct BoundedDomain<T: TotalOrd> {
    lower: Bound<T>,
    upper: Bound<T>,
}
impl<T: TotalOrd> BoundedDomain<T> {
    pub fn new_closed(bounds: (T, T)) -> Fallible<Self> {
        Self::new((Bound::Included(bounds.0), Bound::Included(bounds.1)))
    }
    /// Construct a new BoundedDomain with the given bounds.
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
impl<T: TotalOrd> Debug for BoundedDomain<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "BoundedDomain({})", type_name!(T))
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


/// A Domain that contains maps of (homogeneous) values.
/// 
/// # Proof Definition
/// `MapDomain(key_domain, value_domain, DK, DV)` consists of all hashmaps where 
/// keys are elements of key_domain (of type DK) and
/// values are elements of value_domain (of type DV).
/// 
/// The elements in the DK domain are hashable and have a strict equality operation.
/// 
/// # Example
/// ```
/// use opendp::domains::{MapDomain, AllDomain};
/// // Rust infers the type from the context, at compile-time.
/// // Members of this domain are of type `HashMap<&str, i32>`.
/// let domain = MapDomain::new(AllDomain::new(), AllDomain::new());
/// 
/// use opendp::core::Domain;
/// use std::collections::HashMap;
/// 
/// // create a hashmap we can test with
/// let hashmap = HashMap::from_iter([("a", 23), ("b", 12)]);
/// assert!(domain.member(&hashmap)?);
/// 
/// // Can build up more complicated domains as needed:
/// use opendp::domains::{InherentNullDomain, BoundedDomain};
/// let value_domain = InherentNullDomain::new(BoundedDomain::new_closed((0., 1.))?);
/// let domain = MapDomain::new(AllDomain::new(), value_domain);
/// 
/// // The following is not a member of the hashmap domain, because a value is out-of-range:
/// let hashmap = HashMap::from_iter([("a", 0.), ("b", 2.)]);
/// assert!(!domain.member(&hashmap)?);
/// # opendp::error::Fallible::Ok(())
/// ```
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
/// 
/// # Proof Definition
/// `VectorDomain(inner_domain, D)` is the domain of all vectors of elements drawn from domain `inner_domain`. 
/// 
/// # Example
/// ```
/// use opendp::domains::{VectorDomain, AllDomain, BoundedDomain};
/// use opendp::core::Domain;
/// 
/// // Represents the domain of all vectors.
/// let all_domain = VectorDomain::new(AllDomain::new());
/// assert!(all_domain.member(&vec![1, 2, 3])?);
/// 
/// // Represents the domain of all bounded vectors.
/// let bounded_domain = VectorDomain::new(BoundedDomain::new_closed((-10, 10))?);
/// 
/// // vec![0] is a member, but vec![12] is not, because 12 is out of bounds of the inner domain
/// assert!(bounded_domain.member(&vec![0])?);
/// assert!(!bounded_domain.member(&vec![12])?);
/// 
/// # opendp::error::Fallible::Ok(())
/// ```
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

/// A Domain that specifies the length of the enclosed domain.
/// 
/// # Proof Definition
/// `SizedDomain(inner_domain, size, D)` is the domain of `inner_domain` restricted to only elements with size `size`.
/// 
/// # Example
/// First let `inner_domain` be `VectorDomain::new(AllDomain::<i32>::new())`. 
/// `inner_domain` indicates the set of all i32 vectors.
/// 
/// Then `SizedDomain::new(inner_domain, 3)` indicates the set of all i32 vectors of length 3.
///
/// # Example
/// ```
/// use opendp::domains::{SizedDomain, VectorDomain};
/// // Create a domain that includes all i32 vectors.
/// let vec_domain = VectorDomain::new_all();
/// // Create a domain that includes all i32 vectors of length 3.
/// let sized_domain = SizedDomain::new(vec_domain, 3);
/// 
/// // vec![1, 2, 3] is a member of the sized_domain
/// use opendp::core::Domain;
/// assert!(sized_domain.member(&vec![1i32, 2, 3])?);
/// 
/// // vec![1, 2] is not a member of the sized_domain
/// assert!(!sized_domain.member(&vec![1, 2])?);
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone, PartialEq)]
pub struct SizedDomain<D: Domain> {
    pub inner_domain: D,
    pub size: usize
}
impl<D: Domain> SizedDomain<D> {
    pub fn new(member_domain: D, size: usize) -> Self {
        SizedDomain { inner_domain: member_domain, size }
    }
}
impl<D: Domain> Debug for SizedDomain<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "SizedDomain({:?}, size={})", self.inner_domain, self.size)
    }
}
impl<D: Domain> Domain for SizedDomain<D> where D::Carrier: CollectionSize {
    type Carrier = D::Carrier;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        if val.size() != self.size {
            return Ok(false)
        }
        self.inner_domain.member(val)
    }
}

/// # Proof Definition
/// `InherentNullDomain(element_domain, D)` is the domain of all values of `element_domain` (of type `D`, a domain) 
/// unioned with all null members in `D`. 
/// 
/// The nullity of members in `D` is indicated via the trait [`crate::traits::InherentNull`].
/// 
/// # Notes
/// A domain may have multiple possible null values, 
/// like in the case of floating-point numbers, which have ~`2^MANTISSA_BITS` null values.
/// 
/// Because this domain is defined in terms of a union, 
/// null values need a conceptual definition of equality to uniquely identify them in a set.
/// In order to construct a well-defined set of members in the domain, 
/// we consider null values to have the same identity if their bit representation is equal.
/// 
/// # Example
/// ```
/// use opendp::domains::{InherentNullDomain, AllDomain};
/// let all_domain = AllDomain::new();
/// let null_domain = InherentNullDomain::new(all_domain.clone());
/// 
/// use opendp::core::Domain;
/// // f64 NAN is not a member of all_domain, but is a member of null_domain
/// assert!(!all_domain.member(&f64::NAN)?);
/// assert!(null_domain.member(&f64::NAN)?);
/// 
/// # opendp::error::Fallible::Ok(())
/// ```
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

/// A domain that represents nullity via the Option type.
/// 
/// # Proof Definition
/// `OptionNullDomain(element_domain, D)` is the domain of all values of `element_domain` (of type `D`, a domain) 
/// wrapped in `Some`, as well as `None`.
/// 
/// # Notes
/// This is used to represent nullity for data types like integers or strings, 
/// for which all values they take on are non-null.
/// 
/// # Example
/// ```
/// use opendp::domains::{OptionNullDomain, AllDomain};
/// let null_domain = OptionNullDomain::new(AllDomain::new());
/// 
/// use opendp::core::Domain;
/// assert!(null_domain.member(&Some(1))?);
/// assert!(null_domain.member(&None)?);
/// 
/// # opendp::error::Fallible::Ok(())
/// ```
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

/// retrieves the type_name for a given type
macro_rules! type_name {
    ($ty:ty) => (std::any::type_name::<$ty>().split("::").last().unwrap_or(""))
}
pub(crate) use type_name;

#[cfg(feature = "contrib")]
pub use contrib::*;

#[cfg(feature = "contrib")]
mod contrib {
    use super::*;

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
}
