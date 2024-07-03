# unit-of-privacy
library(opendp)
enable_features("contrib")

d_in <- 1L # neighboring data set distance is at most d_in...
input_metric <- symmetric_distance() # ...in terms of additions/removals

# /unit-of-privacy


# privacy-loss
d_out <- 1.0 # output distributions have distance at most d_out (ε)...
privacy_measure <- max_divergence() # ...in terms of pure-DP

# /privacy-loss


# public-info
col_names <- c(
  "name", "sex", "age", "maritalStatus", "hasChildren", "highestEducationLevel",
  "sourceOfStress", "smoker", "optimism", "lifeSatisfaction", "selfEsteem"
)

# /public-info


# mediate
temp_file <- "teacher_survey.csv"
download.file("https://raw.githubusercontent.com/opendp/opendp/sydney/teacher_survey.csv", temp_file)
data_string <- paste(readLines(temp_file), collapse = "\n")
file.remove(temp_file)

m_sc <- make_sequential_composition(
  input_domain = atom_domain(.T = String),
  input_metric = input_metric,
  output_measure = privacy_measure,
  d_in = d_in,
  d_mids = rep(d_out / 3L, 3L)
)

# Call measurement with data to create a queryable:
qbl_sc <- m_sc(arg = data_string) # Different from Python, which does not require "arg".

# /mediate


# count
count_transformation <- (
  make_split_dataframe(",", col_names = col_names)
  |> then_select_column("age", String) # Different from Python, which uses "make_".
  |> then_count()
)

count_sensitivity <- count_transformation(d_in = d_in) # Different from Python, which uses ".map".
cat("count_sensitivity:", count_sensitivity, "\n")
# 1

count_measurement <- binary_search_chain(
  function(scale) count_transformation |> then_laplace(scale), d_in, d_out / 3L
)
dp_count <- qbl_sc(query = count_measurement) # Different from Python, which does not require "query".
cat("dp_count:", dp_count, "\n")
# /count


# mean
mean_transformation <- (
  make_split_dataframe(",", col_names = col_names)
  |> then_select_column("age", String)
  |> then_cast_default(f64) # Different from Python, which just uses "float".
  |> then_clamp(c(18.0, 70.0))  # a best-guess based on public information
  |> then_resize(size = dp_count, constant = 42.0)
  |> then_mean()
)

mean_measurement <- binary_search_chain(
  function(scale) mean_transformation |> then_laplace(scale), d_in, d_out / 3L
)

dp_mean <- qbl_sc(query = mean_measurement) # Different from Python, which does not require "query".
cat("dp_mean:", dp_mean, "\n")
# /mean
