# unit-of-privacy
library(opendp)
enable_features("contrib")

d_in <- 1L # neighboring data set distance is at most d_in...
input_metric <- symmetric_distance() # ...in terms of additions/removals

input_domain <- vector_domain(atom_domain(.T = i32))

# /unit-of-privacy


# privacy-loss
d_out <- 1.0 # output distributions have distance at most d_out (ε)...
privacy_measure <- max_divergence() # ...in terms of pure-DP

# /privacy-loss

# mediate
data <- sample.int(100L, size = 100L, replace = TRUE)

o_ac <- make_fully_adaptive_composition(
  input_domain = input_domain,
  input_metric = input_metric,
  output_measure = privacy_measure
)

m_ac <- make_privacy_filter(
  odometer = o_ac,
  d_in = d_in,
  d_out = d_out
)

# Call measurement with data to create a queryable:
queryable <- m_ac(arg = data) # Different from Python, which does not require "arg".

# /mediate


# count

make_dp_count <- function(scale) {
  c(input_domain, input_metric) |> then_count() |> then_laplace(scale)
}

scale <- binary_search_param(
  make_dp_count, d_in, d_out / 3L, .T = "float"
)

accuracy <- discrete_laplacian_scale_to_accuracy(
  scale = scale, alpha = 0.05
)
accuracy
# 9.445721638273584

# (with 100(1-alpha) = 95% confidence, the estimated value will differ
#    from the true value by no more than ~9)

# Different from Python, which does not require "query = ".
dp_count <- queryable(query = make_dp_count(scale))

confidence_interval <- c(
  dp_count - accuracy,
  dp_count + accuracy
)
# /count

# public-info
bounds <- c(0L, 100L)

# /public-info


# sum
sum_transformation <- (
  c(input_domain, input_metric)
  |> then_clamp(bounds)
  |> then_sum()
)

sum_measurement <- binary_search_chain(
  \(scale) sum_transformation |> then_laplace(scale), d_in, d_out / 3L
)

# Different from Python, which does not require "query = ".
dp_sum <- queryable(query = sum_measurement)
cat("dp_sum:", dp_sum, "\n")
# /sum
