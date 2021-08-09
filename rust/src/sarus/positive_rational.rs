use std::ops::{Add, Sub, Mul, Div, Rem};
use std::cmp::Ordering;
use std::iter::Iterator;
use std::fmt;
use rug::Rational;
use num::{One, Zero, Num};
use crate::error::*;

#[derive(Clone,Debug)]
pub enum PositiveRational {
    Infinity,
    Number(Rational),
}

pub type QPlus = PositiveRational;

use PositiveRational::*;

impl PositiveRational {
    /// Constructor
    pub fn new(numerator:u128, denominator:u128) -> Fallible<PositiveRational> {
        match (numerator,denominator) {
            ( 1, 0) => Ok(Infinity),
            ( _, 0) => fallible!(DomainMismatch),
            (n,d) => Ok(Number(Rational::from((n, d)))),
        }
    }
}

impl Num for PositiveRational {
    type FromStrRadixErr = Error;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        PositiveRational::new(u128::from_str_radix(str, radix).unwrap_or_default(), 1)
    }
}

use Ordering::*;

/// Order relation of extended rationals
impl PartialOrd for PositiveRational {
    fn partial_cmp(&self, other: &PositiveRational) -> Option<Ordering> {
        match (self, other) {
            (Number(r), Number(s)) => r.partial_cmp(s),
            (Infinity, Infinity) => Some(Equal),
            (Infinity, _) => Some(Greater),
            (_, Infinity) => Some(Less),
            _ => None,
        }
    }
}

/// Order relation of extended rationals
impl Ord for PositiveRational {
    fn cmp(&self, other: &PositiveRational) -> Ordering {
        match (self, other) {
            (Number(r), Number(s)) => r.cmp(s),
            (Infinity, Infinity) => Equal,
            (Infinity, _) => Greater,
            (_, Infinity) => Less,
            _ => Less,
        }
    }
}

/// Equivalence of extended rationals
impl PartialEq for PositiveRational {
    fn eq(&self, other: &PositiveRational) -> bool {
        match (self, other) {
            (Number(r), Number(s)) => r.eq(s),
            (Infinity, Infinity) => true,
            _ => false,
        }
    }
}

/// Equivalence of extended rationals
impl Eq for PositiveRational {}

/// Integer to extended rational
impl From<u128> for PositiveRational {
    fn from(integer: u128) -> Self {
        PositiveRational::new(integer,1).unwrap_assert("Denominador is positive")
    }
}

impl Zero for PositiveRational {
    fn zero() -> PositiveRational {0.into()}

    fn is_zero(&self) -> bool {
        match self {
            PositiveRational::Number(rational) => *rational == Rational::from(0),
            _ => false,
        }
    }
}

impl One for PositiveRational {
    fn one() -> PositiveRational {1.into()}
}

/// Sum of extended rationals
impl Add for PositiveRational {
    type Output = PositiveRational;
    fn add(self, other: PositiveRational) -> PositiveRational {
        match (self, other) {
            (Number(r), Number(s)) => Number(r+s),
            (Infinity, _) => Infinity,
            (_, Infinity) => Infinity,
        }
    }
}

/// Difference of extended rationals
impl Sub for PositiveRational {
    type Output = PositiveRational;
    fn sub(self, other: PositiveRational) -> PositiveRational {
        match (self, other) {
            (Number(r), Number(s)) if r>s => Number(r-s),
            (Infinity, Number(_)) => Infinity,
            _ => 0.into(),
        }
    }
}

/// Product of extended rationals
impl Mul for PositiveRational {
    type Output = PositiveRational;
    fn mul(self, other: PositiveRational) -> PositiveRational {
        match (self, other) {
            (Number(r), Number(s)) => Number(r*s),
            (Infinity, _) => Infinity,
            (_, Infinity) => Infinity,
        }
    }
}

/// Division of extended rationals
impl Div for PositiveRational {
    type Output = PositiveRational;
    fn div(self, other: PositiveRational) -> PositiveRational {
        match (self, other) {
            (_, s) if s==0.into() => Infinity,
            (Number(r), Number(s)) => Number(r/s),
            (Infinity, _) => Infinity,
            (_, Infinity) => Number(0.into()),
        }
    }
}

/// Modulo of extended rationals
impl Rem for PositiveRational {
    type Output = PositiveRational;
    fn rem(self, _: PositiveRational) -> PositiveRational {
        PositiveRational::from(0)
    }
}

impl fmt::Display for PositiveRational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Number(r) => write!(f, "{}", r),
            Infinity => write!(f, "infinity"),
        }
    }
}

/// Simple conversion from a string
impl From<&str> for PositiveRational {
    fn from(privacy_loss_string: &str) -> PositiveRational {
        let (read, numerator, denominator, decimal) = privacy_loss_string.chars().fold((0,0,0,false), |state, char| {
            match (state, char) {
                ((0,0,0,false),'i'|'I') => (1,0,0,false),
                ((1,0,0,false),'n'|'N') => (2,0,0,false),
                ((2,0,0,false),'f'|'F') => (3,1,0,false),
                ((0,0,0,false),'0'..='9') => (1, u128::from(char.to_digit(10).unwrap_or_default()),1,false),
                ((0,0,0,false),'.') => (1,0,1,true),
                ((read,num,1,false),'0'..='9') => (read+1, num*10 + u128::from(char.to_digit(10).unwrap_or_default()),1,false),
                ((read,num,1,false),'.') => (read+1,num,1,true),
                ((read,num,denom,true),'0'..='9') => (read+1,num*10 + u128::from(char.to_digit(10).unwrap_or_default()),denom*10,true),
                (state,_) => state,
            }
        });
        PositiveRational::new(numerator,denominator).unwrap_or(PositiveRational::zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_eq() -> Fallible<()> {
        let q = PositiveRational::from("0.14957");
        let r = PositiveRational::from("1.4957")/10.into();
        assert_eq!(q, r);
        Ok(())
    }

    #[test]
    fn test_from_str_eq_inf() -> Fallible<()> {
        let q = PositiveRational::from("5678.90");
        let r = q.clone()*PositiveRational::Infinity;
        assert_eq!(r, PositiveRational::Infinity);
        Ok(())
    }

    #[test]
    fn test_div_zero_inf() -> Fallible<()> {
        let q = PositiveRational::from("5678.90");
        assert_eq!(q.clone()/0.into(), PositiveRational::Infinity);
        assert_eq!(q.clone()/PositiveRational::Infinity, 0.into());
        Ok(())
    }
}