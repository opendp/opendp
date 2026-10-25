# Construct a PrivacyProfile from a user-defined callback.

Required features: `contrib`, `honest-but-curious`

## Usage

``` r
new_privacy_profile(curve)
```

## Arguments

- curve:

  A privacy curve mapping epsilon to delta

## Details

**Why honest-but-curious?:**

The privacy profile should implement a well-defined
\\(\delta(\epsilon)\\) curve:

- monotonically decreasing

- rejects epsilon values that are less than zero or nan

- returns delta values only within \\(\[0, 1\]\\)
