use super::*;
use crate::error::ExplainUnwrap;

#[test]
fn test_make_split_lines() {
    let transformation = make_split_lines().unwrap_test();
    let arg = "ant\nbat\ncat\n".to_owned();
    let ret = transformation.invoke(&arg).unwrap_test();
    assert_eq!(
        ret,
        vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()]
    );
}

#[test]
fn test_make_split_records() {
    let transformation = make_split_records(None).unwrap_test();
    let arg = vec![
        "ant, foo".to_owned(),
        "bat, bar".to_owned(),
        "cat, baz".to_owned(),
    ];
    let ret = transformation.invoke(&arg).unwrap_test();
    assert_eq!(
        ret,
        vec![
            vec!["ant".to_owned(), "foo".to_owned()],
            vec!["bat".to_owned(), "bar".to_owned()],
            vec!["cat".to_owned(), "baz".to_owned()],
        ]
    );
}

#[test]
fn test_make_create_dataframe() {
    #[allow(deprecated)]
    let transformation = make_create_dataframe::<u32>(vec![0, 1]).unwrap_test();
    let arg = vec![
        vec!["ant".to_owned(), "foo".to_owned()],
        vec!["bat".to_owned(), "bar".to_owned()],
        vec!["cat".to_owned(), "baz".to_owned()],
    ];
    let ret = transformation.invoke(&arg).unwrap_test();
    let expected: DataFrame<u32> = vec![
        (
            0,
            Column::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()]),
        ),
        (
            1,
            Column::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]),
        ),
    ]
    .into_iter()
    .collect();
    assert_eq!(ret, expected);
}

#[test]
fn test_make_split_dataframe() {
    #[allow(deprecated)]
    let transformation =
        make_split_dataframe::<String>(None, vec!["0".to_string(), "1".to_string()]).unwrap_test();
    let arg = "ant, foo\nbat, bar\ncat, baz".to_owned();
    let ret = transformation.invoke(&arg).unwrap_test();
    let expected: DataFrame<String> = vec![
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
    assert_eq!(ret, expected);
}
