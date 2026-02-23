# Construct an instance of `AtomDomain`.

The domain defaults to unbounded if `bounds` is `None`, If `T` is float,
`nan` defaults to `true`.

## Usage

``` r
atom_domain(bounds = NULL, nan = NULL, .T = NULL)
```

## Arguments

- bounds:

  Optional bounds of elements in the domain, if the data type is
  numeric.

- nan:

  Whether the domain may contain NaN, if the data type is float.

- .T:

  The type of the atom.

## Value

AtomDomain

## Details

[atom_domain in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/domains/struct.AtomDomain.html)

## Examples

``` r
atom_domain(.T = "i32")
#> Error in .Call("domains__atom_domain", bounds, nan, .T, rt_parse(.T.bounds),     rt_parse(.T.nan), log_, PACKAGE = "opendp"): "domains__atom_domain" not available for .Call() for package "opendp"
```
