#[cfg(feature="ffi")]
mod ffi;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::repeat;

use crate::core::{Function, StabilityRelation, Transformation};
use crate::data::Column;
use crate::dom::{AllDomain, MapDomain, VectorDomain};
use crate::error::*;
use crate::dist::SymmetricDistance;
use crate::traits::CheckNull;

pub type DataFrame<K> = HashMap<K, Column>;
pub type DataFrameDomain<K> = MapDomain<AllDomain<K>, AllDomain<Column>>;

/// ensure all rows have `len` number of cells
fn conform_records<'a>(len: usize, records: &[Vec<&'a str>]) -> Vec<Vec<&'a str>> {
    records.iter().map(|record| match record.len().cmp(&len) {
        Ordering::Less => // record is too short; pad with empty strings
            record.clone().into_iter().chain(repeat("").take(len - record.len())).collect(),
        Ordering::Equal => // record is just right
            record.clone(),
        Ordering::Greater => // record is too long; slice down
            record[0..len].to_vec()
    }).collect()
}

fn create_dataframe<K: Eq + Hash>(col_names: Vec<K>, records: &[Vec<&str>]) -> DataFrame<K> {
    // make data rectangular
    let records = conform_records(col_names.len(), &records);

    // transpose and collect into dataframe
    col_names.into_iter().enumerate()
        .map(|(i, col_name)| (
            col_name,
            Column::new(records.iter().map(|record| record[i].to_string()).collect())))
        .collect()
}

fn create_dataframe_domain<K: Eq + Hash + CheckNull>() -> DataFrameDomain<K> {
    MapDomain::new(AllDomain::new(), AllDomain::new())
}

pub fn make_create_dataframe<K>(
    col_names: Vec<K>
) -> Fallible<Transformation<VectorDomain<VectorDomain<AllDomain<String>>>, DataFrameDomain<K>, SymmetricDistance, SymmetricDistance>>
    where K: 'static + Eq + Hash + Clone + CheckNull {
    Ok(Transformation::new(
        VectorDomain::new(VectorDomain::new_all()),
        create_dataframe_domain(),
        Function::new(move |arg: &Vec<Vec<String>>| -> DataFrame<K> {
            let arg: Vec<_>  = arg.iter().map(|e| vec_string_to_str(e)).collect();
            create_dataframe(col_names.clone(), &arg)
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityRelation::new_from_constant(1)
    ))
}

fn split_dataframe<K: Hash + Eq>(separator: &str, col_names: Vec<K>, s: &str) -> DataFrame<K> {
    let lines = split_lines(s);
    let records = split_records(separator, &lines);
    let records = conform_records(col_names.len(), &records);
    create_dataframe(col_names, &records)
}

pub fn make_split_dataframe<K>(
    separator: Option<&str>, col_names: Vec<K>
) -> Fallible<Transformation<AllDomain<String>, DataFrameDomain<K>, SymmetricDistance, SymmetricDistance>>
    where K: 'static + Hash + Eq + Clone + CheckNull {
    let separator = separator.unwrap_or(",").to_owned();
    Ok(Transformation::new(
        AllDomain::new(),
        create_dataframe_domain(),
        Function::new(move |arg: &String| split_dataframe(&separator, col_names.clone(), &arg)),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityRelation::new_from_constant(1)))
}

pub fn make_select_column<K, TOA>(key: K) -> Fallible<Transformation<DataFrameDomain<K>, VectorDomain<AllDomain<TOA>>, SymmetricDistance, SymmetricDistance>>
    where K: 'static + Eq + Hash + Debug + CheckNull,
          TOA: 'static + Debug + Clone + PartialEq + CheckNull {
    Ok(Transformation::new(
        create_dataframe_domain(),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &DataFrame<K>| -> Fallible<Vec<TOA>> {
            // retrieve column from dataframe and handle error
            arg.get(&key).ok_or_else(|| err!(FailedFunction, "column does not exist: {:?}", key))?
                // cast down to &Vec<T>
                .as_form::<Vec<TOA>>().map(|c| c.clone())
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityRelation::new_from_constant(1)))
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

/// A [`Transformation`] that takes a `String` and splits it into a `Vec<String>` of its lines.
pub fn make_split_lines() -> Fallible<Transformation<AllDomain<String>, VectorDomain<AllDomain<String>>, SymmetricDistance, SymmetricDistance>> {
    Ok(Transformation::new(
        AllDomain::<String>::new(),
        VectorDomain::new_all(),
        Function::new(|arg: &String| -> Vec<String> {
            arg.lines().map(|v| v.to_owned()).collect()
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityRelation::new_from_constant(1)))
}

fn split_records<'a>(separator: &str, lines: &[&'a str]) -> Vec<Vec<&'a str>> {
    fn split<'a>(line: &'a str, separator: &str) -> Vec<&'a str> {
        line.split(separator).into_iter().map(|e| e.trim()).collect()
    }
    lines.iter().map(|e| split(e, separator)).collect()
}

pub fn make_split_records(separator: Option<&str>) -> Fallible<Transformation<VectorDomain<AllDomain<String>>, VectorDomain<VectorDomain<AllDomain<String>>>, SymmetricDistance, SymmetricDistance>> {
    let separator = separator.unwrap_or(",").to_owned();
    Ok(Transformation::new(
        VectorDomain::new_all(),
        VectorDomain::new(VectorDomain::new_all()),
        // move is necessary because it captures `separator`
        Function::new(move |arg: &Vec<String>| -> Vec<Vec<String>> {
            let arg = vec_string_to_str(arg);
            let ret = split_records(&separator, &arg);
            ret.into_iter().map(vec_str_to_string).collect()
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityRelation::new_from_constant(1)))
}


#[cfg(test)]
mod tests {
    use crate::error::ExplainUnwrap;
    use super::*;

    #[test]
    fn test_make_split_lines() {
        let transformation = make_split_lines().unwrap_test();
        let arg = "ant\nbat\ncat\n".to_owned();
        let ret = transformation.invoke(&arg).unwrap_test();
        assert_eq!(ret, vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()]);
    }

    #[test]
    fn test_make_split_records() {
        let transformation = make_split_records(None).unwrap_test();
        let arg = vec!["ant, foo".to_owned(), "bat, bar".to_owned(), "cat, baz".to_owned()];
        let ret = transformation.invoke(&arg).unwrap_test();
        assert_eq!(ret, vec![
            vec!["ant".to_owned(), "foo".to_owned()],
            vec!["bat".to_owned(), "bar".to_owned()],
            vec!["cat".to_owned(), "baz".to_owned()],
        ]);
    }

    #[test]
    fn test_make_create_dataframe() {
        let transformation = make_create_dataframe::<u32>(vec![0, 1]).unwrap_test();
        let arg = vec![
            vec!["ant".to_owned(), "foo".to_owned()],
            vec!["bat".to_owned(), "bar".to_owned()],
            vec!["cat".to_owned(), "baz".to_owned()],
        ];
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected: DataFrame<u32> = vec![
            (0, Column::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            (1, Column::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_split_dataframe() {
        let transformation = make_split_dataframe::<String>(None, vec!["0".to_string(), "1".to_string()]).unwrap_test();
        let arg = "ant, foo\nbat, bar\ncat, baz".to_owned();
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected: DataFrame<String> = vec![
            ("0".to_owned(), Column::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Column::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_select_column() {
        let transformation = make_select_column::<String, String>("1".to_owned()).unwrap_test();
        let arg: DataFrame<String> = vec![
            ("0".to_owned(), Column::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Column::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        let ret = transformation.invoke(&arg).unwrap_test();
        let expected = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        assert_eq!(ret, expected);
    }
}