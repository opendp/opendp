# nolint start: unnecessary_concatenation_linter.
ATOM_EQUIVALENCE_CLASSES <- list(
  i32 = c("u32", "u64", "i32", "i64", "usize"),
  f64 = c("f32", "f64"),
  bool = c("bool"),
  AnyMeasurement = c("AnyMeasurementPtr", "AnyMeasurement"),
  AnyTransformation = c("AnyTransformationPtr")
)
# nolint end

RUST_TO_R <- list(
  u32 = "integer",
  u64 = "integer",
  i32 = "integer",
  i64 = "integer",
  usize = "integer",
  f32 = "numeric",
  f64 = "numeric",
  bool = "logical",
  String = "character"
)
R_TO_RUST <- list(
  integer = "i32",
  numeric = "f64",
  logical = "bool",
  character = "String",
  domain = "AnyDomain",
  metric = "AnyMetric",
  measure = "AnyMeasure",
  measurement = "AnyMeasurement",
  transformation = "AnyTransformation",
  `function` = "AnyFunction"
)

as_rt_vec <- function(atom_type) {
  new_runtime_type("Vec", list(atom_type))
}

rt_infer <- function(public_example) {
  prospect <- R_TO_RUST[[class(public_example)]]
  if (!is.null(prospect)) {
    runtime_type <- new_runtime_type(prospect)
    if (length(public_example) > 1) {
      runtime_type <- as_rt_vec(runtime_type)
    }
    return(runtime_type)
  }
  if (utils::is.hashtab(public_example)) {
    keys <- NULL
    vals <- NULL
    utils::maphash(public_example, function(k, v) {
      keys <<- c(keys[1], k)
      vals <<- c(vals[1], v)
    })
    .K <- new_runtime_type(R_TO_RUST[[class(keys)]])
    .V <- new_runtime_type(R_TO_RUST[[class(vals)]])
    return(new_runtime_type("HashMap", list(.K, .V)))
  }
  if (is.list(public_example)) {
    return(new_runtime_type("Tuple", lapply(public_example, rt_infer)))
  }
  if (is.null(public_example)) {
    return(new_runtime_type("Option", list(new_runtime_type(is_unknown = TRUE))))
  }

  stop("unrecognized type: ", class(public_example))
}

# nolint start: cyclocomp_linter
rt_parse <- function(type_name, generics = list()) {
  if (inherits(type_name, "runtime_type")) {
    return(type_name)
  }
  if (!is.character(type_name) || length(type_name) != 1) {
    stop("type_name must be a character string")
  }

  type_name <- trimws(type_name)
  # so that you can substitute concrete types
  if (type_name %in% generics) {
    return(new_runtime_type(type_name, is_generic = TRUE))
  }

  # parsing for (A, B, C) tuples
  if (startsWith(type_name, "(") && endsWith(type_name, ")")) {
    args <- parse_args_(substring(type_name, 2, nchar(type_name) - 1), generics)
    return(new_runtime_type("Tuple", args))
  }

  # parsing for A<B, C> generics
  left <- unlist(gregexpr(pattern = "<", type_name))[1]
  right <- unlist(gregexpr(pattern = ">", type_name))
  right <- right[[length(right)]]

  return(if (left == -1) {
    if (type_name == "int") {
      type_name <- "i32"
    } else if (type_name == "float") {
      type_name <- "f64"
    }
    # base case
    new_runtime_type(type_name)
  } else {
    # recursive case
    origin <- if (left > 0) {
      substring(type_name, 1, left - 1)
    } else {
      type_name
    }
    args <- parse_args_(substring(type_name, left + 1, right - 1), generics)
    new_runtime_type(origin, args)
  })
}
# nolint end

parse_args_ <- function(args, generics = list()) {
  args <- strsplit(args, ",\\s*(?![^()<>]*\\))", perl = TRUE)
  return(lapply(args[[1]], function(x) rt_parse(x, generics))) # nolint: unnecessary_lambda_linter.
}

new_runtime_type <- function(origin = NULL, args = NULL, is_generic = FALSE, is_unknown = FALSE) {
  rt <- list(origin = origin, args = args, is_generic = is_generic, is_unknown = is_unknown)
  class(rt) <- "runtime_type"
  rt
}

rt_to_string <- function(rt) {
  if (is.character(rt)) {
    return(rt)
  }
  if (rt$is_generic) {
    return(paste0(".", rt$origin))
  }
  if (rt$is_unknown) {
    return("?")
  }

  if (is.null(rt$args)) {
    return(rt$origin)
  }

  args <- toString(lapply(rt$args, function(v) {
    if (is.list(v)) {
      rt_to_string(v)
    } else {
      v
    }
  }))

  if (rt$origin == "Tuple") {
    return(paste0("(", args, ")"))
  }
  paste0(rt$origin, "<", args, ">")
}

#' @export
print.runtime_type <- function(x, ...) print(rt_to_string(x), ...)

# nolint start: cyclocomp_linter
rt_assert_is_similar <- function(expected, inferred) {
  ERROR_URL_298 <- "https://github.com/opendp/opendp/discussions/298"

  if (!inherits(expected, "runtime_type")) {
    expected <- rt_parse(expected)
  }
  if (!inherits(inferred, "runtime_type")) {
    inferred <- rt_parse(inferred)
  }

  is_option <- function(rt) rt$origin == "Option"
  if (is_option(expected)) {
    expected <- expected$args[[1]]
    if (is_option(inferred)) {
      if (inferred$args[[1]]$is_unknown) {
        return()
      } else {
        inferred <- inferred$args[[1]]
      }
    }
  }

  if (is.null(inferred$args) && expected$origin == "Vec") {
    inferred <- new_runtime_type("Vec", list(inferred))
  }

  if (is.null(expected$args) && is.null(inferred$args)) {
    # if both are primitive

    if (inferred$origin %in% names(ATOM_EQUIVALENCE_CLASSES)) {
      if (!(expected$origin %in% ATOM_EQUIVALENCE_CLASSES[[inferred$origin]])) {
        stop(paste0("inferred type is ", rt_to_string(inferred), ", expected ", rt_to_string(expected), ". See ", ERROR_URL_298))
      }
    } else if (expected$origin == inferred$origin) {
      return()
    } else {
      stop(paste0("inferred type is ", rt_to_string(inferred), ", expected ", rt_to_string(expected), ". See ", ERROR_URL_298))
    }
  } else if (!is.null(expected$args) && !is.null(inferred$args)) {
    if (expected$origin == "Vec" && inferred$origin == "Tuple") {
      if (length(unique(inferred$args)) == 1) {
        rt_assert_is_similar(expected$args[[1]], inferred$args[[1]])
        return()
      }
    }
    if (expected$origin == "Tuple" && inferred$origin == "Vec") {
      if (length(unique(expected$args)) == 1) {
        rt_assert_is_similar(expected$args[[1]], inferred$args[[1]])
        return()
      }
    }
    if (expected$origin != inferred$origin) {
      stop(paste0("inferred type is ", inferred$origin, ", expected ", expected$origin, ". See ", ERROR_URL_298))
    }
    if (length(expected$args) != length(inferred$args)) {
      stop(paste0("inferred type has ", length(inferred$args), " arg(s), expected ", length(expected$args), " arg(s). See ", ERROR_URL_298))
    }

    for (pair in mapply(list, expected$args, inferred$args, SIMPLIFY = FALSE)) {
      rt_assert_is_similar(pair[[1]], pair[[2]])
    }
  } else {
    # inferred type differs in structure
    stop(paste0("inferred type is ", rt_to_string(inferred), ", expected ", rt_to_string(expected), ". See ", ERROR_URL_298))
  }
}
# nolint end

rt_substitute <- function(rt, ...) {
  generics <- list(...)
  if (rt$is_generic) {
    if (rt$origin %in% names(generics)) {
      return(generics[[rt$origin]])
    } else {
      return(rt)
    }
  }

  if (is.null(rt$args)) {
    return(rt)
  }

  args <- lapply(rt$args, function(arg) rt_substitute(arg, ...)) # nolint: unnecessary_lambda_linter.
  new_runtime_type(rt$origin, args)
}

get_atom <- function(type_name) {
  if (!inherits(type_name, "runtime_type")) {
    type_name <- rt_parse(type_name)
  }

  if (is.list(type_name$args)) {
    return(get_atom(type_name$args[[1]]))
  }

  if (type_name$is_generic || type_name$is_unknown) {
    return(NULL)
  }
  type_name
}

get_atom_or_infer <- function(type_name, example) {
  atom <- get_atom(type_name)
  if (is.null(atom)) {
    return(rt_infer(example))
  }
  atom
}

get_first <- function(x) {
  if (is.null(x) || length(x) == 0) {
    NULL
  } else {
    x[[1]]
  }
}

parse_or_infer <- function(type_name, public_example) {
  if (!is.null(type_name)) {
    return(rt_parse(type_name))
  }
  if (!is.null(public_example)) {
    return(rt_infer(public_example))
  }

  stop("either type_name or public_example must be passed")
}

pass_through <- function(x) x

get_carrier_type <- function(value) rt_parse(value("carrier_type"))
get_distance_type <- function(value) rt_parse(value("distance_type"))
get_type <- function(value) rt_parse(value("type"))
get_value_type <- function(value) rt_parse(value("type"))$args[[2]]

#' type signature for an 8-bit signed integer
#'
#' @concept typing
#' @export
i8 <- new_runtime_type("i8")

#' type signature for a 16-bit signed integer
#'
#' @concept typing
#' @export
i16 <- new_runtime_type("i16")

#' type signature for a 32-bit signed integer
#'
#' @concept typing
#' @export
i32 <- new_runtime_type("i32")

#' type signature for a 64-bit signed integer
#'
#' @concept typing
#' @export
i64 <- new_runtime_type("i64")

#' type signature for a 128-bit signed integer
#'
#' @concept typing
#' @export
i128 <- new_runtime_type("i128")

#' type signature for an 8-bit unsigned integer
#'
#' @concept typing
#' @export
u8 <- new_runtime_type("u8")

#' type signature for a 16-bit unsigned integer
#'
#' @concept typing
#' @export
u16 <- new_runtime_type("u16")

#' type signature for a 32-bit unsigned integer
#'
#' @concept typing
#' @export
u32 <- new_runtime_type("u32")

#' type signature for a 64-bit unsigned integer
#'
#' @concept typing
#' @export
u64 <- new_runtime_type("u64")

#' type signature for a 128-bit unsigned integer
#'
#' @concept typing
#' @export
u128 <- new_runtime_type("u128")

#' type signature for a pointer-sized unsigned integer
#'
#' @concept typing
#' @export
usize <- new_runtime_type("usize")

#' type signature for a 32-bit floating point number
#'
#' @concept typing
#' @export
f32 <- new_runtime_type("f32")

#' type signature for a 64-bit floating point number
#'
#' @concept typing
#' @export
f64 <- new_runtime_type("f64")

#' type signature for a string
#'
#' @concept typing
#' @export
String <- new_runtime_type("String")

#' type signature for a boolean
#'
#' @concept typing
#' @export
bool <- new_runtime_type("bool")

AnyMeasurementPtr <- new_runtime_type("AnyMeasurementPtr")
AnyMeasurement <- AnyMeasurementPtr
AnyTransformationPtr <- new_runtime_type("AnyTransformationPtr")
AnyDomain <- new_runtime_type("AnyDomain")
AnyMetric <- new_runtime_type("AnyMetric")
AnyMeasure <- new_runtime_type("AnyMeasure")
