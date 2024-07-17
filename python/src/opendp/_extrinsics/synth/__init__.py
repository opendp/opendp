from opendp.mod import Domain, Metric, Measurement
import opendp.prelude as dp
from opendp._extrinsics.synth.base import Synthesizer


__all__ = ["Synthesizer", "make_private_synthesizer_trainer"]


def make_private_synthesizer_trainer(input_domain: Domain,
                                     input_metric: Metric,
                                     name: str,
                                     epsilon: float,
                                     **kwargs) -> Measurement:

    if "LazyFrameDomain" not in str(input_domain.type):
        raise ValueError("input_domain must be a LazyFrame domain")

    if input_metric != dp.symmetric_distance():
        raise ValueError("input metric must be symmetric distance")

    synthesizer = Synthesizer.create(name, epsilon=epsilon, **kwargs)

    def function(data):
        synthesizer.fit(data.collect())
        return synthesizer

    return dp.m.make_user_measurement(
        input_domain,
        input_metric,
        dp.max_divergence(T=float),
        function,
        lambda d_in: d_in * epsilon)
