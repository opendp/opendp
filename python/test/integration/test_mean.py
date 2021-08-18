import unittest

from opendp.mean import *

enable_features("floating-point")


class DPMeanTest(unittest.TestCase):

    def setUp(self) -> None:
        data_path = os.path.join('..', '..', 'example', 'data', 'PUMS_california_demographics_1000', 'data.csv')
        var_names = ["age", "sex", "educ", "race", "income", "married", "pid"]
        with open(data_path) as input_data:
            self.data = input_data.read()

    def test_dp_mean_compute(self):
        """
        A test to ensure that the computation chain for DP Mean completes and returns a value
        :return:
        """
        epsilon = 1.
        col_names = ["A", "B", "C", "D", "E"]
        column = "E"
        dp_mean = DPMean(self.data)
        res = dp_mean.compute(col_names, column, 0., 10000., 1000, epsilon)
        print(res)
        self.assertEqual(type(res), float)
        self.assertGreater(res, 7000)
        self.assertLess(res, 8000)


if __name__ == '__main__':
    unittest.main()
