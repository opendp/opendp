use std::collections::HashMap;
use std::path::PathBuf;

use crate::Function;

use self::{
    c::{generate_c_headers, generate_c_lib, generate_c_module},
    r::generate_r_module,
};

mod c;
mod r;

// some functions are called directly by R's C layer instead of through the codegen
const BLACKLIST: &'static [&'static str] = &[
    // core
    "_error_free",
    "bool_free",
    "error_free",
    "_function_free",
    "_measurement_free",
    "_transformation_free",
    "_function_free",
    "_metric_free",
    "_measure_free",
    "_domain_free",
    // data
    "object_as_slice",
    "ffislice_of_anyobjectptrs",
    "slice_as_object",
    "str_free",
    "slice_free",
    "object_free",
    // udf
    "make_user_transformation",
    "make_user_measurement",
    "new_function",
];

/// Top-level function to generate R bindings, including all modules.
pub fn generate_bindings(modules: &HashMap<String, Vec<Function>>) -> HashMap<PathBuf, String> {
    let hierarchy: HashMap<String, Vec<String>> =
        serde_json::from_str(&include_str!("../type_hierarchy.json")).unwrap();

    let r_bindings = modules
        .into_iter()
        .map(|(module_name, module)| {
            (
                PathBuf::from(format!("R/{}.R", module_name)),
                generate_r_module(module_name, module, &hierarchy),
            )
        })
        .collect::<HashMap<_, _>>();

    let mut c_bindings = modules
        .into_iter()
        .map(|(module_name, module)| {
            (
                PathBuf::from(format!("src/{}.c", module_name)),
                generate_c_module(module_name, module),
            )
        })
        .collect::<HashMap<_, _>>();

    c_bindings.insert(PathBuf::from("src/lib.c"), generate_c_lib(modules));
    c_bindings.insert(PathBuf::from("src/Ropendp.h"), generate_c_headers(modules));

    r_bindings
        .into_iter()
        .chain(c_bindings.into_iter())
        .collect()
}
