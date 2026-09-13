# partial randomized response constructor

See documentation for
[`make_randomized_response()`](https://docs.opendp.org/reference/make_randomized_response.md)
for details.

## Usage

``` r
then_randomized_response(lhs, categories, prob, .T = NULL)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- categories:

  Set of valid outcomes

- prob:

  Probability of returning the correct answer. Must be in
  `[1/num_categories, 1]`

- .T:

  Data type of a category.

## Value

Measurement
