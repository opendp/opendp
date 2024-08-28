use crate::codegen::python::generate_function;
use crate::{Argument, Function, TypeRecipe, Value};
use std::collections::HashMap;

#[test]
fn test_python_code_generation() {
    let argument = Argument {
        name: Some("fake_argument".to_string()),
        c_type: Some("double".to_string()),
        rust_type: Some(TypeRecipe::Name("f64".to_string())),
        hint: None,
        description: Some("fake description".to_string()),
        default:  Some(Value::Float(99.9)),
        generics: vec![],
        is_type: false,
        do_not_convert: false,
        example: None,
    };

    let return_argument = Argument {
        name: Some("fake_argument".to_string()),
        c_type: Some("double".to_string()),
        rust_type: Some(TypeRecipe::Name("f64".to_string())),
        hint: None,
        description: Some("fake description".to_string()),
        default:  Some(Value::Float(99.9)),
        generics: vec![],
        is_type: false,
        do_not_convert: false,
        example: None,
    };

    let function = Function {
        name: "fake_function".to_string(),
        description: Some("fake description".to_string()),
        features: vec!["fake_feature".to_string()],
        args: vec![argument],
        derived_types: vec![],
        ret: return_argument,
        dependencies: vec![],
        supports_partial: false,
        has_ffi: true,
    };

    let typemap: HashMap<String, String> =
        serde_json::from_str(&include_str!("python_typemap.json")).unwrap();
    let actual_code = generate_function("fake_module", &function, &typemap, &HashMap::new());
    let expected_code = "
def fake_function(
    fake_argument = 99.9
):
    r\"\"\"fake description

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
    assert_eq!(actual_code, expected_code);
}
