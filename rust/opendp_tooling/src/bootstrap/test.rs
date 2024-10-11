use pretty_assertions::assert_eq;
use syn::ReturnType;

use super::docstring::BootstrapDocstring;

#[test]
fn test_docstring_description_from_attrs() {
    let name = "fake_name".to_string();
    let attrs = vec![];
    let output = ReturnType::Default;
    let path = None;
    let features = vec!["feature_1".to_string(), "feature_2".to_string()];

    let result = BootstrapDocstring::from_attrs(&name, attrs, &output, path, features);

    let docstring = result.expect("from_attrs failed");
    let description = docstring.description.unwrap();

    assert_eq!(
        description,
        "Required features: `feature_1`, `feature_2`".to_string()
    );
}
