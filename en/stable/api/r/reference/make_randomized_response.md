# randomized response constructor

Make a Measurement that implements randomized response on a categorical
value.

## Usage

``` r
make_randomized_response(categories, prob, .T = NULL)
```

## Arguments

- categories:

  Set of valid outcomes

- prob:

  Probability of returning the correct answer. Must be in
  `[1/num_categories, 1]`

- .T:

  Data type of a category.

## Value

Measurement

## Details

Required features: `contrib`

[make_randomized_response in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_randomized_response.html)

**Supporting Elements:**

- Input Domain: `AtomDomain<T>`

- Output Type: `DiscreteDistance`

- Input Metric: `MaxDivergence`

- Output Measure: `T`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/measurements/randomized_response/make_randomized_response.pdf)
