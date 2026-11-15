# unit-of-privacy
library(opendp)
enable_features("contrib", "idealized-numerics")

privacy_unit <- unit_of(contributions = 1L)

# /unit-of-privacy


# privacy-loss
privacy_loss <- loss_of(epsilon = 1.0)

# /privacy-loss


# public-info
bounds <- c(0.0, 100.0)
imputed_value <- 50.0

# /public-info


# mediate
data <- runif(100L, min = 0.0, max = 100.0)

context <- Context$compositor(
  data = data,
  privacy_unit = privacy_unit,
  privacy_loss = privacy_loss,
  split_evenly_over = 3L
)

# /mediate


# count
count_query <- query(context) |>
  then_count() |>
  then_laplace(auto())

scale <- param(count_query, .T = "float")
accuracy <- discrete_laplacian_scale_to_accuracy(
  scale = scale,
  alpha = 0.05
)

dp_count <- release(count_query)
confidence_interval <- c(
  dp_count - accuracy,
  dp_count + accuracy
)

# /count


# mean
mean_query <- query(context) |>
  then_impute_constant(imputed_value) |>
  then_clamp(bounds) |>
  then_resize(size = dp_count, constant = imputed_value) |>
  then_mean() |>
  then_laplace(auto())

dp_mean <- release(mean_query)

# /mean
