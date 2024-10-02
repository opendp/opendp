use std::{cmp::Ordering, iter::repeat};

use opendp_derive::bootstrap;

use crate::{
    core::{Function, StabilityMap, Transformation},
    data::Column,
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::SymmetricDistance,
    traits::Hashable,
};

use super::{DataFrame, DataFrameDomain};

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

fn create_dataframe<K: Hashable>(col_names: Vec<K>, records: &[Vec<&str>]) -> DataFrame<K> {
    // make data rectangular
    let records = conform_records(col_names.len(), &records);

    // transpose and collect into dataframe
    col_names
        .into_iter()
        .enumerate()
        .map(|(i, col_name)| {
            (
                col_name,
                Column::new(records.iter().map(|record| record[i].to_string()).collect()),
            )
        })
        .collect()
}

#[bootstrap(features("contrib"))]
#[deprecated(note = "Use Polars instead", since = "0.12.0")]
/// Make a Transformation that constructs a dataframe from a `Vec<Vec<String>>` (a vector of records).
///
/// # Arguments
/// * `col_names` - Column names for each record entry.
///
/// # Generics
/// * `K` - categorical/hashable data type of column names
pub fn make_create_dataframe<K>(
    col_names: Vec<K>,
) -> Fallible<
    Transformation<
        VectorDomain<VectorDomain<AtomDomain<String>>>,
        DataFrameDomain<K>,
        SymmetricDistance,
        SymmetricDistance,
    >,
>
where
    K: Hashable,
{
    Transformation::new(
        VectorDomain::new(VectorDomain::new(AtomDomain::default())),
        DataFrameDomain::new(),
        Function::new(move |arg: &Vec<Vec<String>>| -> DataFrame<K> {
            let arg: Vec<_> = arg.iter().map(|e| vec_string_to_str(e)).collect();
            create_dataframe(col_names.clone(), &arg)
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1),
    )
}

fn split_dataframe<K: Hashable>(separator: &str, col_names: Vec<K>, s: &str) -> DataFrame<K> {
    let lines = split_lines(s);
    let records = split_records(separator, &lines);
    let records = conform_records(col_names.len(), &records);
    create_dataframe(col_names, &records)
}

#[bootstrap(
    features("contrib"),
    arguments(separator(c_type = "char *", rust_type = b"null"))
)]
#[deprecated(note = "Use Polars instead", since = "0.12.0")]
/// Make a Transformation that splits each record in a String into a `Vec<Vec<String>>`,
/// and loads the resulting table into a dataframe keyed by `col_names`.
///
/// # Arguments
/// * `separator` - The token(s) that separate entries in each record.
/// * `col_names` - Column names for each record entry.
///
/// # Generics
/// * `K` - categorical/hashable data type of column names
pub fn make_split_dataframe<K>(
    separator: Option<&str>,
    col_names: Vec<K>,
) -> Fallible<
    Transformation<AtomDomain<String>, DataFrameDomain<K>, SymmetricDistance, SymmetricDistance>,
>
where
    K: Hashable,
{
    let separator = separator.unwrap_or(",").to_owned();
    Transformation::new(
        AtomDomain::default(),
        DataFrameDomain::new(),
        Function::new(move |arg: &String| split_dataframe(&separator, col_names.clone(), &arg)),
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
        VectorDomain::new(AtomDomain::default()),
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
        VectorDomain::new(AtomDomain::default()),
        VectorDomain::new(VectorDomain::new(AtomDomain::default())),
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
mod test;
