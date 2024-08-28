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

    let actual_code = generate_function("fake_module", &function, &HashMap::new(), &HashMap::new());
    let expected_code = "TODO";
    assert_eq!(actual_code, expected_code);
}
