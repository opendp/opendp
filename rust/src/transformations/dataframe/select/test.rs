use super::*;
use crate::{data::Column, error::ExplainUnwrap};

#[test]
fn test_make_select_column() {
    #[allow(deprecated)]
    let transformation = make_select_column::<String, String>("1".to_owned()).unwrap_test();
    let arg: DataFrame<String> = vec![
        (
            "0".to_owned(),
            Column::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()]),
        ),
        (
            "1".to_owned(),
            Column::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]),
        ),
    ]
    .into_iter()
    .collect();
    let ret = transformation.invoke(&arg).unwrap_test();
    let expected = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
    assert_eq!(ret, expected);
}
