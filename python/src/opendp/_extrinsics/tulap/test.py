import numpy as np
import pytest
from unittest.mock import patch
import math
from postprocessors import _ptulap, _make_ump_test, _make_oneside_pvalue 
import opendp as dp

def test_positive_input():
    """Test with a positive t, checks basic operation"""
    t = np.array([1])  # Adjusted to array
    result = _ptulap(t, epsilon=0.1, delta=0.1)
    assert isinstance(result, np.ndarray)
    assert result[0] > 0, "Result should be positive for positive t"

def test_negative_input():
    """Test with a negative t, checks basic operation"""
    t = -1
    result = _ptulap(t, epsilon=0.1, delta=0.1)
    assert isinstance(result, np.ndarray)
    assert result < 1, "Result should be less than 1 for negative t"

def test_array_input():
    """Test with an array of t values"""
    t = np.array([0, 1, -1])
    result = _ptulap(t, epsilon=0.1, delta=0.1)
    assert isinstance(result, np.ndarray)
    assert len(result) == 3, "Result should have the same length as input"

def test_inf_handling():
    """Test to ensure infinities are handled correctly"""
    result = _ptulap(np.array([np.inf]), epsilon=0.1, delta=0.1)
    assert not np.isinf(result).any(), "Result should not contain infinities"

def test_left_tail_basic():
    """Test the left tail functionality with basic inputs."""
    size = 10
    theta = 0.5
    alpha = 0.05
    epsilon = 0.1
    delta = 0.01
    tail = "left"
    data = True  

    ump_test_func = _make_ump_test(theta, size, alpha, epsilon, delta, tail)
    result = ump_test_func(data)
    assert isinstance(result, list), "Result should be a list"
    assert all(isinstance(item, float) for item in result), "All items in the result should be floats"

@patch('postprocessors._ptulap')
@patch('postprocessors.dp.new_function')
def test_right_tail_single_value(mock_new_function, mock_ptulap):
    # Adjust the mock to return an array that has the correct shape (11 elements for size=10)
    size = 10
    mock_ptulap.return_value = np.array([0.5] * (size + 1))  # Now correctly sized to match the binom.pmf array
    mock_new_function.side_effect = lambda func, TO: func

    theta = 0.5
    epsilon = 0.1
    delta = 0.01
    tail = "right"
    Z = np.array([5,1])

    # Execute the function
    pvalue_func = _make_oneside_pvalue(theta, size, epsilon, delta, tail)
    pvalue = pvalue_func(Z)

    # Assert that the p-value calculation behaves as expected
    # The dot operation should now succeed without a shape mismatch error
    assert np.all(pvalue >= 0) and np.all(pvalue <= 1), "P-values should be within [0, 1]"
