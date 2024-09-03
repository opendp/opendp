use syn::ReturnType;

use super::docstring::BootstrapDocstring;

#[test]
fn test_docstring_description_from_attrs() {
    let attrs = vec![];
    let output = ReturnType::Default;
    let path = None;

    let result = BootstrapDocstring::from_attrs(attrs, &output, path);

    if let Ok(docstring) = result {
        if let Some(description) = docstring.description {
            assert_eq!(description, "TODO".to_string())
        }
    }
}