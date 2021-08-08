use std::ops::{Add, Sub, Mul, Div, Rem};
use std::cmp::Ordering;
use std::fmt;
use std::iter::Iterator;
use std::convert::TryFrom;
use rug::Integer;
use num::{Num, One, Zero};

#[derive(Clone,Debug)]
pub struct ExtendedRational {
    numerator: Integer,
    /// This can be 0 to represent + or - infnity
    denominator: Integer,
}

impl ExtendedRational {
    /// Constructor
    pub fn new<N:Into<Integer>,D:Into<Integer>>(numerator:N, denominator:D) -> ExtendedRational {
        ExtendedRational {numerator:numerator.into(), denominator:denominator.into()}
    }

    pub fn positive_infinity() -> ExtendedRational {ExtendedRational::new(1,0)}
    pub fn negative_infinity() -> ExtendedRational {ExtendedRational::new(-1,0)}
    pub fn indeterminate() -> ExtendedRational {ExtendedRational::new(0,0)}
    pub fn is_number(&self) -> bool {self.denominator == 0}
    pub fn is_positive_infinity(&self) -> bool {self.numerator == 1 && self.denominator == 0}
    pub fn is_negative_infinity(&self) -> bool {self.numerator == -1 && self.denominator == 0}
    pub fn is_indeterminate(&self) -> bool {self.numerator == 0 && self.denominator == 0}
    /// Simplify the fraction
    fn simplify(&self) -> ExtendedRational {
        if self.denominator!=0 {
            let gcd = self.numerator.clone().gcd(&self.denominator);
            ExtendedRational::new(
                self.numerator.clone()/&gcd,
                self.denominator.clone()/&gcd
            )
        } else {
            // Does nothing
            self.clone()
        }
    }
}

impl Zero for ExtendedRational {
    fn zero() -> ExtendedRational {
        return ExtendedRational::new(0,1)
    }

    fn is_zero(&self) -> bool {
        return self.numerator == 0 && self.denominator != 0;
    }
}

impl One for ExtendedRational {
    fn one() -> ExtendedRational {
        return ExtendedRational::new(1,1)
    }
}

/// Sum of extended rationals
impl Add for ExtendedRational {
    type Output = ExtendedRational;
    fn add(self, other: ExtendedRational) -> ExtendedRational {
        if self.denominator > 0 {
            if other.denominator > 0 {
                ExtendedRational::new(
                    self.numerator*&other.denominator + &self.denominator*other.numerator,
                    self.denominator * other.denominator,
                ).simplify()
            } else {other}
        } else {
            if other.denominator > 0 {self}
            else if self.numerator == other.denominator {
                self
            } else {ExtendedRational::indeterminate()}
        }
    }
}

/// Difference of extended rationals
impl Sub for ExtendedRational {
    type Output = ExtendedRational;
    fn sub(self, other: ExtendedRational) -> ExtendedRational {
        if self.denominator > 0 {
            if other.denominator > 0 {
                ExtendedRational::new(
                    self.numerator*&other.denominator + &self.denominator*other.numerator,
                    self.denominator * other.denominator,
               ).simplify()
            } else {ExtendedRational::new(-other.numerator, other.denominator)}
        } else {
            if other.denominator > 0 {self}
            else if self.numerator == -other.denominator {
                self
            } else {ExtendedRational::indeterminate()}
        }
    }
}

/// Product of extended rationals
impl Mul for ExtendedRational {
    type Output = ExtendedRational;
    fn mul(self, other: ExtendedRational) -> ExtendedRational {
        if self.denominator > 0 {
            if other.denominator > 0 {
                ExtendedRational::new(
                    self.numerator * other.numerator,
                    self.denominator * other.denominator,
               ).simplify()
            } else {ExtendedRational::new((self.numerator*other.numerator).signum(), 0)}
        } else {ExtendedRational::new((self.numerator*other.numerator).signum(), self.denominator)}
    }
}

/// Division of extended rationals
impl Div for ExtendedRational {
    type Output = ExtendedRational;
    fn div(self, other: ExtendedRational) -> ExtendedRational {
        ExtendedRational::new( 
             self.numerator / other.denominator,
             self.denominator / other.numerator,
        ).simplify()
    }
}

/// Modulo of extended rationals
impl Rem for ExtendedRational {
    type Output = ExtendedRational;
    fn rem(self, other: ExtendedRational) -> ExtendedRational {
        ExtendedRational::new(0, 1).simplify()
    }
}

/// Order relation of extended rationals
impl PartialOrd for ExtendedRational {
    fn partial_cmp(&self, other: &ExtendedRational) -> Option<Ordering> {
        let copy = self.clone();
        (copy.numerator*&other.denominator).partial_cmp(&(copy.denominator*&other.numerator))
    }
}

impl PartialEq for ExtendedRational {
    fn eq(&self, other: &ExtendedRational) -> bool {
        let copy = self.clone();
        return copy.numerator*&other.denominator == copy.denominator*&other.numerator;
    }
}

/// Equivalence of extended rationals
impl Eq for ExtendedRational {}

/// Order relation of extended rationals
impl Ord for ExtendedRational {
    fn cmp(&self, other: &ExtendedRational) -> Ordering {
        let copy = self.clone();
        (copy.numerator*&other.denominator).cmp(&(copy.denominator*&other.numerator))
    }
}

/// Integer to extended rational
impl From<Integer> for ExtendedRational {
    fn from(integer: Integer) -> Self {
        ExtendedRational::new(integer,1)
    }
}

impl fmt::Display for ExtendedRational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.denominator {
            d if *d==0 => match &self.numerator {
                n if *n==-1 => write!(f, "-inf"),
                n if *n==0 => write!(f, "indeterminate"),
                n if *n==1 => write!(f, "+inf"),
                _ => write!(f, "invalid"),
            },
            _ => write!(f, "{}/{}", self.numerator, self.denominator),
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
                ((0,0,0,0,false),'0'..='9') => (1,1,i32::try_from(char.to_digit(10).unwrap_or_default()).unwrap_or_default(),1,false),
                ((read,sign,num,1,false),'0'..='9') => (read+1,sign, num*10 + i32::try_from(char.to_digit(10).unwrap_or_default()).unwrap_or_default(),1,false),
                ((read,sign,num,1,false),'.') => (read+1,sign,num,1,true),
                ((read,sign,num,denom,true),'0'..='9') => (read+1,sign,num*10 + i32::try_from(char.to_digit(10).unwrap_or_default()).unwrap_or_default(),denom*10,true),
                ((2,sign,1,0,false),'n'|'N') => (3,sign,1,0,false),
                ((3,sign,1,0,false),'f'|'F') => (4,sign,1,0,false),
                (state,_) => state,
            }
        });
        ExtendedRational::new(sign*numerator,denominator).simplify()
    }
}