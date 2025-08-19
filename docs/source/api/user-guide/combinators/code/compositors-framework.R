# init
library(opendp)
enable_features("contrib")
# /init

# up-front
# define the dataset space and how distances are measured
input_space <- c(vector_domain(atom_domain(.T = i32)), symmetric_distance())

meas_count <- input_space |> then_count() |> then_laplace(scale = 1.0)
meas_sum <- input_space |>
  then_clamp(c(0L, 10L)) |>
  then_sum() |>
  then_laplace(scale = 5.0)
# /up-front


# nolint start
# print-up-front
meas_count
# Measurement(
#   input_domain=VectorDomain(AtomDomain(T=i32)),
#   input_metric=SymmetricDistance(),
#   output_measure=MaxDivergence
# )

meas_sum
# Measurement(
#   input_domain=VectorDomain(AtomDomain(T=i32)),
#   input_metric=SymmetricDistance(),
#   output_measure=MaxDivergence
# )
# /print-up-front
# nolint end


# non-adaptive-composition-init
meas_mean_fraction <- make_composition(c(meas_sum, meas_count))

int_dataset <- c(1L, 2L, 3L, 4L, 5L, 6L, 7L, 8L, 9L, 10L)
unlist(meas_mean_fraction(arg = int_dataset))
# [1] 52 10
# /non-adaptive-composition-init


# non-adaptive-composition-map
meas_mean_fraction(d_in = 1L)
# 3.0
# /non-adaptive-composition-map

# med-adaptive-composition-init
meas_adaptive_comp <- make_adaptive_composition(
  input_domain = vector_domain(atom_domain(.T = i32)),
  input_metric = symmetric_distance(),
  output_measure = max_divergence(),
  d_in = 1L,
  d_mids = c(2.0, 1.0)
)
# /med-adaptive-composition-init

# med-adaptive-composition-map
meas_adaptive_comp(d_in = 1L)
# 3.0
# /med-adaptive-composition-map

# med-adaptive-composition-invoke
int_dataset <- c(1L, 2L, 3L, 4L, 5L, 6L, 7L, 8L, 9L, 10L)
qbl_adaptive_comp <- meas_adaptive_comp(arg = int_dataset)
# /med-adaptive-composition-invoke


# med-adaptive-composition-query
qbl_adaptive_comp(query = meas_sum)
# 61
qbl_adaptive_comp(query = meas_count)
# 10
# /med-adaptive-composition-query

# fully-adaptive-composition
odom_fully_adaptive_comp <- make_fully_adaptive_composition(
  input_domain = vector_domain(atom_domain(.T = i32)),
  input_metric = symmetric_distance(),
  output_measure = max_divergence()
)
# /fully-adaptive-composition

# fully-adaptive-composition-invoke
int_dataset <- c(1L, 2L, 3L, 4L, 5L, 6L, 7L, 8L, 9L, 10L)
qbl_fully_adaptive_comp <- odom_fully_adaptive_comp(arg = int_dataset)
# /fully-adaptive-composition-invoke

# fully-adaptive-composition-loss1
qbl_fully_adaptive_comp(d_in = 1L)
# 0.0
# /fully-adaptive-composition-loss1

# fully-adaptive-composition-eval1
qbl_fully_adaptive_comp(query = meas_sum)
# 56
qbl_fully_adaptive_comp(query = meas_count)
# 10
# /fully-adaptive-composition-eval1

# fully-adaptive-composition-loss2
qbl_fully_adaptive_comp(d_in = 1L)
# 3.0
# /fully-adaptive-composition-loss2

# privacy-filter
meas_fully_adaptive_comp <- make_privacy_filter(
  odom_fully_adaptive_comp,
  d_in = 1L,
  d_out = 2.0
)
# /privacy-filter

# privacy-filter-invoke
int_dataset <- c(1L, 2L, 3L, 4L, 5L, 6L, 7L, 8L, 9L, 10L)
qbl_fully_adaptive_comp <- meas_fully_adaptive_comp(arg = int_dataset)
# /privacy-filter-invoke

# privacy-filter-eval1
qbl_fully_adaptive_comp(query = meas_count)
# 11
qbl_fully_adaptive_comp(query = meas_count)
# 9
# /privacy-filter-eval1

# privacy-filter-loss1
qbl_fully_adaptive_comp(d_in = 1L)
# 2.0
# /privacy-filter-loss1


# privacy-filter-eval2
tryCatch(
  qbl_fully_adaptive_comp(query = meas_count),
  error = print
)
# [FailedFunction] : insufficient privacy budget: 3.0 > 2.0
# /privacy-filter-eval2

# measurement-chaining1
str_space <- c(vector_domain(atom_domain(.T = String)), symmetric_distance())
meas_adaptive_comp_str <- str_space |>
  then_cast_default(i32) |>
  then_adaptive_composition(
    output_measure = max_divergence(),
    d_in = 1L,
    d_mids = c(2.0, 1.0)
  )

qbl_adaptive_comp_str <- meas_adaptive_comp_str(arg = c("1", "2", "3", "4", "5", "6", "7", "8", "9", "10"))
qbl_adaptive_comp_str(query = meas_sum)
# 69
qbl_adaptive_comp_str(query = meas_count)
# 10
# /measurement-chaining1

# measurement-chaining2
max_contributions <- 1L
sum_trans <- input_space |> then_clamp(c(0L, 10L)) |> then_sum()
meas_adaptive_comp <- sum_trans |>
  then_adaptive_composition(
    output_measure = max_divergence(),
    d_in = sum_trans(d_in = max_contributions),
    d_mids = c(2.0, 1.0)
  )
# /measurement-chaining2
