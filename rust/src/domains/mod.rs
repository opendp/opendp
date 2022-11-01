//! Various implementations of Domain.
//!
//! These different versions of [`Domain`] provide a general set of models used throughout OpenDP.
//! Most of the implementations are generic, with the type parameter setting the underlying [`Domain::Carrier`]
//! type.
//!
//! A data domain is a representation of the set of values on which the function associated with a transformation or measurement can operate.
//! Each metric (see [`crate::metrics`]) is associated with certain data domains.
//! The [`Domain`] trait is implemented for all domains used in OpenDP.

#[cfg(feature = "ffi")]
mod ffi;

// Once we have things using `Any` that are outside of `contrib`, this should specify `feature="ffi"`.
#[cfg(feature = "contrib")]
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Bound;

use crate::core::Domain;
use crate::error::Fallible;
use crate::traits::{CheckAtom, InherentNull, TotalOrd};
use std::fmt::{Debug, Formatter};

#[cfg(feature = "contrib")]
mod poly;
#[cfg(feature = "contrib")]
pub use poly::*;

/// # Proof Definition
/// `AtomDomain(T)` is the domain of all values of an atomic type `T`.
/// If bounds are set, then the domain is restricted to the bounds.
/// If nullable is set, then null value(s) are included in the domain.
///
/// # Notes
/// If nullable is set, a domain may have multiple possible null values,
/// like in the case of floating-point numbers, which have ~`2^MANTISSA_BITS` null values.
///
/// Because domains are defined in terms of a union,
/// null values need a conceptual definition of equality to uniquely identify them in a set.
/// In order to construct a well-defined set of members in the domain,
/// we consider null values to have the same identity if their bit representation is equal.
///
/// # Example
/// ```
/// // Create a domain that includes all values `{0, 1, ..., 2^32 - 1}`.
/// use opendp::domains::AtomDomain;
/// let i32_domain = AtomDomain::<i32>::default();
///
/// // 1 is a member of the i32_domain
/// use opendp::core::Domain;
/// assert!(i32_domain.member(&1)?);
///
/// // Create a domain that includes all non-null 32-bit floats.
/// let f32_domain = AtomDomain::<f32>::default();
///
/// // 1. is a member of the f32_domain
/// assert!(f32_domain.member(&1.)?);
/// // NAN is not a member of the f32_domain
/// assert!(!f32_domain.member(&f32::NAN)?);
/// # opendp::error::Fallible::Ok(())
/// ```
///
/// # Null Example
/// ```
/// use opendp::domains::{Null, AtomDomain};
/// let all_domain = AtomDomain::default();
/// let null_domain = AtomDomain::new_nullable();
///
/// use opendp::core::Domain;
/// // f64 NAN is not a member of all_domain, but is a member of null_domain
/// assert!(!all_domain.member(&f64::NAN)?);
/// assert!(null_domain.member(&f64::NAN)?);
///
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone, PartialEq)]
pub struct AtomDomain<T: CheckAtom> {
    bounds: Option<Bounds<T>>,
    nullable: bool,
}

impl<T: CheckAtom> Debug for AtomDomain<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let bounds = self
            .bounds
            .as_ref()
            .map(|b| format!("bounds={:?}, ", b))
            .unwrap_or_default();
        let nullable = self.nullable.then(|| "nullable=true, ").unwrap_or_default();
        write!(f, "AtomDomain({}{}T={})", bounds, nullable, type_name!(T))
    }
}
impl<T: CheckAtom> Default for AtomDomain<T> {
    fn default() -> Self {
        AtomDomain {
            bounds: None,
            nullable: false,
        }
    }
}
impl<T: CheckAtom> AtomDomain<T> {
    pub fn new(bounds: Option<Bounds<T>>, nullable: Option<Null<T>>) -> Self {
        AtomDomain {
            bounds,
            nullable: nullable.is_some(),
        }
    }
    pub fn has_null(&self) -> bool {
        self.nullable.is_some()
    }
}
impl<T: CheckAtom + InherentNull> AtomDomain<T> {
    pub fn new_nullable() -> Self {
        AtomDomain {
            bounds: None,
            nullable: true,
        }
    }
}
impl<T: CheckAtom + TotalOrd> AtomDomain<T> {
    pub fn new_closed(bounds: (T, T)) -> Fallible<Self> {
        Ok(AtomDomain {
            bounds: Some(Bounds::new_closed(bounds)?),
            nullable: false,
        })
    }
}

impl<T: CheckAtom> Domain for AtomDomain<T> {
    type Carrier = T;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        val.check_member(self.bounds.clone(), self.nullable)
    }
}

/// # Proof Definition
/// `Null(T)` is a marker that can only be constructed by values of `T` that can contain inherent nullity.
///
/// The nullity of members in `T` is indicated via the trait [`crate::traits::InherentNull`].
#[derive(PartialEq)]
pub struct Null<T> {
    pub _marker: PhantomData<T>,
}
impl<T> Clone for Null<T> {
    fn clone(&self) -> Self {
        Self {
            _marker: self._marker.clone(),
        }
    }
}
impl<T: InherentNull> Default for Null<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: InherentNull> Null<T> {
    pub fn new() -> Self {
        Null {
            _marker: PhantomData,
        }
    }
}
impl<T> Debug for Null<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Null({:?})", type_name!(T))
    }
}

/// # Proof Definition
/// `Bounds(lower, upper, T)` is the interval of all **non-null** values of type `T`
/// between some `lower` bound and `upper` bound.
///
/// # Notes
/// The bounds may be inclusive, exclusive, or unbounded (for half-open intervals).
/// For a type `T` to be valid, it must be totally ordered ([`crate::traits::TotalOrd`]).
///
/// It is impossible to construct an instance of `Bounds` with inconsistent bounds.
/// The constructors for this struct return an error if `lower > upper`, or if the bounds both exclude and include a value.
#[derive(Clone, PartialEq)]
pub struct Bounds<T> {
    lower: Bound<T>,
    upper: Bound<T>,
}
impl<T: TotalOrd> Bounds<T> {
    pub fn new_closed(bounds: (T, T)) -> Fallible<Self> {
        Self::new((Bound::Included(bounds.0), Bound::Included(bounds.1)))
    }
    /// Checks that the arguments are well-formed.
    pub fn new(bounds: (Bound<T>, Bound<T>)) -> Fallible<Self> {
        let (lower, upper) = bounds;
        fn get<T>(value: &Bound<T>) -> Option<&T> {
            match value {
                Bound::Included(value) => Some(value),
                Bound::Excluded(value) => Some(value),
                Bound::Unbounded => None,
            }
        }
        if let Some((v_lower, v_upper)) = get(&lower).zip(get(&upper)) {
            if v_lower > v_upper {
                return fallible!(
                    MakeDomain,
                    "lower bound may not be greater than upper bound"
                );
            }
            if v_lower == v_upper {
                match (&lower, &upper) {
                    (Bound::Included(_l), Bound::Excluded(_u)) => {
                        return fallible!(MakeDomain, "upper bound excludes inclusive lower bound")
                    }
                    (Bound::Excluded(_l), Bound::Included(_u)) => {
                        return fallible!(MakeDomain, "lower bound excludes inclusive upper bound")
                    }
                    _ => (),
                }
            }
        }
        Ok(Bounds { lower, upper })
    }
}
impl<T: Debug> Debug for Bounds<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let lower = match &self.lower {
            Bound::Included(v) => format!("[{:?}", v),
            Bound::Excluded(v) => format!("({:?}", v),
            Bound::Unbounded => "(∞".to_string(),
        };
        let upper = match &self.upper {
            Bound::Included(v) => format!("{:?}]", v),
            Bound::Excluded(v) => format!("{:?})", v),
            Bound::Unbounded => "∞)".to_string(),
        };
        write!(f, "{}, {}", lower, upper)
    }
}
impl<T: Clone + TotalOrd> Bounds<T> {
    pub fn member(&self, val: &T) -> Fallible<bool> {
        Ok(match &self.lower {
            Bound::Included(bound) => val.total_ge(bound)?,
            Bound::Excluded(bound) => val.total_gt(bound)?,
            Bound::Unbounded => true,
        } && match &self.upper {
            Bound::Included(bound) => val.total_le(bound)?,
            Bound::Excluded(bound) => val.total_lt(bound)?,
            Bound::Unbounded => true,
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
/// use opendp::domains::{MapDomain, AtomDomain};
/// // Rust infers the type from the context, at compile-time.
/// // Members of this domain are of type `HashMap<&str, i32>`.
/// let domain = MapDomain::new(AtomDomain::default(), AtomDomain::default());
///
/// use opendp::core::Domain;
/// use std::collections::HashMap;
///
/// // create a hashmap we can test with
/// let hashmap = HashMap::from_iter([("a", 23), ("b", 12)]);
/// assert!(domain.member(&hashmap)?);
///
/// // Can build up more complicated domains as needed:
/// let value_domain = AtomDomain::new_closed((0., 1.))?;
/// let domain = MapDomain::new(AtomDomain::default(), value_domain);
///
/// // The following is not a member of the hashmap domain, because a value is out-of-range:
/// let hashmap = HashMap::from_iter([("a", 0.), ("b", 2.)]);
/// assert!(!domain.member(&hashmap)?);
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct MapDomain<DK: Domain, DV: Domain>
where
    DK::Carrier: Eq + Hash,
{
    pub key_domain: DK,
    pub value_domain: DV,
}
impl<DK: Domain, DV: Domain> MapDomain<DK, DV>
where
    DK::Carrier: Eq + Hash,
{
    pub fn new(key_domain: DK, element_domain: DV) -> Self {
        MapDomain {
            key_domain,
            value_domain: element_domain,
        }
    }
}

impl<DK: Domain, DV: Domain> Domain for MapDomain<DK, DV>
where
    DK::Carrier: Eq + Hash,
{
    type Carrier = HashMap<DK::Carrier, DV::Carrier>;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        for (k, v) in val {
            if !self.key_domain.member(k)? || !self.value_domain.member(v)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// A Domain that contains vectors of (homogeneous) values.
///
/// # Proof Definition
/// `VectorDomain(inner_domain, D, Option<size>)` is the domain of all vectors of elements drawn from domain `inner_domain`.
/// If size is specified, then the domain is further restricted to all vectors of the given size.
///
/// # Example
/// ```
/// use opendp::domains::{VectorDomain, AtomDomain};
/// use opendp::core::Domain;
///
/// // Represents the domain of vectors.
/// let vec_domain = VectorDomain::new(AtomDomain::default(), None);
/// assert!(vec_domain.member(&vec![1, 2, 3])?);
///
/// // Represents the domain of all vectors of bounded elements.
/// let bounded_domain = VectorDomain::new(AtomDomain::new_closed((-10, 10))?, None);
///
/// // vec![0] is a member, but vec![12] is not, because 12 is out of bounds of the inner domain
/// assert!(bounded_domain.member(&vec![0])?);
/// assert!(!bounded_domain.member(&vec![12])?);
///
/// # opendp::error::Fallible::Ok(())
/// ```
///
/// # Size Example
/// ```
/// use opendp::domains::{VectorDomain, AtomDomain};
/// // Create a domain that includes all i32 vectors of length 3.
/// let sized_domain = VectorDomain::new(AtomDomain::default(), Some(3));
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
pub struct VectorDomain<D: Domain> {
    pub element_domain: D,
    pub size: Option<usize>,
}
impl<D: Domain> Debug for VectorDomain<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let size_str = self
            .size
            .map(|size| format!(", size={:?}", size))
            .unwrap_or_default();
        write!(f, "VectorDomain({:?}{})", self.element_domain, size_str)
    }
}
impl<D: Domain + Default> Default for VectorDomain<D> {
    fn default() -> Self {
        Self::new(D::default(), None)
    }
}
impl<D: Domain> VectorDomain<D> {
    pub fn new(element_domain: D, size: Option<usize>) -> Self {
        VectorDomain {
            element_domain,
            size,
        }
    }
}
impl<D: Domain> Domain for VectorDomain<D> {
    type Carrier = Vec<D::Carrier>;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        for e in val {
            if !self.element_domain.member(e)? {
                return Ok(false);
            }
        }
        if let Some(size) = self.size {
            if size != val.len() {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// A domain that represents nullity via the Option type.
///
/// # Proof Definition
/// `OptionDomain(element_domain, D)` is the domain of all values of `element_domain` (of type `D`, a domain)
/// wrapped in `Some`, as well as `None`.
///
/// # Notes
/// This is used to represent nullity for data types like integers or strings,
/// for which all values they take on are non-null.
///
/// # Example
/// ```
/// use opendp::domains::{OptionDomain, AtomDomain};
/// let null_domain = OptionDomain::new(AtomDomain::default());
///
/// use opendp::core::Domain;
/// assert!(null_domain.member(&Some(1))?);
/// assert!(null_domain.member(&None)?);
///
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone, PartialEq)]
pub struct OptionDomain<D: Domain> {
    pub element_domain: D,
}
impl<D: Domain + Default> Default for OptionDomain<D> {
    fn default() -> Self {
        Self::new(D::default())
    }
}
impl<D: Domain> OptionDomain<D> {
    pub fn new(member_domain: D) -> Self {
        OptionDomain {
            element_domain: member_domain,
        }
    }
}
impl<D: Domain> Debug for OptionDomain<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "OptionDomain({:?})", self.element_domain)
    }
}
impl<D: Domain> Domain for OptionDomain<D> {
    type Carrier = Option<D::Carrier>;
    fn member(&self, value: &Self::Carrier) -> Fallible<bool> {
        value
            .as_ref()
            .map(|v| self.element_domain.member(v))
            .unwrap_or(Ok(true))
    }
}

/// retrieves the type_name for a given type
macro_rules! type_name {
    ($ty:ty) => {
        std::any::type_name::<$ty>()
            .split("::")
            .last()
            .unwrap_or("")
    };
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
        element_domain: Box<D>,
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
    impl<D: Domain> Domain for DataDomain<D>
    where
        D::Carrier: 'static,
    {
        type Carrier = Box<dyn Any>;
        fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
            let val = val
                .downcast_ref::<D::Carrier>()
                .ok_or_else(|| err!(FailedCast, "failed to downcast to carrier type"))?;
            self.form_domain.member(val)
        }
    }
}
