use std::{cmp::Ordering, iter::repeat};

use opendp_derive::bootstrap;
use polars::prelude::*;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain, LazyFrameDomain, SeriesDomain},
    error::Fallible,
    metrics::SymmetricDistance,
};

#[cfg(feature = "ffi")]
mod ffi;

/// ensure all rows have `len` number of cells
fn conform_records<'a>(len: usize, records: &[Vec<&'a str>]) -> Vec<Vec<&'a str>> {
    records
        .iter()
        .map(|record| match record.len().cmp(&len) {
            Ordering::Less =>
            // record is too short; pad with empty strings
            {
                record
                    .clone()
                    .into_iter()
                    .chain(repeat("").take(len - record.len()))
                    .collect()
            }
            Ordering::Equal =>
            // record is just right
            {
                record.clone()
            }
            Ordering::Greater =>
            // record is too long; slice down
            {
                record[0..len].to_vec()
            }
        })
        .collect()
}

fn create_dataframe(col_names: Vec<String>, records: &[Vec<&str>]) -> Fallible<LazyFrame> {
    // make data rectangular
    let records = conform_records(col_names.len(), &records);

    let mut df = LazyFrame::default();

    // transpose and collect into dataframe
    DataFrame::new(
        col_names
            .into_iter()
            .enumerate()
            .map(|(i, col_name)| {
                Series::new(
                    col_name.as_str(),
                    records
                        .iter()
                        .map(|record| record[i].to_string())
                        .collect::<Vec<_>>(),
                )
            })
            .collect(),
    ).map(DataFrame::lazy).map_err(From::from)
}

#[bootstrap(features("contrib"))]
/// Make a Transformation that constructs a dataframe from a `Vec<Vec<String>>` (a vector of records).
///
/// # Arguments
/// * `col_names` - Column names for each record entry.
pub fn make_create_dataframe(
    col_names: Vec<String>,
) -> Fallible<
    Transformation<
        VectorDomain<VectorDomain<AtomDomain<String>>>,
        LazyFrameDomain,
        SymmetricDistance,
        SymmetricDistance,
    >,
>
{
    Transformation::new(
        VectorDomain::new(VectorDomain::new(AtomDomain::default(), None), None),
        LazyFrameDomain::new(col_names.iter().map(|name| SeriesDomain::new::<String>(name)).collect())?,
        Function::new_fallible(move |arg: &Vec<Vec<String>>| -> Fallible<LazyFrame> {
            let arg: Vec<_> = arg.iter().map(|e| vec_string_to_str(e)).collect();
            create_dataframe(col_names.clone(), &arg)
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1),
    )
}

fn split_dataframe(separator: &str, col_names: Vec<String>, s: &str) -> Fallible<LazyFrame> {
    let lines = split_lines(s);
    let records = split_records(separator, &lines);
    let records = conform_records(col_names.len(), &records);
    create_dataframe(col_names, &records)
}

#[bootstrap(
    features("contrib"),
    arguments(separator(c_type = "char *", rust_type = b"null"))
)]
/// Make a Transformation that splits each record in a String into a `Vec<Vec<String>>`,
/// and loads the resulting table into a dataframe keyed by `col_names`.
///
/// # Arguments
/// * `separator` - The token(s) that separate entries in each record.
/// * `col_names` - Column names for each record entry.
///
/// # Generics
/// * `K` - categorical/hashable data type of column names
pub fn make_split_dataframe(
    separator: Option<&str>,
    col_names: Vec<String>,
) -> Fallible<
    Transformation<AtomDomain<String>, LazyFrameDomain, SymmetricDistance, SymmetricDistance>,
> {
    let separator = separator.unwrap_or(",").to_owned();
    Transformation::new(
        AtomDomain::default(),
        LazyFrameDomain::new(col_names.iter().map(|name| SeriesDomain::new::<String>(name)).collect())?,
        Function::new_fallible(move |arg: &String| split_dataframe(&separator, col_names.clone(), &arg)),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1),
    )
}

fn vec_string_to_str(src: &[String]) -> Vec<&str> {
    src.iter().map(|e| e.as_str()).collect()
}

fn vec_str_to_string(src: Vec<&str>) -> Vec<String> {
    src.into_iter().map(|e| e.to_owned()).collect()
}

fn split_lines(s: &str) -> Vec<&str> {
    s.lines().collect()
}

#[bootstrap(features("contrib"))]
/// Make a Transformation that takes a string and splits it into a `Vec<String>` of its lines.
pub fn make_split_lines() -> Fallible<
    Transformation<
        AtomDomain<String>,
        VectorDomain<AtomDomain<String>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
> {
    Transformation::new(
        AtomDomain::<String>::default(),
        VectorDomain::new(AtomDomain::default(), None),
        Function::new(|arg: &String| -> Vec<String> {
            arg.lines().map(|v| v.to_owned()).collect()
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1),
    )
}

fn split_records<'a>(separator: &str, lines: &[&'a str]) -> Vec<Vec<&'a str>> {
    fn split<'a>(line: &'a str, separator: &str) -> Vec<&'a str> {
        line.split(separator)
            .into_iter()
            .map(|e| e.trim())
            .collect()
    }
    lines.iter().map(|e| split(e, separator)).collect()
}

#[bootstrap(
    features("contrib"),
    arguments(separator(c_type = "char *", rust_type = b"null"))
)]
/// Make a Transformation that splits each record in a `Vec<String>` into a `Vec<Vec<String>>`.
///
/// # Arguments
/// * `separator` - The token(s) that separate entries in each record.
pub fn make_split_records(
    separator: Option<&str>,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<String>>,
        VectorDomain<VectorDomain<AtomDomain<String>>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
> {
    let separator = separator.unwrap_or(",").to_owned();
    Transformation::new(
        VectorDomain::new(AtomDomain::default(), None),
        VectorDomain::new(VectorDomain::new(AtomDomain::default(), None), None),
        // move is necessary because it captures `separator`
        Function::new(move |arg: &Vec<String>| -> Vec<Vec<String>> {
            let arg = vec_string_to_str(arg);
            let ret = split_records(&separator, &arg);
            ret.into_iter().map(vec_str_to_string).collect()
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1),
    )
}

#[cfg(test)]
mod tests {
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
    fn test_make_create_dataframe() -> Fallible<()> {
        let transformation = make_create_dataframe(vec!["0".to_string(), "1".to_string()]).unwrap_test();
        let arg = vec![
            vec!["ant".to_owned(), "foo".to_owned()],
            vec!["bat".to_owned(), "bar".to_owned()],
            vec!["cat".to_owned(), "baz".to_owned()],
        ];
        let ret = transformation.invoke(&arg)?.collect()?;
        let expected = df![
            "0" => vec!["ant", "bat", "cat"],
            "1" => vec!["foo", "bar", "baz"]
        ]?;
        assert_eq!(ret, expected);
        Ok(())
    }

    #[test]
    fn test_make_split_dataframe() -> Fallible<()> {
        let transformation =
            make_split_dataframe(None, vec!["0".to_string(), "1".to_string()])
                .unwrap_test();
        let arg = "ant, foo\nbat, bar\ncat, baz".to_owned();
        let ret = transformation.invoke(&arg)?.collect()?;
        let expected = df![
            "0" => &["ant", "bat", "cat"],
            "1" => &["foo", "bar", "baz"]
        ]?;
        assert_eq!(ret, expected);
        Ok(())
    }
}
