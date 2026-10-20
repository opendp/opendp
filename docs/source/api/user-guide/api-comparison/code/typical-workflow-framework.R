# unit-of-privacy
library(opendp)
enable_features("contrib")

d_in <- 1L # neighboring data set distance is at most d_in...
input_metric <- symmetric_distance() # ...in terms of additions/removals
input_domain <- vector_domain(atom_domain(.T = f64, nan = FALSE))

# /unit-of-privacy


# privacy-loss
d_out <- 1.0 # output distributions have distance at most d_out (ε)...
privacy_measure <- max_divergence() # ...in terms of pure-DP

# /privacy-loss


# public-info
bounds <- c(0.0, 100.0)
imputed_value <- 50.0

# /public-info


# mediate
data <- runif(100L, min = 0.0, max = 100.0)

m_sc <- make_adaptive_composition(
  input_domain = input_domain,
  input_metric = input_metric,
  output_measure = privacy_measure,
  d_in = d_in,
  d_mids = rep(d_out / 3L, 3L)
)

# Call measurement with data to create a queryable:
queryable <- m_sc(arg = data) # Different from Python, which does not require "arg".

# /mediate


# count
count_transformation <- (
  make_count(input_domain, input_metric)
)

count_sensitivity <- count_transformation(d_in = d_in) # Different from Python, which uses ".map".
cat("count_sensitivity:", count_sensitivity, "\n")
# 1

count_measurement <- binary_search_chain(
  function(scale) count_transformation |> then_laplace(scale), d_in, d_out / 3L
)
dp_count <- queryable(query = count_measurement) # Different from Python, which does not require "query".
cat("dp_count:", dp_count, "\n")
# /count


# mean
mean_transformation <- (
  make_clamp(input_domain, input_metric, bounds)
  |> then_resize(size = dp_count, constant = imputed_value)
  |> then_mean()
)

mean_measurement <- binary_search_chain(
  function(scale) mean_transformation |> then_laplace(scale), d_in, d_out / 3L
)

dp_mean <- queryable(query = mean_measurement) # Different from Python, which does not require "query".
cat("dp_mean:", dp_mean, "\n")
# /mean
