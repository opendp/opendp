import opendp.measurements as meas
import opendp.mod as mod
import opendp.transformations as trans


class PrivateData:

    def __init__(self, data, pre_transformation=None, make_measurement=None, post_transformation=None):
        self.data = data
        self.pre_transformation = pre_transformation
        self.make_measurement = make_measurement
        self.post_transformation = post_transformation

    def _chain_transformation(self, transformation):
        def chain(x1, x0):
            return x0 >> x1 if x0 is not None else x1
        if self.make_measurement is None:
            return PrivateData(self.data, chain(transformation, self.pre_transformation), self.make_measurement, self.post_transformation)
        else:
            return PrivateData(self.data, self.pre_transformation, self.make_measurement, chain(transformation, self.post_transformation))

    def _chain_measurement(self, make_measurement):
        if self.make_measurement is not None:
            raise Exception("Can only apply one Measurement")
        return PrivateData(self.data, self.pre_transformation, make_measurement, self.post_transformation)

    def clamp(self, bounds, TA=None):
        return self._chain_transformation(trans.make_clamp(bounds, TA=TA))

    def bounded_sum(self, bounds, MI="SymmetricDistance", T=None):
        return self._chain_transformation(trans.make_bounded_sum(bounds, MI=MI, T=T))

    def base_laplace(self, k=-1074, D="AllDomain<T>"):
        return self._chain_measurement(lambda scale: meas.make_base_laplace(scale, k=k, D=D))

    def get(self, d_in, d_out):
        if self.make_measurement is None:
            raise Exception("Must apply one Measurement")
        def make_chain(param):
            measurement = self.make_measurement(param)
            if self.pre_transformation is not None:
                measurement = self.pre_transformation >> measurement
            if self.post_transformation is not None:
                measurement = measurement >> self.post_transformation
            return measurement
        chain = mod.binary_search_chain(make_chain, d_in=d_in, d_out=d_out)
        return chain(self.data)


def test_simple():
    mod.enable_features("contrib", "floating_point")
    data = [1.0, 2.0, 3.0]
    bounds = (0.0, 5.0)

    private_data = PrivateData(data)
    answer = (
        private_data
            .clamp(bounds)
            .bounded_sum(bounds)
            .base_laplace()
            .get(1, 1.0)
    )
    print(answer)
