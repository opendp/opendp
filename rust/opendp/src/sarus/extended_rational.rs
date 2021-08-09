use std::ops::{Add, Sub, Mul, Div, Rem};
use std::cmp::Ordering;
use std::iter::Iterator;
use std::convert::TryFrom;
use std::fmt;
use rug::Rational;
use num::{One, Zero, Num};
use crate::error::*;

#[derive(Clone,Debug)]
pub enum ExtendedRational {
    NegativeInfinity,
    PositiveInfinity,
    Number(Rational),
}

pub type QBar = ExtendedRational;

use ExtendedRational::*;

impl ExtendedRational {
    /// Constructor
    pub fn new(numerator:i128, denominator:u128) -> Fallible<ExtendedRational> {
        match (numerator,denominator) {
            (n,d) if d>0 => Ok(Number(Rational::from((n, d)))),
            (-1, 0) => Ok(NegativeInfinity),
            ( 1, 0) => Ok(PositiveInfinity),
            _ => fallible!(DomainMismatch),
        }
    }
}

impl Num for ExtendedRational {
    type FromStrRadixErr = Error;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        ExtendedRational::new(i128::from_str_radix(str, radix).unwrap_or_default(), 1)
    }
}

use Ordering::*;

/// Order relation of extended rationals
impl PartialOrd for ExtendedRational {
    fn partial_cmp(&self, other: &ExtendedRational) -> Option<Ordering> {
        match (self, other) {
            (Number(r), Number(s)) => r.partial_cmp(s),
            (NegativeInfinity, NegativeInfinity) => Some(Equal),
            (NegativeInfinity, PositiveInfinity) => Some(Less),
            (PositiveInfinity, NegativeInfinity) => Some(Greater),
            (PositiveInfinity, PositiveInfinity) => Some(Equal),
            (NegativeInfinity, _) => Some(Less),
            (PositiveInfinity, _) => Some(Greater),
            (_, NegativeInfinity) => Some(Greater),
            (_, PositiveInfinity) => Some(Less),
        }
    }
}

/// Order relation of extended rationals
impl Ord for ExtendedRational {
    fn cmp(&self, other: &ExtendedRational) -> Ordering {
        match (self, other) {
            (Number(r), Number(s)) => r.cmp(s),
            (NegativeInfinity, NegativeInfinity) => Equal,
            (NegativeInfinity, PositiveInfinity) => Less,
            (PositiveInfinity, NegativeInfinity) => Greater,
            (PositiveInfinity, PositiveInfinity) => Equal,
            (NegativeInfinity, _) => Less,
            (PositiveInfinity, _) => Greater,
            (_, NegativeInfinity) => Greater,
            (_, PositiveInfinity) => Less,
        }
    }
}

/// Equivalence of extended rationals
impl PartialEq for ExtendedRational {
    fn eq(&self, other: &ExtendedRational) -> bool {
        match (self, other) {
            (Number(r), Number(s)) => r.eq(s),
            (NegativeInfinity, NegativeInfinity) => true,
            (PositiveInfinity, PositiveInfinity) => true,
            _ => false,
        }
    }
}

/// Equivalence of extended rationals
impl Eq for ExtendedRational {}

/// Integer to extended rational
impl From<i128> for ExtendedRational {
    fn from(integer: i128) -> Self {
        ExtendedRational::new(integer,1).unwrap_assert("Denominador is positive")
    }
}

impl Zero for ExtendedRational {
    fn zero() -> ExtendedRational {0.into()}

    fn is_zero(&self) -> bool {
        match self {
            ExtendedRational::Number(rational) => *rational == Rational::from(0),
            _ => false,
        }
    }
}

impl One for ExtendedRational {
    fn one() -> ExtendedRational {1.into()}
}

/// Sum of extended rationals
impl Add for ExtendedRational {
    type Output = ExtendedRational;
    fn add(self, other: ExtendedRational) -> ExtendedRational {
        match (self, other) {
            (Number(r), Number(s)) => Number(r+s),
            // Left is stronger than right in case of indeterminate
            (NegativeInfinity, _) => NegativeInfinity,
            (PositiveInfinity, _) => PositiveInfinity,
            (_, NegativeInfinity) => NegativeInfinity,
            (_, PositiveInfinity) => PositiveInfinity,
        }
    }
}

/// Difference of extended rationals
impl Sub for ExtendedRational {
    type Output = ExtendedRational;
    fn sub(self, other: ExtendedRational) -> ExtendedRational {
        match (self, other) {
            (Number(r), Number(s)) => Number(r-s),
            (NegativeInfinity, _) => NegativeInfinity,
            (PositiveInfinity, _) => PositiveInfinity,
            (_, NegativeInfinity) => PositiveInfinity,
            (_, PositiveInfinity) => NegativeInfinity,
        }
    }
}

/// Product of extended rationals
impl Mul for ExtendedRational {
    type Output = ExtendedRational;
    fn mul(self, other: ExtendedRational) -> ExtendedRational {
        match (self, other) {
            (Number(r), Number(s)) => Number(r*s),
            (NegativeInfinity, s) if s>0.into() => NegativeInfinity,
            (PositiveInfinity, s) if s>0.into() => PositiveInfinity,
            (NegativeInfinity, s) if s<0.into() => PositiveInfinity,
            (PositiveInfinity, s) if s<0.into() => NegativeInfinity,
            (r, NegativeInfinity) if r>0.into() => NegativeInfinity,
            (r, PositiveInfinity) if r>0.into() => PositiveInfinity,
            (r, NegativeInfinity) if r<0.into() => PositiveInfinity,
            (r, PositiveInfinity) if r<0.into() => NegativeInfinity,
            (s,_) => s,
        }
    }
}

/// Division of extended rationals
impl Div for ExtendedRational {
    type Output = ExtendedRational;
    fn div(self, other: ExtendedRational) -> ExtendedRational {
        match (self, other) {
            (r, s) if r > 0.into() && s==0.into() => PositiveInfinity,
            (r, s) if r < 0.into() && s==0.into() => NegativeInfinity,
            (Number(r), Number(s)) if s != 0 => Number(r/s),
            (NegativeInfinity, s) if s>0.into() => NegativeInfinity,
            (PositiveInfinity, s) if s>0.into() => PositiveInfinity,
            (NegativeInfinity, s) if s<0.into() => PositiveInfinity,
            (PositiveInfinity, s) if s<0.into() => NegativeInfinity,
            (_, NegativeInfinity) => Number(0.into()),
            (_, PositiveInfinity) => Number(0.into()),
            (s,_) => s,
        }
    }
}

/// Modulo of extended rationals
impl Rem for ExtendedRational {
    type Output = ExtendedRational;
    fn rem(self, _: ExtendedRational) -> ExtendedRational {
        ExtendedRational::from(0)
    }
}

impl fmt::Display for ExtendedRational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Number(r) => write!(f, "{}", r),
            NegativeInfinity => write!(f, "-inf"),
            PositiveInfinity => write!(f, "+inf"),
        }
    }
}

/// Simple conversion from a string
impl From<&str> for ExtendedRational {
    fn from(privacy_loss_string: &str) -> ExtendedRational {
        let (read, sign, numerator, denominator, decimal) = privacy_loss_string.chars().fold((0,0,0,0,false), |state, char| {
            match (state, char) {
                ((0,0,0,0,false),'-') => (1,-1,0,0,false),
                ((0,0,0,0,false),'+') => (1, 1,0,0,false),
                ((0,0,0,0,false),'i'|'I') => (2, 1,1,0,false),
                ((1,-1,0,0,false),'i'|'I') => (2,-1,1,0,false),
                ((1, 1,0,0,false),'i'|'I') => (2, 1,1,0,false),
                ((0,0,0,0,false),'0'..='9') => (1,1,i128::try_from(char.to_digit(10).unwrap_or_default()).unwrap_or_default(),1,false),
                ((1,-1,0,0,false),'0'..='9') => (1,-1,i128::try_from(char.to_digit(10).unwrap_or_default()).unwrap_or_default(),1,false),
                ((1, 1,0,0,false),'0'..='9') => (1, 1,i128::try_from(char.to_digit(10).unwrap_or_default()).unwrap_or_default(),1,false),
                ((read,sign,num,1,false),'0'..='9') => (read+1,sign, num*10 + i128::try_from(char.to_digit(10).unwrap_or_default()).unwrap_or_default(),1,false),
                ((read,sign,num,1,false),'.') => (read+1,sign,num,1,true),
                ((read,sign,num,denom,true),'0'..='9') => (read+1,sign,num*10 + i128::try_from(char.to_digit(10).unwrap_or_default()).unwrap_or_default(),denom*10,true),
                ((2,sign,1,0,false),'n'|'N') => (3,sign,1,0,false),
                ((3,sign,1,0,false),'f'|'F') => (4,sign,1,0,false),
                (state,_) => state,
            }
        });
        ExtendedRational::new(sign*numerator,denominator).unwrap_or(ExtendedRational::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_eq() -> Fallible<()> {
        let q = ExtendedRational::from("0.14957");
        let r = ExtendedRational::from("1.4957")/10.into();
        assert_eq!(q, r);
        Ok(())
    }

    #[test]
    fn test_from_str_eq_inf() -> Fallible<()> {
        let q = ExtendedRational::from("5678.90");
        let r = q.clone()*ExtendedRational::NegativeInfinity;
        assert_eq!(r, ExtendedRational::NegativeInfinity);
        Ok(())
    }

    #[test]
    fn test_div_zero_inf() -> Fallible<()> {
        let q = ExtendedRational::from("87678.90");
        let r = ExtendedRational::from("-0.001");
        assert_eq!(q.clone()/0.into(), ExtendedRational::PositiveInfinity);
        assert_eq!(r.clone()/0.into(), ExtendedRational::NegativeInfinity);
        assert_eq!((ExtendedRational::from(0)-q.clone())/ExtendedRational::NegativeInfinity, 0.into());
        Ok(())
    }
}