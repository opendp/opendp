import unittest

from opendp.mean import *

enable_features("floating-point")


class DPMeanTest(unittest.TestCase):

    def setUp(self) -> None:
        self.data = '59,1,9,1,0,1\n31,0,1,3,17000,0\n36,1,11,1,0,1\n54,1,11,1,9100,1\n39,0,5,3,37000,0\n34,0,9,1,0,1\n'\
                    '93,1,8,1,6000,1\n69,0,13,1,350000,1\n40,1,11,3,33000,1\n27,1,11,1,25000,0\n59,1,13,1,49000,1\n' \
                    '31,1,11,3,0,1\n73,1,13,4,35500,0\n89,1,9,1,4000,1\n39,1,10,3,15000,0\n51,1,13,4,120000,1\n' \
                    '32,0,9,1,13000,0\n52,0,11,2,45000,0\n24,0,7,1,0,0\n48,1,10,1,4300,1\n51,0,1,3,16000,1\n' \
                    '43,1,14,1,365000,1\n29,0,4,3,20000,0\n44,1,15,1,17900,1\n87,1,8,1,3600,0\n27,1,11,3,10800,0\n' \
                    '58,0,13,1,60900,1\n32,1,11,3,25000,1\n'

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
        self.assertEqual(type(res), float)


if __name__ == '__main__':
    unittest.main()
