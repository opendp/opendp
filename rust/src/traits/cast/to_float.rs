use dashu::{
    base::{Approximation, BitTest, FloatEncoding, Sign},
    integer::UBig,
    rational::RBig,
};

fn cmp_ubig_to_scaled(numer: &UBig, denom: &UBig, exp: isize) -> std::cmp::Ordering {
    if exp >= 0 {
        numer.cmp(&(denom << exp as usize))
    } else {
        (numer << (-exp) as usize).cmp(denom)
    }
}

fn floor_log2_ratio(numer: &UBig, denom: &UBig) -> isize {
    let exp = numer.bit_len() as isize - denom.bit_len() as isize;

    if cmp_ubig_to_scaled(numer, denom, exp).is_lt() {
        exp - 1
    } else {
        exp
    }
}

fn with_sign<T>(v: T, sign: Sign, error: Sign) -> Approximation<T, Sign> {
    Approximation::Inexact(
        v,
        if sign == Sign::Positive {
            error
        } else {
            -error
        },
    )
}

fn round_scaled_ratio_to_ubig(numer: &UBig, denom: &UBig, exp: isize) -> Approximation<UBig, Sign> {
    let (scaled_num, scaled_den) = if exp >= 0 {
        (numer.clone(), denom << exp as usize)
    } else {
        (numer << (-exp) as usize, denom.clone())
    };

    let q = &scaled_num / &scaled_den;
    let r = scaled_num - &q * &scaled_den;

    if r.is_zero() {
        return Approximation::Exact(q);
    }

    let twice_r = r << 1;
    if twice_r > scaled_den || (twice_r == scaled_den && (&q & UBig::ONE).is_one()) {
        Approximation::Inexact(q + UBig::ONE, Sign::Positive)
    } else {
        Approximation::Inexact(q, Sign::Negative)
    }
}

macro_rules! impl_rbig_to_float {
    ($approx_fn:ident, $ty:ty, $mantissa:ty, $precision:expr, $emin:expr, $emax:expr) => {
        pub(super) fn $approx_fn(v: RBig) -> Approximation<$ty, Sign> {
            let (signed_numer, denom) = v.into_parts();
            let (sign, numer) = signed_numer.into_parts();

            if numer.is_zero() {
                return Approximation::Exact(0.0);
            }

            let value_exp = floor_log2_ratio(&numer, &denom);
            let min_subnormal_exp = $emin - ($precision - 1);

            if value_exp > $emax {
                let value = if sign == Sign::Negative {
                    <$ty>::NEG_INFINITY
                } else {
                    <$ty>::INFINITY
                };
                return with_sign(value, sign, Sign::Positive);
            }

            if value_exp < min_subnormal_exp - 1 {
                let value = if sign == Sign::Negative { -0.0 } else { 0.0 };
                return with_sign(value, sign, Sign::Negative);
            }

            let mantissa_exp = if value_exp < $emin {
                min_subnormal_exp
            } else {
                value_exp - ($precision - 1)
            };

            let (mantissa, error) = match round_scaled_ratio_to_ubig(&numer, &denom, mantissa_exp) {
                Approximation::Exact(mantissa) => (mantissa, None),
                Approximation::Inexact(mantissa, error) => (mantissa, Some(error)),
            };
            let mantissa: $mantissa = mantissa
                .try_into()
                .expect("rounded mantissa fits native float encoding");
            let mantissa = if sign == Sign::Negative {
                -mantissa
            } else {
                mantissa
            };

            let value = <$ty>::encode(mantissa, mantissa_exp as i16).value();
            if let Some(error) = error {
                with_sign(value, sign, error)
            } else {
                Approximation::Exact(value)
            }
        }
    };
}

impl_rbig_to_float!(to_f32_approx, f32, i32, 24, -126, 127);
impl_rbig_to_float!(to_f64_approx, f64, i64, 53, -1022, 1023);

pub(super) fn to_f32_rounded(v: RBig) -> f32 {
    to_f32_approx(v).value()
}

pub(super) fn to_f64_rounded(v: RBig) -> f64 {
    to_f64_approx(v).value()
}
