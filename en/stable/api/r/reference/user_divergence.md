# Privacy measure with meaning defined by an OpenDP Library user (you).

Any two instances of UserDivergence are equal if their descriptors
compare equal.

## Usage

``` r
user_divergence(identifier, descriptor = NULL)
```

## Arguments

- identifier:

  A string description of the privacy measure.

- descriptor:

  Additional constraints on the privacy measure.

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
