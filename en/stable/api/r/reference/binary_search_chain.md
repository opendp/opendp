# Find the highest-utility (`d_in`, `d_out`)-close Transformation or Measurement.

Searches for the numeric parameter to `make_chain` that results in a
computation that most tightly satisfies `d_out` when datasets differ by
at most `d_in`, then returns the Transformation or Measurement
corresponding to said parameter.

## Usage

``` r
binary_search_chain(make_chain, d_in, d_out, bounds = NULL, .T = NULL)
```

## Arguments

- make_chain:

  a function that takes a number and returns a Transformation or
  Measurement

- d_in:

  how far apart input datasets can be

- d_out:

  how far apart output datasets or distributions can be

- bounds:

  a 2-tuple of the lower and upper bounds on the input of `make_chain`

- .T:

  type of argument to `make_chain`, either "float" or "int"

## Value

a Transformation or Measurement (chain) that is (`d_in`, `d_out`)-close.

## Details

See `binary_search_param` to retrieve the discovered parameter instead
of the complete computation chain.

## Examples

``` r
enable_features("contrib")
# create a sum transformation over the space of float vectors
s_vec <- c(vector_domain(atom_domain(.T = "float", nan = FALSE)), symmetric_distance())
#> Error in .Call("domains__atom_domain", bounds, nan, .T, rt_parse(.T.bounds),     rt_parse(.T.nan), log_, PACKAGE = "opendp"): "domains__atom_domain" not available for .Call() for package "opendp"
t_sum <- s_vec |> then_clamp(c(0., 1.)) |> then_sum()
#> Error: object 's_vec' not found

# find a measurement that satisfies epsilon = 1 when datasets differ by at most one record
m_sum <- binary_search_chain(\(s) t_sum |> then_laplace(s), d_in = 1L, d_out = 1.)
#> Error: unable to infer type `.T`; pass the type `.T` or bounds
```
