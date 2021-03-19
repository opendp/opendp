use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;
use std::iter::repeat;
use std::marker::PhantomData;
use std::str::FromStr;

use crate::{Error, Fallible};
use crate::core::{DatasetMetric, Function, StabilityRelation, Transformation};
use crate::data::Column;
use crate::dom::{AllDomain, MapDomain, VectorDomain};
use crate::trans::{MakeTransformation0, MakeTransformation1, MakeTransformation2};
use std::hash::Hash;

pub type DataFrame<K> = HashMap<K, Column>;
pub type DataFrameDomain<K> = MapDomain<AllDomain<K>, AllDomain<Column>>;

pub struct CreateDataFrame<M> {
    metric: PhantomData<M>
}

/// ensure all rows have `len` number of cells
fn conform_records<'a>(len: usize, records: &Vec<Vec<&'a str>>) -> Vec<Vec<&'a str>> {
    records.into_iter().map(|record| match record.len().cmp(&len) {
        Ordering::Less => // record is too short; pad with empty strings
            record.clone().into_iter().chain(repeat("").take(len - record.len())).collect(),
        Ordering::Equal => // record is just right
            record.clone(),
        Ordering::Greater => // record is too long; slice down
            record[0..len].to_vec().clone()
    }).collect()
}

fn create_dataframe<K: Eq + Hash>(col_names: Vec<K>, records: &Vec<Vec<&str>>) -> DataFrame<K> {
    // make data rectangular
    let records = conform_records(col_names.len(), &records);

    // transpose and collect into dataframe
    col_names.into_iter().enumerate()
        .map(|(i, col_name)| (
            col_name,
            Column::new(records.iter().map(|record| record[i].to_string()).collect())))
        .collect()
}

pub fn create_dataframe_domain<K: Eq + Hash>() -> DataFrameDomain<K> {
    MapDomain::new(AllDomain::new(), AllDomain::new())
}


impl<M, K> MakeTransformation1<VectorDomain<VectorDomain<AllDomain<String>>>, DataFrameDomain<K>, M, M, Vec<K>> for CreateDataFrame<M>
    where M: Clone + DatasetMetric<Distance=u32>,
          K: 'static + Eq + Hash + Debug + Clone {
    fn make1(col_names: Vec<K>) -> Fallible<Transformation<VectorDomain<VectorDomain<AllDomain<String>>>, DataFrameDomain<K>, M, M>> {
        Ok(Transformation::new(
            VectorDomain::new(VectorDomain::new_all()),
            create_dataframe_domain(),
            Function::new(move |arg: &Vec<Vec<String>>| -> DataFrame<K> {
                let arg = arg.into_iter().map(|e| vec_string_to_str(e)).collect();
                create_dataframe(col_names.clone(), &arg)
            }),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(1_u32)))
    }
}

pub struct SplitDataFrame<M> {
    metric: PhantomData<M>
}

fn split_dataframe<'a, K: Hash + Eq>(separator: &str, col_names: Vec<K>, s: &str) -> DataFrame<K> {
    let lines = split_lines(s);
    let records = split_records(separator, &lines);
    let records = conform_records(col_names.len(), &records);
    create_dataframe(col_names, &records)
}

impl<M, K> MakeTransformation2<AllDomain<String>, DataFrameDomain<K>, M, M, Option<&str>, Vec<K>> for SplitDataFrame<M>
    where K: 'static + Hash + Eq + Clone,
          M: Clone + DatasetMetric<Distance=u32> {
    fn make2(separator: Option<&str>, col_names: Vec<K>) -> Fallible<Transformation<AllDomain<String>, DataFrameDomain<K>, M, M>> {
        let separator = separator.unwrap_or(",").to_owned();
        Ok(Transformation::new(
            AllDomain::new(),
            create_dataframe_domain(),
            Function::new(move |arg: &String| split_dataframe(&separator, col_names.clone(), &arg)),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(1_u32)))
    }
}

pub struct ParseColumn<M, T> {
    metric: PhantomData<M>,
    data: PhantomData<T>,
}

fn replace_col<K: Eq + Hash + Debug + Clone>(key: &K, df: &DataFrame<K>, col: Column) -> Fallible<DataFrame<K>> {
    let mut df = df.clone();
    *df.get_mut(key)
        .ok_or_else(|| Error::FailedFunction(format!("column does not exist: {:?}", key)))? = col;
    Ok(df)
}

fn parse_column<K, T>(key: &K, impute: bool, df: &DataFrame<K>) -> Fallible<DataFrame<K>>
    where T: 'static + Debug + Clone + PartialEq + FromStr + Default,
          K: Eq + Hash + Clone + Debug,
          T::Err: Debug {
    let col = df.get(key)
        .ok_or_else(|| Error::FailedFunction(format!("column does not exist: {:?}", key)))?
        .as_form()?;
    let col = vec_string_to_str(col);
    let col = parse_series::<T>(&col, impute)?;
    replace_col(key, &df, col.into())
}

impl<M, K, T> MakeTransformation2<DataFrameDomain<K>, DataFrameDomain<K>, M, M, K, bool> for ParseColumn<M, T>
    where M: Clone + DatasetMetric<Distance=u32>,
          K: 'static + Hash + Eq + Debug + Clone,
          T: 'static + Debug + FromStr + Clone + Default + PartialEq,
          T::Err: Debug {
    fn make2(key: K, impute: bool) -> Fallible<Transformation<DataFrameDomain<K>, DataFrameDomain<K>, M, M>> {
        Ok(Transformation::new(
            create_dataframe_domain(),
            create_dataframe_domain(),
            Function::new_fallible(move |arg: &DataFrame<K>| parse_column::<K, T>(&key, impute, arg)),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(1_u32)))
    }
}

pub struct SelectColumn<M, T> {
    metric: PhantomData<M>,
    data: PhantomData<T>,
}


impl<M, K, T> MakeTransformation1<DataFrameDomain<K>, VectorDomain<AllDomain<T>>, M, M, K> for SelectColumn<M, T>
    where M: Clone + DatasetMetric<Distance=u32>,
          K: 'static + Eq + Hash + Debug,
          T: 'static + Debug + Clone + PartialEq {
    fn make1(key: K) -> Fallible<Transformation<DataFrameDomain<K>, VectorDomain<AllDomain<T>>, M, M>> {
        Ok(Transformation::new(
            create_dataframe_domain(),
            VectorDomain::new_all(),
            Function::new_fallible(move |arg: &DataFrame<K>| -> Fallible<Vec<T>> {
                // retrieve column from dataframe and handle error
                arg.get(&key).ok_or_else(|| Error::FailedFunction(format!("column does not exist: {:?}", key)))?
                    // cast down to &Vec<T>
                    .as_form::<Vec<T>>().map(|c| c.clone())
            }),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(1_u32)))
    }
}

/// A [`Transformation`] that takes a `String` and splits it into a `Vec<String>` of its lines.
pub struct SplitLines<M> {
    metric: PhantomData<M>
}

fn vec_string_to_str(src: &Vec<String>) -> Vec<&str> {
    src.into_iter().map(|e| e.as_str()).collect()
}

fn vec_str_to_string(src: Vec<&str>) -> Vec<String> {
    src.into_iter().map(|e| e.to_owned()).collect()
}

fn split_lines(s: &str) -> Vec<&str> {
    s.lines().collect()
}

impl<M> MakeTransformation0<AllDomain<String>, VectorDomain<AllDomain<String>>, M, M> for SplitLines<M>
    where M: Clone + DatasetMetric<Distance=u32> {
    fn make0() -> Fallible<Transformation<AllDomain<String>, VectorDomain<AllDomain<String>>, M, M>> {
        Ok(Transformation::new(
            AllDomain::<String>::new(),
            VectorDomain::new_all(),
            Function::new(|arg: &String| -> Vec<String> {
                arg.lines().map(|v| v.to_owned()).collect()
            }),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(1_u32)))
    }
}

pub struct ParseSeries<T, M> {
    data: PhantomData<T>,
    metric: PhantomData<M>,
}

fn parse_series<T>(col: &Vec<&str>, default_on_error: bool) -> Fallible<Vec<T>> where
    T: FromStr + Default,
    T::Err: Debug {
    if default_on_error {
        Ok(col.into_iter().map(|v| v.parse().unwrap_or_default()).collect())
    } else {
        col.into_iter().map(|v| v.parse().map_err(|e| Error::FailedCastTo(format!("{:?}", e)).into())).collect()
    }
}

impl<T, M> MakeTransformation1<VectorDomain<AllDomain<String>>, VectorDomain<AllDomain<T>>, M, M, bool> for ParseSeries<T, M>
    where M: Clone + DatasetMetric<Distance=u32>,
          T: FromStr + Default,
          T::Err: Debug {
    fn make1(impute: bool) -> Fallible<Transformation<VectorDomain<AllDomain<String>>, VectorDomain<AllDomain<T>>, M, M>> {
        Ok(Transformation::new(
            VectorDomain::new_all(),
            VectorDomain::new_all(),
            Function::new_fallible(move |arg: &Vec<String>| -> Fallible<Vec<T>> {
                let arg = vec_string_to_str(arg);
                parse_series(&arg, impute)
            }),
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(1_u32)))
    }
}

pub struct SplitRecords<M> {
    // TODO: evaluate if we want to keep PhantomData around in these structs
    metric: PhantomData<M>
}

fn split_records<'a>(separator: &str, lines: &Vec<&'a str>) -> Vec<Vec<&'a str>> {
    fn split<'a>(line: &'a str, separator: &str) -> Vec<&'a str> {
        line.split(separator).into_iter().map(|e| e.trim()).collect()
    }
    lines.into_iter().map(|e| split(e, separator)).collect()
}

impl<M> MakeTransformation1<VectorDomain<AllDomain<String>>, VectorDomain<VectorDomain<AllDomain<String>>>, M, M, Option<&str>> for SplitRecords<M>
    where M: Clone + DatasetMetric<Distance=u32> {
    fn make1(separator: Option<&str>) -> Fallible<Transformation<VectorDomain<AllDomain<String>>, VectorDomain<VectorDomain<AllDomain<String>>>, M, M>> {
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
            M::new(),
            M::new(),
            StabilityRelation::new_from_constant(1_u32)))
    }
}


#[cfg(test)]
mod tests {
    use crate::core::ChainTT;
    use crate::dist::HammingDistance;

    use super::*;

    #[test]
    fn test_make_create_dataframe() {
        let transformation = CreateDataFrame::<HammingDistance>::make(vec!["0".to_string(), "1".to_string()]).unwrap();
        let arg = vec![
            vec!["ant".to_owned(), "foo".to_owned()],
            vec!["bat".to_owned(), "bar".to_owned()],
            vec!["cat".to_owned(), "baz".to_owned()],
        ];
        let ret = transformation.function.eval(&arg).unwrap();
        let expected: DataFrame<String> = vec![
            ("0".to_owned(), Column::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Column::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_split_dataframe() {
        let transformation = SplitDataFrame::<HammingDistance>::make(None, vec!["0".to_string(), "1".to_string()]).unwrap();
        let arg = "ant, foo\nbat, bar\ncat, baz".to_owned();
        let ret = transformation.function.eval(&arg).unwrap();
        let expected: DataFrame<String> = vec![
            ("0".to_owned(), Column::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Column::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_parse_column() {
        let transformation = ParseColumn::<HammingDistance, i32>::make(1, true).unwrap();
        let arg: DataFrame<usize> = vec![
            (0, Column::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            (1, Column::new(vec!["1".to_owned(), "2".to_owned(), "".to_owned()])),
        ].into_iter().collect();
        let ret = transformation.function.eval(&arg).unwrap();
        let expected: DataFrame<usize> = vec![
            (0, Column::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            (1, Column::new(vec![1, 2, 0])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_parse_columns() {
        let transformation0 = ParseColumn::<HammingDistance, i32>::make("1".to_string(), true).unwrap();
        let transformation1 = ParseColumn::<HammingDistance, f64>::make("2".to_string(), true).unwrap();
        let transformation = ChainTT::make(&transformation1, &transformation0).unwrap();
        let arg: DataFrame<String> = vec![
            ("0".to_owned(), Column::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Column::new(vec!["1".to_owned(), "2".to_owned(), "3".to_owned()])),
            ("2".to_owned(), Column::new(vec!["1.1".to_owned(), "2.2".to_owned(), "3.3".to_owned()])),
        ].into_iter().collect();
        let ret = transformation.function.eval(&arg).unwrap();
        let expected: DataFrame<String> = vec![
            ("0".to_owned(), Column::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Column::new(vec![1, 2, 3])),
            ("2".to_owned(), Column::new(vec![1.1, 2.2, 3.3])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_select_column() {
        let transformation = SelectColumn::<HammingDistance, String>::make("1".to_owned()).unwrap();
        let arg: DataFrame<String> = vec![
            ("0".to_owned(), Column::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Column::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        let ret = transformation.function.eval(&arg).unwrap();
        let expected = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        assert_eq!(ret, expected);
    }
}