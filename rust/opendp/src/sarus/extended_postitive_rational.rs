use std::ops::{Add, Sub, Mul, Div, Rem};
use std::cmp::Ordering;
use std::iter::Iterator;
use std::convert::TryFrom;
use std::fmt;
use rug::Rational;
use num::{One, Zero};
use crate::error::*;

#[derive(Clone,Debug)]
pub enum ExtendedPositiveRational {
    Infinity,
    Number(Rational),
}

pub type QPlus = ExtendedPositiveRational;

use ExtendedPositiveRational::*;

impl ExtendedPositiveRational {
    /// Constructor
    pub fn new(numerator:u128, denominator:u128) -> Fallible<ExtendedPositiveRational> {
        match (numerator,denominator) {
            (n,d) => Ok(Number(Rational::from((n, d)))),
            ( 1, 0) => Ok(Infinity),
            _ => fallible!(DomainMismatch),
        }
    }
}

use Ordering::*;

/// Order relation of extended rationals
impl PartialOrd for ExtendedPositiveRational {
    fn partial_cmp(&self, other: &ExtendedPositiveRational) -> Option<Ordering> {
        match (self, other) {
            (Number(r), Number(s)) => r.partial_cmp(s),
            (Infinity, s) => Some(Greater),
            (r, Infinity) => Some(Less),
            (Infinity, Infinity) => Some(Equal),
            _ => None,
        }
    }
}

/// Order relation of extended rationals
impl Ord for ExtendedPositiveRational {
    fn cmp(&self, other: &ExtendedPositiveRational) -> Ordering {
        match (self, other) {
            (Number(r), Number(s)) => r.cmp(s),
            (Infinity, s) => Greater,
            (r, Infinity) => Less,
            (Infinity, Infinity) => Equal,
            _ => Less,
        }
    }
}

/// Equivalence of extended rationals
impl PartialEq for ExtendedPositiveRational {
    fn eq(&self, other: &ExtendedPositiveRational) -> bool {
        match (self, other) {
            (Number(r), Number(s)) => r.eq(s),
            (Infinity, Infinity) => true,
            _ => false,
        }
    }
}

/// Equivalence of extended rationals
impl Eq for ExtendedPositiveRational {}

/// Integer to extended rational
impl From<u128> for ExtendedPositiveRational {
    fn from(integer: u128) -> Self {
        ExtendedPositiveRational::new(integer,1).unwrap_assert("Denominador is positive")
    }
}

impl Zero for ExtendedPositiveRational {
    fn zero() -> ExtendedPositiveRational {0.into()}

    fn is_zero(&self) -> bool {
        match self {
            ExtendedPositiveRational::Number(rational) => *rational == Rational::from(0),
            _ => false,
        }
    }
}

impl One for ExtendedPositiveRational {
    fn one() -> ExtendedPositiveRational {1.into()}
}

/// Sum of extended rationals
impl Add for ExtendedPositiveRational {
    type Output = ExtendedPositiveRational;
    fn add(self, other: ExtendedPositiveRational) -> ExtendedPositiveRational {
        match (self, other) {
            (Number(r), Number(s)) => Number(r+s),
            (Infinity, _) => Infinity,
            (_, Infinity) => Infinity,
        }
    }
}

/// Difference of extended rationals
impl Sub for ExtendedPositiveRational {
    type Output = ExtendedPositiveRational;
    fn sub(self, other: ExtendedPositiveRational) -> ExtendedPositiveRational {
        match (self, other) {
            (Number(r), Number(s)) if r>s => Number(r-s),
            (Infinity, s) => Infinity,
            _ => 0.into(),
        }
    }
}

/// Product of extended rationals
impl Mul for ExtendedPositiveRational {
    type Output = ExtendedPositiveRational;
    fn mul(self, other: ExtendedPositiveRational) -> ExtendedPositiveRational {
        match (self, other) {
            (Number(r), Number(s)) => Number(r*s),
            (Infinity, s) => Infinity,
            (r, Infinity) => Infinity,
        }
    }
}

/// Division of extended rationals
impl Div for ExtendedPositiveRational {
    type Output = ExtendedPositiveRational;
    fn div(self, other: ExtendedPositiveRational) -> ExtendedPositiveRational {
        match (self, other) {
            (r, s) if s==0.into() => Infinity,
            (Number(r), Number(s)) => Number(r/s),
            (Infinity, s) => Infinity,
            (r, Infinity) => Number(0),
        }
    }
}

/// Modulo of extended rationals
impl Rem for ExtendedPositiveRational {
    type Output = ExtendedPositiveRational;
    fn rem(self, other: ExtendedPositiveRational) -> ExtendedPositiveRational {
        ExtendedPositiveRational::from(0)
    }
}

impl fmt::Display for ExtendedPositiveRational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Number(r) => write!(f, "{}", r),
            Infinity => write!(f, "infinity"),
        }
    }
}

/// Simple conversion from a string
impl From<&str> for ExtendedPositiveRational {
    fn from(privacy_loss_string: &str) -> ExtendedPositiveRational {
        let (read, numerator, denominator, decimal) = privacy_loss_string.chars().fold((0,0,0,0,false), |state, char| {
            match (state, char) {
                ((0,0,0,false),'i'|'I') => (1,0,0,false),
                ((1,0,0,false),'n'|'N') => (2,0,0,false),
                ((2,1,0,false),'f'|'F') => (3,1,0,false),
                ((read,num,1,false),'0'..='9') => (read+1, num*10 + u128::from(char.to_digit(10).unwrap_or_default()),1,false),
                ((read,num,1,false),'.') => (read+1,num,1,true),
                ((read,num,denom,true),'0'..='9') => (read+1,num*10 + i128::from(char.to_digit(10).unwrap_or_default()),denom*10,true),
                (state,_) => state,
            }
        });
        ExtendedPositiveRational::new(numerator,denominator).unwrap_or(ExtendedPositiveRational::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_eq() -> Fallible<()> {
        let q = ExtendedPositiveRational::from("0.14957");
        let r = ExtendedPositiveRational::from("1.4957")/10.into();
        assert_eq!(q, r);
        Ok(())
    }

    #[test]
    fn test_from_str_eq_inf() -> Fallible<()> {
        let q = ExtendedPositiveRational::from("5678.90");
        let r = q.clone()*ExtendedPositiveRational::NegativeInfinity;
        assert_eq!(r, ExtendedPositiveRational::NegativeInfinity);
        Ok(())
    }
}