# 1
from postprocessors import _ptulap
import opendp.prelude as dp
import math
import numpy as np
from scipy.stats import binom
import scipy
import pdb
dp.enable_features("contrib")
t_values = np.array([0.1, 0.5, 1.0, -0.5, -1.0])  # example input values
m = 0
b = 0.5
q = 0.3

ptulap_results = _ptulap(t_values, m, b, q)
print (ptulap_results)

# 2
from postprocessors import make_ump_test
theta = 0.5  # Probability of success
size = 10  # Sample size
alpha = 0.05  # Significance level
epsilon = 0.1  # Differential privacy parameter epsilon
delta = 0.01  # Differential privacy parameter delta
tail = "left"  # Tail of the test
ump_test_function = make_ump_test(theta, size, alpha, epsilon, delta, tail)
data_for_test = 6  
ump_test_result = ump_test_function(data_for_test)
print(ump_test_result)




#3
from postprocessors import make_oneside_pvalue
theta = 0.5  # Probability of success in binomial distribution
size = 10    # Number of trials
b = 0.5      # Tulap parameter b
q = 0.3      # Tulap parameter q
tail = "right"  # Tail for the p-value calculation

# Create a function to calculate p-values
pvalue_function = make_oneside_pvalue(theta, size, b, q, tail)

# Generate an example set of Z values (Tulap random variables)
Z_values = np.array([2, 3, 4])  # Example Tulap random variables

# Calculate p-values
pvalues = pvalue_function(Z_values)
print(pvalues)



# 4
from postprocessors import make_twoside_pvalue
theta = 0.5  # Probability of success in binomial distribution
size = 10    # Number of trials
b = 0.5      # Tulap parameter b
q = 0.3      # Tulap parameter q

# Create a function to calculate two-sided p-values
twoside_pvalue_function = make_twoside_pvalue(theta, size, b, q)

# Generate an example set of Z values (Tulap random variables)
Z_values = np.array([2, 3, 4])  # Example Tulap random variables

# Calculate two-sided p-values
twoside_pvalues = twoside_pvalue_function(Z_values)

print (twoside_pvalues)



# 5
from postprocessors import make_CI
alpha = 0.05  # significance level, e.g., 5%
size = 100    # sample size
b = 0.1       # some parameter related to the statistical measure or differential privacy
q = 0.2       # another parameter, possibly a quantile or differential privacy parameter
tail = "lower" # type of tail for the confidence interval

# Assuming Z is a statistic calculated from your data
Z = 0.5 # This is a placeholder value. Replace with your actual statistic.

# Creating the confidence interval
CI_function = make_CI(alpha, size, b, q, tail)
print (CI_function)

# 6
from postprocessors import make_CI_twoside
alpha = 0.05  # significance level
size = 100    # sample size
b = 0.1       # some parameter related to the statistical measure or differential privacy
q = 0.2       # another parameter, possibly a quantile or differential privacy parameter

# Assuming Z is a statistic calculated from your data
Z = 50 # This is a placeholder value. Replace with your actual statistic.

# Creating the two-sided confidence interval
CI_twoside_function = make_CI_twoside(alpha, size, b, q)

# Using the confidence interval function
confidence_interval = CI_twoside_function(Z)

# Output the confidence interval
print("Two-sided Confidence Interval:", confidence_interval)
