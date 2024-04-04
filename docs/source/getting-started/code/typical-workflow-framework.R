options(error = traceback)

# unit-of-privacy
library(opendp)
enable_features("contrib")

d_in <- 1L # neighboring data set distance is at most d_in...
input_metric <- symmetric_distance() # ...in terms of additions/removals

# /unit-of-privacy


# privacy-loss
d_out <- 1.0 # output distributions have distance at most d_out (ε)...
privacy_measure <- max_divergence(.T = f64) # ...in terms of pure-DP

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

# TODO: Currently failing with "inferred type is f64, expected u32."
m_sc <- make_sequential_composition(
  input_domain = atom_domain(.T = String),
  input_metric = input_metric,
  output_measure = privacy_measure,
  d_in = d_in,
  d_mids = rep(d_out / 3L, 3L)
)

# TODO: Haven't actually run the code below, because of the error above!
# Call measurement with data to create a queryable:
qbl_sc <- m_sc(data)

# /mediate


# count
count_transformation <- (
  make_split_dataframe(",", col_names = col_names)
  |> make_select_column("age", str)
  |> then_count()
)

count_sensitivity <- count_transformation.map(d_in)
count_sensitivity
# 1

count_measurement <- binary_search_chain(
  function(scale) count_transformation |> dp.m.then_laplace(scale), d_in, d_out / 3L
)
dp_count <- qbl_sc(count_measurement)

# /count


# mean
mean_transformation <- (
  make_split_dataframe(",", col_names = col_names)
  |> make_select_column("age", str)
  |> then_cast_default(float)
  |> then_clamp(c(18.0, 70.0))  # a best-guess based on public information
  |> then_resize(size = dp_count, constant = 42.0)
  |> then_mean()
)

mean_measurement <- dp.binary_search_chain(
  function(scale) mean_transformation |> dp.m.then_laplace(scale), d_in, d_out / 3L
)

dp_mean <- qbl_sc(mean_measurement)

# /mean
