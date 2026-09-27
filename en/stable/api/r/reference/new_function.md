# Construct a Function from a user-defined callback. Can be used to build a postprocessor.

Required features: `contrib`, `honest-but-curious`

## Usage

``` r
new_function(function_, .TO)
```

## Arguments

- function\_:

  A function mapping data to a value of type `TO`

- .TO:

  Output Type

## Value

Function

## Details

[new_function in Rust
documentation.](https://docs.rs/opendp/0.15.1/opendp/core/struct.Function.html)

**Why honest-but-curious?:**

An OpenDP `function` must satisfy two criteria. These invariants about
functions are necessary to show correctness of other algorithms.

First, `function` must not use global state. For instance, a
postprocessor that accesses the system clock time can be used to build a
measurement that reveals elapsed execution time, which escalates a
side-channel vulnerability into a direct vulnerability.

Secondly, `function` must only raise data-independent exceptions. For
instance, raising an exception with the value of a DP release will both
reveal the DP output and cancel the computation, potentially avoiding
privacy accounting.
