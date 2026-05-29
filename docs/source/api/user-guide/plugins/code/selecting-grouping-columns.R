library(opendp)

enable_features("honest-but-curious", "contrib")


# plugins
make_grouping_cols_score <- function(
  input_domain,
  input_metric,
  candidates,
  min_bin_contributions
) {
  score <- function(x, cols) {
    key <- if (length(cols) == 1L) {
      x[[cols]]
    } else {
      interaction(x[, cols, drop = FALSE], drop = TRUE, lex.order = TRUE)
    }
    sum(table(key) >= min_bin_contributions)
  }

  make_user_transformation(
    input_domain = input_domain,
    input_metric = input_metric,
    output_domain = vector_domain(atom_domain(.T = f64, nan = FALSE)),
    output_metric = linf_distance(.T = f64, monotonic = TRUE),
    function_ = function(x) {
      vapply(candidates, function(cols) score(x, cols), numeric(1L))
    },
    stability_map = function(d_in) as.numeric(d_in)
  )
}
then_grouping_cols_score <- to_then(make_grouping_cols_score)

make_select_grouping_cols <- function(
  input_domain,
  input_metric,
  candidates,
  min_bin_size,
  scale
) {
  c(input_domain, input_metric) |>
    then_grouping_cols_score(candidates, min_bin_size) |>
    then_noisy_max(max_divergence(), scale = scale) |>
    then_postprocess(function(idx) candidates[[idx + 1L]])
}
then_select_grouping_cols <- to_then(make_select_grouping_cols)
# /plugins


# dp-mechanism
row_count <- 50L
col_count <- 4L
private_data <- data.frame(
  setNames(
    replicate(
      col_count, sample(0L:1L, row_count, replace = TRUE), simplify = FALSE
    ),
    paste0("too_uniform_", seq_len(col_count) - 1L)
  ),
  setNames(
    replicate(
      col_count,
      sample(0L:row_count, row_count, replace = TRUE),
      simplify = FALSE
    ),
    paste0("too_diverse_", seq_len(col_count) - 1L)
  ),
  setNames(
    replicate(
      col_count,
      sample(0L:20L, row_count, replace = TRUE),
      simplify = FALSE
    ),
    paste0("just_right_", seq_len(col_count) - 1L)
  ),
  check.names = FALSE
)

powerset <- function(values) {
  unlist(
    lapply(seq_along(values), function(k) combn(values, k, simplify = FALSE)),
    recursive = FALSE
  )
}

candidates <- powerset(colnames(private_data))

input_space <- c(
  user_domain(
    identifier = "DataFrameDomain",
    member = function(x) is.data.frame(x)
  ),
  symmetric_distance()
)
m_select_gcols <- input_space |>
  then_select_grouping_cols(
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
