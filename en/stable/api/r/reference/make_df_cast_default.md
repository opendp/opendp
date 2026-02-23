# df cast default constructor

Make a Transformation that casts the elements in a column in a dataframe
from type `TIA` to type `TOA`. If cast fails, fill with default.

## Usage

``` r
make_df_cast_default(input_domain, input_metric, column_name, .TIA, .TOA)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

- column_name:

  column name to be transformed

- .TIA:

  Atomic Input Type to cast from

- .TOA:

  Atomic Output Type to cast into

## Value

Transformation

## Details

|        |                  |
|--------|------------------|
| `TIA`  | `TIA::default()` |
| float  | `0.`             |
| int    | `0`              |
| string | `""`             |
| bool   | `false`          |

Required features: `contrib`

[make_df_cast_default in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_df_cast_default.html)

**Supporting Elements:**

- Input Domain: `DataFrameDomain<TK>`

- Output Domain: `M`

- Input Metric: `DataFrameDomain<TK>`

- Output Metric: `M`
