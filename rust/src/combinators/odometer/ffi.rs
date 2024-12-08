use crate::ffi::any::AnyObject;

use super::{OdometerAnswer, OdometerQuery};

pub type AnyOdometerQuery = OdometerQuery<AnyObject, AnyObject>;
pub type AnyOdometerAnswer = OdometerAnswer<AnyObject, AnyObject>;
