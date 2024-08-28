use crate::codegen::{python, r};
use crate::{Argument, Function, TypeRecipe, Value};
use std::collections::HashMap;

fn make_argument() -> Argument {
    Argument {
        name: Some("fake_argument".to_string()),
        c_type: Some("double".to_string()),
        rust_type: Some(TypeRecipe::Name("f64".to_string())),
        hint: None,
        description: Some("fake description".to_string()),
        default: Some(Value::Float(99.9)),
        generics: vec![],
        is_type: false,
        do_not_convert: false,
        example: None,
    }
}

fn make_function(parameter_argument: Argument, return_argument: Argument) -> Function {
    Function {
        name: "fake_function".to_string(),
        description: Some("fake description".to_string()),
        features: vec!["fake_feature".to_string()],
        args: vec![parameter_argument],
        derived_types: vec![],
        ret: return_argument,
        dependencies: vec![],
        supports_partial: false,
        has_ffi: true,
    }
}

#[test]
fn test_python_code_generation() {
    let parameter_argument = make_argument();
    let return_argument = make_argument();
    let function = make_function(parameter_argument, return_argument);

    let typemap: HashMap<String, String> =
        serde_json::from_str(&include_str!("python_typemap.json")).unwrap();
    let generated_code =
        python::generate_function("fake_module", &function, &typemap, &HashMap::new());
    let expected_code = "
def fake_function(
    fake_argument = 99.9
):
    r\"\"\"fake description

    Requires `enable_features(\"fake_feature\")`.

    :param fake_argument: fake description
    :return: fake description
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    \"\"\"
    assert_features(\"fake_feature\")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_fake_argument = py_to_c(fake_argument, c_type=ctypes.c_double, type_name=f64)

    # Call library function.
    lib_function = lib.opendp_fake_module__fake_function
    lib_function.argtypes = [ctypes.c_double]
    lib_function.restype = ctypes.c_double

    output = c_to_py(lib_function(c_fake_argument))

    return output
";
    let esc_generated_code = generated_code.replace("\"", "\\\"");
    let esc_expected_code = expected_code.replace("\"", "\\\"");
    assert_eq!(generated_code, expected_code, "\nGenerated:{esc_generated_code}\nExpected:{esc_expected_code}");
}

#[test]
fn test_r_code_generation() {
    let parameter_argument = make_argument();
    let return_argument = make_argument();
    let function = make_function(parameter_argument, return_argument);

    let hierarchy: HashMap<String, Vec<String>> =
        serde_json::from_str(&include_str!("type_hierarchy.json")).unwrap();
    let generated_code = r::r::generate_r_function("fake_module", &function, &hierarchy);
    let expected_code = "
#' fake description
#'
#' Requires `enable_features(\"fake_feature\")`.
#'
#' @concept fake_module
#' @param fake_argument fake description
#' @return fake description
#' @export
fake_function <- function(
  fake_argument = 99.9
) {
  assert_features(\"fake_feature\")

  # No type arguments to standardize.
  log <- new_constructor_log(\"fake_function\", \"fake_module\", new_hashtab(
    list(\"fake_argument\"),
    list(unbox2(fake_argument))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(fake_argument))

  # Call wrapper function.
  output <- .Call(
    \"fake_module__fake_function\",
    fake_argument,
    log, PACKAGE = \"opendp\")
  output
}
";
    let esc_generated_code = generated_code.replace("\"", "\\\"");
    let esc_expected_code = expected_code.replace("\"", "\\\"");
    assert_eq!(generated_code, expected_code, "\nGenerated:{esc_generated_code}\nExpected:{esc_expected_code}");
}
