library(opendp)
options(warning.length = 8170L)

test_that("atom_domain", {
  domain <- atom_domain(.T = "i32")
  expect_equal(domain_carrier_type(domain), "i32")
  expect_equal(domain_debug(domain), "AtomDomain(T=i32)")

  domain <- atom_domain(bounds = list(1L, 2L))
})


test_that("vector_domain", {
  domain <- vector_domain(atom_domain(.T = "i32"))
  expect_true(member(domain, c(1L, 2L)))
  expect_equal(domain_carrier_type(domain), "Vec<i32>")
  expect_equal(domain_debug(domain), "VectorDomain(AtomDomain(T=i32))")

  domain <- vector_domain(atom_domain(bounds = list(1, 2)), size = 12L)
  expect_true(member(domain, rep(1, 12L)))
  expect_false(member(domain, rep(1, 11L)))
  expect_false(member(domain, rep(3, 12L)))
})

test_that("map_domain", {
  domain <- map_domain(atom_domain(.T = "String"), atom_domain(.T = "f64"))
  expect_true(member(domain, new_hashtab(c("A", "B"), c(3, 4))))
  expect_equal(domain_carrier_type(domain), "HashMap<String, f64>")
  expect_equal(domain_debug(domain), "MapDomain { key_domain: AtomDomain(T=String), value_domain: AtomDomain(nan=true, T=f64) }")
})
