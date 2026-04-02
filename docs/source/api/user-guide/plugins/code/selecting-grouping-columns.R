library(opendp)

enable_features("honest-but-curious", "contrib")


# plugins
make_grouping_cols_score <- function(candidates, min_bin_contributions) {
  score <- function(x, cols) {
    key <- if (length(cols) == 1) {
      x[[cols]]
    } else {
      interaction(x[, cols, drop = FALSE], drop = TRUE, lex.order = TRUE)
    }
    sum(table(key) >= min_bin_contributions)
  }

  make_user_transformation(
    input_domain = user_domain(
      identifier = "DataFrameDomain",
      member = function(x) is.data.frame(x)
    ),
    input_metric = symmetric_distance(),
    output_domain = vector_domain(atom_domain(.T = f64, nan = FALSE)),
    output_metric = linf_distance(.T = f64, monotonic = TRUE),
    function_ = function(x) vapply(candidates, function(cols) score(x, cols), numeric(1)),
    stability_map = function(d_in) as.numeric(d_in)
  )
}

make_select_grouping_cols <- function(candidates, min_bin_size, scale) {
  make_grouping_cols_score(candidates, min_bin_size) >>
    then_noisy_max(max_divergence(), scale = scale) >>
    new_function(
      function_ = function(idx) candidates[[idx + 1L]],
      .TO = "ExtrinsicObject"
    )
}
# /plugins


# dp-mechanism
row_count <- 50L
col_count <- 4L
private_data <- data.frame(
  setNames(replicate(col_count, sample(0:1, row_count, replace = TRUE), simplify = FALSE), paste0("too_uniform_", seq_len(col_count) - 1L)),
  setNames(replicate(col_count, sample(0:row_count, row_count, replace = TRUE), simplify = FALSE), paste0("too_diverse_", seq_len(col_count) - 1L)),
  setNames(replicate(col_count, sample(0:20, row_count, replace = TRUE), simplify = FALSE), paste0("just_right_", seq_len(col_count) - 1L)),
  check.names = FALSE
)

powerset <- function(values) {
  unlist(lapply(seq_along(values), function(k) combn(values, k, simplify = FALSE)), recursive = FALSE)
}

candidates <- powerset(colnames(private_data))

m_select_gcols <- make_select_grouping_cols(
  candidates = candidates,
  min_bin_size = 89L,
  scale = 10.
)

m_select_gcols(d_in = 1L)
# /dp-mechanism


# dp-release
dp_selected_grouping_columns <- m_select_gcols(arg = private_data)
dp_selected_grouping_columns
# /dp-release
