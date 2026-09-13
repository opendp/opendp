# Construct a new UserDomain. Any two instances of an UserDomain are equal if their string descriptors are equal. Contains a function used to check if any value is a member of the domain.

Required features: `honest-but-curious`

## Usage

``` r
user_domain(identifier, member, descriptor = NULL)
```

## Arguments

- identifier:

  A string description of the data domain.

- member:

  A function used to test if a value is a member of the data domain.

- descriptor:

  Additional constraints on the domain.

## Value

Domain

## Details

**Why honest-but-curious?:**

The identifier must uniquely identify this domain. If the identifier is
not uniquely identifying, then two different domains with the same
identifier will chain, which can violate transformation stability.

In addition, the member function must:

1.  be a pure function

2.  be sound (only return true if its input is a member of the domain).
