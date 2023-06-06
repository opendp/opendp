library(opendpbase)

test_that("atom_domain", {
    domain <- atom_domain(.T = "i32")
    expect_equal(domain_carrier_type(domain), "i32")
    expect_equal(domain_debug(domain), "AtomDomain(T=i32)")

    domain <- atom_domain(bounds = list(1L, 2L))
})