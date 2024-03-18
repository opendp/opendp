import unittest
from postprocessors import Tulap

class TestTulap(unittest.TestCase):
    def setUp(self):
        self.data = [1, 2, 3]  
        self.epsilon = 0.1
        self.delta = 0.01
        self.theta = 0.5
        self.size = 100
        self.tulap_instance = Tulap(self.data, self.epsilon, self.delta, self.theta, self.size)

    def test_ump_test(self):
        alpha = 0.05
        tail = 'left'  
        result = self.tulap_instance.ump_test(alpha, tail)
        self.assertIsNotNone(result)  

    def test_CI(self):
        alpha = 0.05
        tail = 'lower'  # Can also be 'right' or None
        result = self.tulap_instance.CI(alpha, tail)
        self.assertIsNotNone(result)

    def test_p_value(self):
        tail = 'left'  # Can also be 'right' or None
        result = self.tulap_instance.p_value(tail)
        self.assertIsNotNone(result)

if __name__ == '__main__':
    unittest.main()
