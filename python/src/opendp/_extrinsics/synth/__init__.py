from opendp.mod import Domain, Metric
import opendp.prelude as dp


class Synthesizer:
    def __init__(self):
        self._is_fitted = False

    def fit(self, data):
        assert not self._is_fitted, "Synthesizer is already fitted"

    def sample(self, num_records):
        assert self._is_fitted, "Synthesizer must be fitted first"

    def releasable(self):
        assert self._is_fitted, "Synthesizer must be fitted first"


class IdentitySynthesizer(Synthesizer):
    def __init__(self, epsilon):
        super().__init__()

    def fit(self, data):
        super().fit(data)
        self._is_fitted = True
        self._data = data

    def sample(self, num_records):
        super().sample(num_records)
        return self._data.sample(num_records)


def make_private_synthesizer_trainer(input_domain: Domain,
                                     input_metric: Metric,
                                     epsilon: float,
                                     name: str,
                                     **kwargs):

    print("THIS IS A PLACEHOLDER. NOT DIFFERENTIALLY PRIVATE!!!")

    if "LazyFrameDomain" not in str(input_domain.type):
        raise ValueError("input_domain must be a LazyFrame domain")

    if input_metric != dp.symmetric_distance():
        raise ValueError("input metric must be symmetric distance")

    match name.lower():
        case "identity":
            synthesizer = IdentitySynthesizer(epsilon)
        case _:
            raise ValueError(f"Synthesizer {name} not found")

    def function(data):
        synthesizer.fit(data.collect())
        return synthesizer

    return dp.m.make_user_measurement(
        input_domain,
        input_metric,
        dp.max_divergence(T=float),
        function,
        lambda d_in: d_in * epsilon)
