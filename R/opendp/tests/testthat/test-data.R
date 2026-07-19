opendp::enable_features("contrib", "honest-but-curious")

scalar_space_of <- function(.T) c(atom_domain(.T = .T), discrete_distance())
vector_space_of <- function(.T) c(vector_domain(atom_domain(.T = .T)), symmetric_distance())
map_space_of <- function(.K, .V) c(map_domain(atom_domain(.T = .K), atom_domain(.T = .V)), symmetric_distance())

hashtab_eq <- function(x, y) {
  if (utils::numhash(x) != utils::numhash(y)) return(FALSE)
  equal <- TRUE
  utils::maphash(x, function(k, v) {
    equal <<- equal && identical(utils::gethash(y, k), v)
  })
  equal
}

test_that("vector data loaders", {
  expect_equal((vector_space_of(f64) |> then_identity())(arg = 1), 1)
  expect_equal((vector_space_of(i32) |> then_identity())(arg = 1L), 1L)
  expect_equal((vector_space_of(usize) |> then_identity())(arg = 1L), 1L)
  expect_true((vector_space_of(bool) |> then_identity())(arg = TRUE))
  expect_equal((vector_space_of(String) |> then_identity())(arg = "A"), "A")
})

test_that("scalar data loaders", {
  expect_equal((scalar_space_of(f64) |> then_identity())(arg = 1), 1)
  expect_equal((scalar_space_of(i32) |> then_identity())(arg = 1L), 1L)
  expect_equal((scalar_space_of(usize) |> then_identity())(arg = 1L), 1L)
  expect_true((scalar_space_of(bool) |> then_identity())(arg = TRUE))
  expect_equal((scalar_space_of(String) |> then_identity())(arg = "A"), "A")
})

test_that("hashmap data loaders", {
  h_String_bool <- new_hashtab(c("A", "B"), c(TRUE, FALSE))
  h_String_bool_out <- (map_space_of(String, bool) |> then_identity())(arg = h_String_bool)
  expect_true(hashtab_eq(h_String_bool, h_String_bool_out))

  h_String_f64 <- new_hashtab(c("A", "B"), c(23, 12))
  h_String_f64_out <- (map_space_of(String, f64) |> then_identity())(arg = h_String_f64)
  expect_true(hashtab_eq(h_String_f64, h_String_f64_out))

  expect_false(hashtab_eq(h_String_bool, h_String_f64))
})


if (Sys.getenv("OPENDP_TEST_RELEASE", unset = "false") != "false") {
  test_that("scalar release-mode data loaders", {
    expect_equal((scalar_space_of(f32) |> then_identity())(arg = 1), 1)
    expect_equal((scalar_space_of(u32) |> then_identity())(arg = 1L), 1L)
    expect_equal((scalar_space_of(i64) |> then_identity())(arg = 1L), 1L)
    expect_equal((scalar_space_of(u64) |> then_identity())(arg = 1L), 1L)
  })

  test_that("vector release-mode data loaders", {
    expect_equal((vector_space_of(f32) |> then_identity())(arg = 1), 1)
    expect_equal((vector_space_of(u32) |> then_identity())(arg = 1L), 1L)
    expect_equal((vector_space_of(i64) |> then_identity())(arg = 1L), 1L)
    expect_equal((vector_space_of(u64) |> then_identity())(arg = 1L), 1L)
  })

  test_that("hashmap release-mode data loaders", {
    h_i32_String <- new_hashtab(c(1L, 2L), c("A", "B"))
    h_i32_String_out <- (map_space_of(i32, String) | then_identity())(arg = h_i32_String)
    expect_true(hashtab_eq(h_i32_String, h_i32_String_out))

    h_bool_i32 <- new_hashtab(c(TRUE, FALSE), c(1L, 2L))
    h_bool_i32_out <- (map_space_of(bool, i32) | then_identity())(arg = h_bool_i32)
    expect_true(hashtab_eq(h_bool_i32, h_bool_i32_out))
  })
}

test_that("bitvec data loader", {
  input_space <- c(bitvector_domain(max_weight = 4L), discrete_distance())
  m_ident <- input_space |> then_identity()

  expect_equal(m_ident(arg = charToRaw("")), charToRaw(""))
  expect_equal(m_ident(arg = charToRaw("abc")), charToRaw("abc"))
})
