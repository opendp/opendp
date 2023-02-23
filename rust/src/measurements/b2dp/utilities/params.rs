//! Implements the base-2 privacy parameter `eta`, which takes the form
//! `eta = -z * log_2(x/2^y)` for positive integer `x`, `y`, `z` where
//! `x < 2^y`.

use rug::{ops::Pow, Float};

use crate::error::Fallible;

/// Privacy parameter of the form `Eta = -z * log_2(x/2^y)` where
/// `x < 2^y` and `x,y,z > 0`.
#[derive(Debug, Copy, Clone)]
pub struct Eta {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl PartialEq for Eta {
    fn eq(&self, other: &Self) -> bool {
        if self.x != other.x {
            return false;
        }
        if self.y != other.y {
            return false;
        }
        if self.z != other.z {
            return false;
        }
        true
    }
}

// Constructors
impl Eta {
    /// Creates `Eta` privacy parameter from the given `x`, `y` and `z`.
    /// ## Returns
    /// `Result<Eta,&str>` with the created `Eta` on sucess or an error string
    /// on failure.
    /// ## Errors
    /// Returns `Err` if `x`, `y`, or `z` do not meet the requirements.
    pub fn new(x: u32, y: u32, z: u32) -> Fallible<Eta> {
        let eta = Eta { x, y, z };
        eta.check()?;
        Ok(eta)
    }

    /// Creates `Eta` privacy parameter approximating the base `e` parameter
    /// `epsilon` by underestimating with granularity of approximately 0.3.
    /// ## Arguments
    ///   * `epsilon`: the desired base `e` privacy parameter
    /// ## Returns
    /// `Result<Eta,&str>` with the created `Eta` on sucess or an error string
    /// on failure.
    pub fn from_epsilon(epsilon: f64) -> Fallible<Eta> {
        let mut x = 3;
        let mut y = 2;
        let mut z = 1;
        let mut eta = Eta::new(x, y, z).unwrap();
        while eta.get_approximate_epsilon() > epsilon {
            y += 1;
            x = 2u32.pow(y) - 1;
            eta = Eta::new(x, y, z).unwrap();
        }

        let mut eta_closer = Eta::new(x, y, z).unwrap();
        while eta_closer.get_approximate_epsilon() <= epsilon {
            z += 1;
            eta_closer = Eta::new(x, y, z).unwrap();
        }
        if eta_closer.get_approximate_epsilon() > epsilon {
            z -= 1;
        }
        eta = Eta::new(x, y, z).unwrap();
        Ok(eta)
    }
}

// Methods
impl Eta {
    pub fn check(&self) -> Fallible<()> {
        // Check all parameters nonzero
        if self.x == 0 {
            return fallible!(FailedFunction, "x must be nonzero");
        }
        if self.y == 0 {
            return fallible!(FailedFunction, "y must be nonzero");
        }
        if self.z == 0 {
            return fallible!(FailedFunction, "z must be nonzero");
        }

        // Check x < 2^y
        if self.x > 2u32.pow(self.y) - 1 {
            return fallible!(FailedFunction, "x > 2^y - 1");
        }
        Ok(())
    }

    /// Returns the approximate corresponding epsilon
    /// Note that this value may not be identical to `epsilon` parameter used
    /// if constructed with `from_epsilon`.
    pub fn get_approximate_epsilon(&self) -> f64 {
        let base = self.get_base(53).unwrap().to_f64();
        -base.ln()
    }

    /// Get the base `2^(-eta)`
    /// ## Arguments
    ///   * precision: the precision with which to construct the base
    /// ## Returns
    /// Returns an `mpfr::Float` with the requested precision equivalent to
    /// `2^(-eta.z * log_2(eta.x /2^(eta.y)))` or an error.
    /// ## Errors
    /// Returns an error if the `check()` method fails, i.e. not properly initialized.
    pub fn get_base(&self, precision: u32) -> Fallible<Float> {
        self.check()?;
        let v = Float::i_exp(self.x as i32, -(self.y as i32));
        let x_2_pow_neg_y = Float::with_val(precision, v);
        let z = Float::with_val(precision, self.z);
        let base = x_2_pow_neg_y.pow(z);
        return Ok(base);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gmp_mpfr_sys::mpfr;

    #[test]
    fn test_from_epsilon() {
        for &epsilon in [1.0f64, 1.25, 1.5, 3.0, 5.0].iter() {
            let eta_result = Eta::from_epsilon(epsilon);
            assert!(eta_result.is_ok());
            let eta = eta_result.unwrap();
            let approximate_epsilon = eta.get_approximate_epsilon();
            assert!(epsilon - approximate_epsilon < 0.3);
            assert!(epsilon > approximate_epsilon);
        }
    }

    #[test]
    fn test_base_creation_simple() {
        // Construct a privacy parameter
        let eta = Eta::new(1, 1, 1).unwrap();
        // Set a precision
        let precision = 53;

        // unsafe block to access mpfr flags
        unsafe {
            // clear the flags
            mpfr::clear_flags();

            // get the base and confirm the correct value
            let base = eta.get_base(precision).unwrap();
            assert_eq!(base, 0.5);

            // confirm that flags not set
            let flags = mpfr::flags_save();
            assert_eq!(flags, 0);
        }
    }

    #[test]
    fn test_base_creation() {
        // Construct a privacy parameter
        let eta = Eta::new(3, 2, 2).unwrap();
        // Set a precision
        let precision = 53;

        // unsafe block to access mpfr flags
        unsafe {
            // clear the flags
            mpfr::clear_flags();

            // get the base and confirm the correct value
            let base = eta.get_base(precision).unwrap();
            assert_eq!(base, 0.5625);

            // confirm that flags not set
            let flags = mpfr::flags_save();
            assert_eq!(flags, 0);
        }
    }

    /// Tests that zero paramaters result in error
    #[test]
    fn test_zero_x() {
        let bad_x = Eta::new(0, 1, 2);
        assert!(bad_x.is_err());
    }
    #[test]
    fn test_zero_y() {
        let bad_y = Eta::new(1, 0, 2);
        assert!(bad_y.is_err());
    }

    #[test]
    fn test_zero_z() {
        let bad_z = Eta::new(1, 1, 0);
        assert!(bad_z.is_err());
    }

    /// Tests condition on x and y
    #[test]
    fn test_x_and_y_sizes() {
        let bad_x = Eta::new(3, 1, 1);
        assert!(bad_x.is_err());
    }

    /// Successfully creates a good Eta
    #[test]
    fn test_eta_creation() {
        let eta = Eta::new(3, 2, 1);
        assert!(eta.is_ok());
        let eta = eta.unwrap();
        assert_eq!(eta.x, 3);
        assert_eq!(eta.y, 2);
        assert_eq!(eta.z, 1);
    }

    #[test]
    fn test_epsilon_approximation() {
        // Construct a privacy parameter
        let eta = Eta::new(1, 1, 1).unwrap();
        // Compute the approximate epsilon
        let approx_epsilon = eta.get_approximate_epsilon();
        // Check that the values match
        assert!(approx_epsilon - 0.69315 < 0.01);
    }
}
