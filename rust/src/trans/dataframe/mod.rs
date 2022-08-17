mod apply;
pub use apply::*;

mod create;
pub use create::*;

mod select;
pub use select::*;

mod subset;
pub use subset::*;

use std::collections::HashMap;

use crate::data::Column;
use crate::domains::{AllDomain, MapDomain};

pub type DataFrame<K> = HashMap<K, Column>;
pub type DataFrameDomain<K> = MapDomain<AllDomain<K>, AllDomain<Column>>;
