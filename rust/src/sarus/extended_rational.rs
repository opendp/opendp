use std::ops::{Add, Sub, Mul, Div, Rem};
use std::cmp::Ordering;
use std::error::Error;
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
    fn new<N:Into<Integer>,D:Into<Integer>>(numerator:N, denominator:D) -> ExtendedRational {
        ExtendedRational {numerator:numerator.into(), denominator:denominator.into()}
    }

    fn positive_infinity() -> ExtendedRational {ExtendedRational::new(1,0)}
    fn negative_infinity() -> ExtendedRational {ExtendedRational::new(-1,0)}
    /// Simplify the fraction
    fn simplify(&self) -> ExtendedRational {
        let gcd = self.numerator.gcd(&self.denominator);
        ExtendedRational {
            numerator: self.numerator/gcd,
            denominator: self.denominator/gcd,
        }
    }
}

impl Zero for ExtendedRational {
    fn zero() -> ExtendedRational {
        return ExtendedRational::new(0,1)
    }

    fn is_zero(&self) -> bool {
        return self.numerator == 0;
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
        ExtendedRational {
            numerator: other.denominator*self.numerator + self.denominator*other.numerator,
            denominator: self.denominator * other.denominator,
        }.simplify()
    }
}

/// Difference of extended rationals
impl Sub for ExtendedRational {
    type Output = ExtendedRational;
    fn sub(self, other: ExtendedRational) -> ExtendedRational {
        ExtendedRational {
            numerator: other.denominator*self.numerator - self.denominator*other.numerator,
            denominator: self.denominator * other.denominator,
        }.simplify()
    }
}

/// Product of extended rationals
impl Mul for ExtendedRational {
    type Output = ExtendedRational;
    fn mul(self, other: ExtendedRational) -> ExtendedRational {
        ExtendedRational {
            numerator: self.numerator * other.numerator,
            denominator: self.denominator * other.denominator,
        }.simplify()
    }
}

/// Division of extended rationals
impl Div for ExtendedRational {
    type Output = ExtendedRational;
    fn div(self, other: ExtendedRational) -> ExtendedRational {
        ExtendedRational {
            numerator: self.numerator * other.denominator,
            denominator: self.denominator * other.numerator,
        }.simplify()
    }
}

/// Modulo of extended rationals
impl Rem for ExtendedRational {
    type Output = ExtendedRational;
    fn rem(self, other: ExtendedRational) -> ExtendedRational {
        ExtendedRational {
            numerator: self.numerator * other.denominator,
            denominator: self.denominator * other.numerator,
        }.simplify()
    }
}

/// Order relation of extended rationals
impl PartialOrd for ExtendedRational {
    fn partial_cmp(&self, other: &ExtendedRational) -> Option<Ordering> {
        (self.numerator*other.denominator).partial_cmp(&(self.denominator*other.numerator))
    }
}

impl PartialEq for ExtendedRational {
    fn eq(&self, other: &ExtendedRational) -> bool {
        return self.numerator*other.denominator == self.denominator*other.numerator;
    }
}

/// Equivalence of extended rationals
impl Eq for ExtendedRational {}

/// Order relation of extended rationals
impl Ord for ExtendedRational {
    fn cmp(&self, other: &ExtendedRational) -> Ordering {
        (self.numerator*other.denominator).cmp(&(self.denominator*other.numerator))
    }
}

/// Integer to extended rational
impl From<Integer> for ExtendedRational {
    fn from(integer: Integer) -> Self {
        ExtendedRational::new(integer,1)
    }
}

#[derive(Debug)]
struct ParsingError;
impl fmt::Display for ParsingError {fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Parsing error")
}}
impl Error for ParsingError {}

/// Simple conversion from a string
impl TryFrom<String> for ExtendedRational {
    type Error = Box<dyn Error>;
    fn try_from(privacy_loss_string: String) -> Result<ExtendedRational, Self::Error> {
        let (read, sign, numerator, denominator, decimal, infinite) = privacy_loss_string.chars().try_fold::<_,_,Result<_, Self::Error>>((0,0,0,1,false,false), |state, char| {
            match (state, char) {
                ((0,0,0,1,false,false),'-') => Ok((1,-1,0,1,false,false)),
                ((0,0,0,1,false,false),'+') => Ok((1, 1,0,1,false,false)),
                ((0,0,0,1,false,false),'i'|'I') => Ok((2, 1,1,0,false,false)),
                ((1,-1,0,1,false,false),'i'|'I') => Ok((2,-1,1,0,false,true)),
                ((1, 1,0,1,false,false),'i'|'I') => Ok((2, 1,1,0,false,true)),
                ((read,0,0,1,false,false),'0'..='9') => Ok((read+1,1,0,1,false,true)),
                ((read,sign,num,1,false,false),'0'..='9') => Ok((read+1,sign, num*10 + i32::try_from(char.to_digit(10).ok_or(Box::new(ParsingError{}))?)?,1,false,false)),
                ((read,sign,num,1,false,false),'.') => Ok((read+1,sign,num,1,true,false)),
                ((read,sign,num,denom,true,false),'0'..='9') => Ok((read+1,sign,num*10 + i32::try_from(char.to_digit(10).ok_or(Box::new(ParsingError{}))?)?,denom*10,true,false)),
                ((2,sign,1,0,false,true),'n'|'N') => Ok((3,sign,1,0,false,true)),
                ((3,sign,1,0,false,true),'f'|'F') => Ok((4,sign,1,0,false,true)),
                (_,_) => Err(Box::new(ParsingError{}))
            }
        })?;
        Ok(ExtendedRational::new(sign*numerator,denominator))
    }
}