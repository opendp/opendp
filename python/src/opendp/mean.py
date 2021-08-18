from opendp.trans import *
from opendp.meas import *


class DPMean(object):

    def __init__(self, data, impute_constant=0.):
        self.data = data
        self.impute_constant = impute_constant

    def _binary_search(self, predicate, start, end):
        if start > end:
            raise ValueError

        if not predicate(end):
            raise ValueError("no possible value in range")

        while True:
            mid = (start + end) / 2
            passes = predicate(mid)

            if passes and end - start < .00001:
                return mid

            if passes:
                end = mid
            else:
                start = mid

    def check_scale(self, scale, preprocessor, dataset_distance, epsilon):
        """
        Return T/F
        :param scale:
        :param preprocessor:
        :param dataset_distance:
        :param epsilon:
        :return:
        """
        return (preprocessor >> make_base_laplace(scale)).check(dataset_distance, epsilon)

    def compute(self, col_names, index, lower, upper, n, epsilon):
        """
        Draft of a function to be used on the backend for DPCreator
        :param index: Column index to select data from
        :param lower: Lower bound for clamp
        :param upper: Upper bound for clamp
        :param n: Estimated number of values in data
        :param epsilon: Privacy budget
        :return:
        """
        preprocessor = (
            # Convert data into Vec<Vec<String>>
                make_split_dataframe(separator=",", col_names=col_names) >>
                # Selects a column of df, Vec<str>
                make_select_column(key=index, T=str) >>
                # Cast the column as Vec<Optional<Float>>
                make_cast(TI=str, TO=float) >>
                # Impute missing values to 0 Vec<Float>
                make_impute_constant(self.impute_constant) >>
                # Clamp age values
                make_clamp(lower, upper) >>
                make_resize_bounded(self.impute_constant, n, lower, upper) >>
                make_bounded_mean(lower, upper, n=n, T=float)
        )
        scale = self._binary_search(lambda s: self.check_scale(s, preprocessor, 1, epsilon), 0., 10.)
        preprocessor = preprocessor >> make_base_laplace(scale)
        return preprocessor(self.data)