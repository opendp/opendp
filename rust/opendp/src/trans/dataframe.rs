use std::collections::HashMap;
use std::fmt::Debug;
use std::iter::repeat;
use std::marker::PhantomData;
use std::str::FromStr;

use crate::core::{DatasetMetric, Metric, Transformation};
use crate::data::{Data, Element};
use crate::dom::{AllDomain, MapDomain, VectorDomain};
use crate::trans::{MakeTransformation0, MakeTransformation1, MakeTransformation2};

pub struct CreateDataFrame<M> {
    metric: PhantomData<M>
}

fn conform_records<'a>(len: usize, records: &Vec<Vec<&'a str>>) -> Vec<Vec<&'a str>> {
    fn conform<'a>(record: &Vec<&'a str>, len: usize) -> Vec<&'a str> {
        if record.len() > len {
            record[0..len].to_vec().clone()
        } else if record.len() < len {
            record.clone().into_iter().chain(repeat("").take(len - record.len())).collect()
        } else {
            record.clone()
        }
    }
    records.into_iter().map(|e| conform(e, len)).collect()
}

pub type DataFrame = HashMap<String, Data>;

fn create_dataframe(col_count: usize, records: &Vec<Vec<&str>>) -> DataFrame {
    let records = conform_records(col_count, &records);
    let mut cols = vec![Vec::new(); col_count];
    for record in records.into_iter() {
        for i in 0..col_count {
            cols[i].push(record[i])
        }
    }
    cols.into_iter().enumerate().map(|(k, v)| (k.to_string(), Data::new(vec_str_to_string(v)))).collect()
}

pub fn create_dataframe_domain() -> MapDomain<AllDomain<Data>> {
    MapDomain::new(AllDomain::new())
}


impl<M> MakeTransformation1<VectorDomain<VectorDomain<AllDomain<String>>>, MapDomain<AllDomain<Data>>, M, M, usize> for CreateDataFrame<M>
    where M: Clone + Metric<Distance=u32> + DatasetMetric {
    fn construct(col_count: usize) -> Transformation<VectorDomain<VectorDomain<AllDomain<String>>>, MapDomain<AllDomain<Data>>, M, M> {
        Transformation::new(
            VectorDomain::new(VectorDomain::new_all()),
            create_dataframe_domain(),
            // move is necessary because it captures `col_count`
            move |arg: &Vec<Vec<String>>| -> DataFrame {
                let arg = arg.into_iter().map(|e| vec_string_to_str(e)).collect();
                create_dataframe(col_count, &arg)
            },
            M::new(),
            M::new(),
            |d_in: &u32, d_out: &u32| *d_out >= *d_in)
    }
}

pub struct SplitDataFrame<M> {
    metric: PhantomData<M>
}

fn split_dataframe<'a>(separator: &str, col_count: usize, s: &str) -> DataFrame {
    let lines = split_lines(s);
    let records = split_records(separator, &lines);
    let records = conform_records(col_count, &records);
    create_dataframe(col_count, &records)
}

impl<M> MakeTransformation2<AllDomain<String>, MapDomain<AllDomain<Data>>, M, M, Option<&str>, usize> for SplitDataFrame<M>
    where M: Clone + Metric<Distance=u32> + DatasetMetric {
    fn construct(separator: Option<&str>, col_count: usize) -> Transformation<AllDomain<String>, MapDomain<AllDomain<Data>>, M, M> {
        let separator = separator.unwrap_or(",").to_owned();
        Transformation::new(
            AllDomain::new(),
            create_dataframe_domain(),
            move |arg: &String| -> DataFrame {
                split_dataframe(&separator, col_count, &arg)
            },
            M::new(),
            M::new(),
            |d_in: &u32, d_out: &u32| *d_out >= *d_in)
    }
}

pub struct ParseColumn<M, T> {
    metric: PhantomData<M>,
    data: PhantomData<T>,
}

fn replace_col(key: &str, df: &DataFrame, col: &Data) -> DataFrame {
    let mut df = df.clone();
    *df.get_mut(key).unwrap() = col.clone();
    df
}

fn parse_column<T>(key: &str, impute: bool, df: &DataFrame) -> DataFrame where
    T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
    let col = df.get(key).unwrap();
    let col = col.as_form();
    let col = vec_string_to_str(col);
    let col = parse_series::<T>(&col, impute);
    replace_col(key, &df, &col.into())
}

impl<M, T> MakeTransformation2<MapDomain<AllDomain<Data>>, MapDomain<AllDomain<Data>>, M, M, &str, bool> for ParseColumn<M, T>
    where M: Clone + Metric<Distance=u32> + DatasetMetric,
          T: 'static + Element + FromStr + Clone + Default + PartialEq,
          T::Err: Debug {
    fn construct(key: &str, impute: bool) -> Transformation<MapDomain<AllDomain<Data>>, MapDomain<AllDomain<Data>>, M, M> {
        let key = key.to_owned();
        Transformation::new(
            create_dataframe_domain(),
            create_dataframe_domain(),
            move |arg: &DataFrame| -> DataFrame {
                parse_column::<T>(&key, impute, arg)
            },
            M::new(),
            M::new(),
            |d_in: &u32, d_out: &u32| *d_out >= *d_in)
    }
}

pub struct SelectColumn<M, T> {
    metric: PhantomData<M>,
    data: PhantomData<T>,
}


impl<M, T> MakeTransformation1<MapDomain<AllDomain<Data>>, VectorDomain<AllDomain<T>>, M, M, &str> for SelectColumn<M, T>
    where M: Clone + Metric<Distance=u32> + DatasetMetric,
          T: 'static + Element + Clone + PartialEq {
    fn construct(key: &str) -> Transformation<MapDomain<AllDomain<Data>>, VectorDomain<AllDomain<T>>, M, M> {
        let key = key.to_owned();
        Transformation::new(
            create_dataframe_domain(),
            VectorDomain::new_all(),
            move |arg: &DataFrame| -> Vec<T> {
                let ret = arg.get(&key).expect("Missing dataframe column");
                let ret: &Vec<T> = ret.as_form();
                ret.clone()
            },
            M::new(),
            M::new(),
            |d_in: &u32, d_out: &u32| *d_out >= *d_in)
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
    where M: Clone + Metric<Distance=u32> + DatasetMetric {
    fn construct() -> Transformation<AllDomain<String>, VectorDomain<AllDomain<String>>, M, M> {
        Transformation::new(
            AllDomain::<String>::new(),
            VectorDomain::new_all(),
            |arg: &String| -> Vec<String> {
                arg.lines().map(|v| v.to_owned()).collect()
            },
            M::new(),
            M::new(),
            |d_in: &u32, d_out: &u32| *d_out >= *d_in)
    }
}

pub struct ParseSeries<T, M> {
    data: PhantomData<T>,
    metric: PhantomData<M>,
}

fn parse_series<T>(col: &Vec<&str>, default_on_error: bool) -> Vec<T> where
    T: FromStr + Default,
    T::Err: Debug {
    if default_on_error {
        col.into_iter().map(|e| e.parse().unwrap_or_else(|_| T::default())).collect()
    } else {
        col.into_iter().map(|e| e.parse().unwrap()).collect()
    }
}

impl<T, M> MakeTransformation1<VectorDomain<AllDomain<String>>, VectorDomain<AllDomain<T>>, M, M, bool> for ParseSeries<T, M>
    where M: Clone + Metric<Distance=u32> + DatasetMetric,
          T: FromStr + Default,
          T::Err: Debug {
    fn construct(impute: bool) -> Transformation<VectorDomain<AllDomain<String>>, VectorDomain<AllDomain<T>>, M, M> {
        Transformation::new(
            VectorDomain::new_all(),
            VectorDomain::new_all(),
            // move is necessary because it captures `impute`
            move |arg: &Vec<String>| -> Vec<T> {
                let arg = vec_string_to_str(arg);
                parse_series(&arg, impute)
            },
            M::new(),
            M::new(),
            |d_in: &u32, d_out: &u32| *d_out >= *d_in)
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
    where M: Clone + Metric<Distance=u32> + DatasetMetric {
    fn construct(separator: Option<&str>) -> Transformation<VectorDomain<AllDomain<String>>, VectorDomain<VectorDomain<AllDomain<String>>>, M, M> {
        let separator = separator.unwrap_or(",").to_owned();
        Transformation::new(
            VectorDomain::new_all(),
            VectorDomain::new(VectorDomain::new_all()),
            // move is necessary because it captures `separator`
            move |arg: &Vec<String>| -> Vec<Vec<String>> {
                let arg = vec_string_to_str(arg);
                let ret = split_records(&separator, &arg);
                ret.into_iter().map(vec_str_to_string).collect()
            },
            M::new(),
            M::new(),
            |d_in: &u32, d_out: &u32| *d_out >= *d_in)
    }
}


#[cfg(test)]
mod tests {
    use crate::core::{ChainTT};

    use super::*;
    use crate::dist::HammingDistance;

    #[test]
    fn test_make_create_dataframe() {
        let transformation = CreateDataFrame::<HammingDistance>::construct(2);
        let arg = vec![
            vec!["ant".to_owned(), "foo".to_owned()],
            vec!["bat".to_owned(), "bar".to_owned()],
            vec!["cat".to_owned(), "baz".to_owned()],
        ];
        let ret = transformation.function.eval(&arg);
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_split_dataframe() {
        let transformation = SplitDataFrame::<HammingDistance>::construct(None, 2);
        let arg = "ant, foo\nbat, bar\ncat, baz".to_owned();
        let ret = transformation.function.eval(&arg);
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_parse_column() {
        let transformation = ParseColumn::<HammingDistance, i32>::construct("1", true);
        let arg: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["1".to_owned(), "2".to_owned(), "".to_owned()])),
        ].into_iter().collect();
        let ret = transformation.function.eval(&arg);
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec![1, 2, 0])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_parse_columns() {
        let transformation0 = ParseColumn::<HammingDistance, i32>::construct("1", true);
        let transformation1 = ParseColumn::<HammingDistance, f64>::construct("2", true);
        let transformation = ChainTT::construct(&transformation1, &transformation0);
        let arg: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["1".to_owned(), "2".to_owned(), "3".to_owned()])),
            ("2".to_owned(), Data::new(vec!["1.1".to_owned(), "2.2".to_owned(), "3.3".to_owned()])),
        ].into_iter().collect();
        let ret = transformation.function.eval(&arg);
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec![1, 2, 3])),
            ("2".to_owned(), Data::new(vec![1.1, 2.2, 3.3])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_select_column() {
        let transformation = SelectColumn::<HammingDistance, String>::construct("1");
        let arg: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        let ret = transformation.function.eval(&arg);
        let expected = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        assert_eq!(ret, expected);
    }
}