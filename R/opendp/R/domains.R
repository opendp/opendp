# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' Construct an instance of `AtomDomain`.
#'
#' [atom_domain in Rust documentation.](https://docs.rs/opendp/0.12.1-beta.20250220.1/opendp/domains/fn.atom_domain.html)
#'
#' @concept domains
#' @param bounds undocumented
#' @param nullable undocumented
#' @param .T The type of the atom.
#' @return Domain
#' @examples
#' atom_domain(.T = "i32")
#' @export
atom_domain <- function(
  bounds = NULL,
  nullable = FALSE,
  .T = NULL
) {
  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = get_first(bounds))
  .T.bounds <- new_runtime_type(origin = "Option", args = list(new_runtime_type(origin = "Tuple", args = list(.T, .T))))

  log_ <- new_constructor_log("atom_domain", "domains", new_hashtab(
    list("bounds", "nullable", "T"),
    list(bounds, unbox2(nullable), .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))
  rt_assert_is_similar(expected = bool, inferred = rt_infer(nullable))

  # Call wrapper function.
  output <- .Call(
    "domains__atom_domain",
    bounds, nullable, .T, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}


#' Construct an instance of `BitVectorDomain`.
#'
#' @concept domains
#' @param max_weight The maximum number of positive bits.
#' @return Domain
#' @export
bitvector_domain <- function(
  max_weight = NULL
) {
  # Standardize type arguments.
  .T.max_weight <- new_runtime_type(origin = "Option", args = list(u32))

  log_ <- new_constructor_log("bitvector_domain", "domains", new_hashtab(
    list("max_weight"),
    list(max_weight)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.max_weight, inferred = rt_infer(max_weight))

  # Call wrapper function.
  output <- .Call(
    "domains__bitvector_domain",
    max_weight, rt_parse(.T.max_weight),
    log_, PACKAGE = "opendp")
  output
}


#' Get the carrier type of a `domain`.
#'
#' @concept domains
#' @param this The domain to retrieve the carrier type from.
#' @return str
#' @export
domain_carrier_type <- function(
  this
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("domain_carrier_type", "domains", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "domains__domain_carrier_type",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Debug a `domain`.
#'
#' @concept domains
#' @param this The domain to debug (stringify).
#' @return str
#' @export
domain_debug <- function(
  this
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("domain_debug", "domains", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "domains__domain_debug",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Get the type of a `domain`.
#'
#' @concept domains
#' @param this The domain to retrieve the type from.
#' @return str
#' @export
domain_type <- function(
  this
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("domain_type", "domains", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "domains__domain_type",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Construct an instance of `MapDomain`.
#'
#' @concept domains
#' @param key_domain domain of keys in the hashmap
#' @param value_domain domain of values in the hashmap
#' @return Domain
#' @export
map_domain <- function(
  key_domain,
  value_domain
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("map_domain", "domains", new_hashtab(
    list("key_domain", "value_domain"),
    list(key_domain, value_domain)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = AnyDomain, inferred = rt_infer(key_domain))
  rt_assert_is_similar(expected = AnyDomain, inferred = rt_infer(value_domain))

  # Call wrapper function.
  output <- .Call(
    "domains__map_domain",
    key_domain, value_domain,
    log_, PACKAGE = "opendp")
  output
}


#' Check membership in a `domain`.
#'
#' @concept domains
#' @param this The domain to check membership in.
#' @param val A potential element of the domain.
#' @export
member <- function(
  this,
  val
) {
  # Standardize type arguments.
  .T.val <- domain_carrier_type(this)

  log_ <- new_constructor_log("member", "domains", new_hashtab(
    list("this", "val"),
    list(this, val)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = AnyDomain, inferred = rt_infer(this))
  rt_assert_is_similar(expected = .T.val, inferred = rt_infer(val))

  # Call wrapper function.
  output <- .Call(
    "domains__member",
    this, val, rt_parse(.T.val),
    log_, PACKAGE = "opendp")
  output
}


#' Construct an instance of `OptionDomain`.
#'
#' [option_domain in Rust documentation.](https://docs.rs/opendp/0.12.1-beta.20250220.1/opendp/domains/fn.option_domain.html)
#'
#' @concept domains
#' @param element_domain undocumented
#' @param .D The type of the inner domain.
#' @return Domain
#' @export
option_domain <- function(
  element_domain,
  .D = NULL
) {
  # Standardize type arguments.
  .D <- parse_or_infer(type_name = .D, public_example = element_domain)

  log_ <- new_constructor_log("option_domain", "domains", new_hashtab(
    list("element_domain", "D"),
    list(element_domain, .D)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .D, inferred = rt_infer(element_domain))

  # Call wrapper function.
  output <- .Call(
    "domains__option_domain",
    element_domain, .D,
    log_, PACKAGE = "opendp")
  output
}


#' Construct an instance of `VectorDomain`.
#'
#' @concept domains
#' @param atom_domain The inner domain.
#' @param size undocumented
#' @return Domain
#' @export
vector_domain <- function(
  atom_domain,
  size = NULL
) {
  # Standardize type arguments.
  .T.size <- new_runtime_type(origin = "Option", args = list(i32))

  log_ <- new_constructor_log("vector_domain", "domains", new_hashtab(
    list("atom_domain", "size"),
    list(atom_domain, size)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.size, inferred = rt_infer(size))

  # Call wrapper function.
  output <- .Call(
    "domains__vector_domain",
    atom_domain, size, rt_parse(.T.size),
    log_, PACKAGE = "opendp")
  output
}
