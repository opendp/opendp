# 1
# convert this to notebook 
from postprocessors import _ptulap
import opendp.prelude as dp
import numpy as np  # type: ignore[import]

dp.enable_features("contrib")
t_values = np.array([0.1, 0.5, 1.0, -0.5, -1.0])  # example input values
m = 0
epsilon = 0.5
delta = 0.3

ptulap_results = _ptulap(t_values, m, epsilon, delta)
print (ptulap_results)
print ("END OF 1")

# 2
from opendp._extrinsics.tulap.postprocessors import make_ump_test

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
print ("END OF 2")


theta = 0.5  # Probability of success in binomial distribution
size = 10    # Number of trials
epsilon = 0.5      # Tulap parameter b
delta= 0.3      # Tulap parameter q
tail = "right"  # Tail for the p-value calculation

# Create a function to calculate p-values
pvalue_function = make_oneside_pvalue(theta, size, epsilon, delta, tail)

# Generate an example set of Z values (Tulap random variables)
Z_values = np.array([2, 3, 4])  # Example Tulap random variables

# Calculate p-values
pvalues = pvalue_function(Z_values)
print(pvalues)
print("type: ", type(pvalues))
print ("END OF 3")


# 4
from opendp._extrinsics.tulap.postprocessors import make_twoside_pvalue

theta = 0.5  # Probability of success in binomial distribution
size = 10    # Number of trials
epsilon= 0.5      # Tulap parameter b
delta= 0.3      # Tulap parameter q

# Create a function to calculate two-sided p-values
twoside_pvalue_function = make_twoside_pvalue(theta, size, epsilon, delta)

# Generate an example set of Z values (Tulap random variables)
Z_values = np.array([1])  # Example Tulap random variables

# Calculate two-sided p-values
twoside_pvalues = twoside_pvalue_function(Z_values)

print ("this is two sided p value: ", twoside_pvalues)
print ("this is two sided p value type:  ", type(twoside_pvalues))

print ("END OF 4")

# 5
# Example parameters for make_CI2
alpha = 0.05
size = 100
epsilon = 0.1
delta = 0.01
tail = "lower"

# Obtain the DP-wrapped CI function
CI2_function = make_CI(alpha, size, epsilon, delta, tail)

# Now, call the DP-wrapped function with an example input
Z_example = [1]  # Example input for the CI function
result = CI2_function(Z_example)

print("Result from the DP-wrapped CI function:", result)


