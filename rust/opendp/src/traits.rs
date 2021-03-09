use std::convert::TryInto;

pub trait DPDistanceCast<T> where Self: Sized {
    fn cast(x: T) -> Result<Self, &'static str>;
}

impl<T> DPDistanceCast<T> for T {
    fn cast(x: T) -> Result<T, &'static str> {
        Ok(x)
    }
}

impl DPDistanceCast<f64> for i8 {
    fn cast(x: f64) -> Result<i8, &'static str> {
        if x.is_sign_negative() {
            return Err("distances cannot be negative")
        }
        Ok(x as i8)
    }
}

impl DPDistanceCast<i8> for f64 {
    fn cast(x: i8) -> Result<f64, &'static str> {
        Ok(x as f64)
    }
}
