use crate::codegen::python::generate_function;
use crate::{Argument, Function};
use std::collections::HashMap;

#[test]
fn test_python_code_generation() {
    // TODO: What is the least we can fill in below to get it to run?
    let argument = Argument {
        name: None,
        c_type: None,
        rust_type: None,
        hint: None,
        description: None,
        default: None,
        generics: vec![],
        is_type: true,
        do_not_convert: false,
        example: None,
    };

    let function = Function {
        name: "fake_function".to_string(),
        description: Some("fake description".to_string()),
        features: vec!["fake_feature".to_string()],
        args: vec![],
        derived_types: vec![],
        ret: argument,
        dependencies: vec![],
        supports_partial: false,
        has_ffi: true,
    };

    let actual_code = generate_function("fake_module", &function, &HashMap::new(), &HashMap::new());
    let expected_code = "TODO";
    assert_eq!(actual_code, expected_code);
}
