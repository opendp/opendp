# Privacy measure with meaning defined by an OpenDP Library user (you).

Any two instances of UserDivergence are equal if their string
descriptors are equal.

## Usage

``` r
user_divergence(descriptor)
```

## Arguments

- descriptor:

  A string description of the privacy measure.

## Value

Measure

## Details

Required features: `honest-but-curious`

**Why honest-but-curious?:**

The essential requirement of a privacy measure is that it is closed
under postprocessing. Your privacy measure `D` must satisfy that, for
any pure function `f` and any two distributions `Y, Y'`, then \\(D(Y,
Y') \ge D(f(Y), f(Y'))\\).

Beyond this, you should also consider whether your privacy measure can
be used to provide meaningful privacy guarantees to your privacy units.
