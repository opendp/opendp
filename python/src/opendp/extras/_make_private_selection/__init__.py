import opendp.prelude as dp
from opendp._lib import get_np_csprng
from opendp.mod import Measurement


def make_select_private_candidate_above_threshold(
    measurement: Measurement, threshold: float, stop_probability: float
) -> Measurement:
    """Select a private candidate whose score is above a threshold.

    Given `measurement` that satisfies ε-DP, returns new measurement M' that satisfies 2ε-DP.
    M' releases the first invocation of `measurement` whose score is above `threshold`.
    
    Each time a score is below `threshold`
    the algorithm terminates with probability `stop_probability` and returns nothing.

    This combinator considers the score to be the first element of the release of `measurement`.

    Algorithm 1 in `Private selection from private candidates <https://arxiv.org/pdf/1811.07971.pdf#page=7>`_ (Liu and Talwar, STOC 2019).

    :param measurement: A measurement that returns an iterable when invoked,
                 where the first element is a score and remaining elements are.
    :param threshold: The threshold score. Return immediately if the score is above this threshold.
    :param stop_probability: The probability of stopping early at any iteration.
    :returns: A new measurement that returns the first invocation whose score (first value) is above threshold.
    """

    dp.assert_features("contrib", "floating-point")

    np_csprng = get_np_csprng()

    # If stop_probability is 1, the measurement is executed only once with double of the privacy budget,
    # so we prevent this inefficient case.
    if not 0 <= stop_probability < 1:
        raise ValueError("stop_probability must be between 0 and 1")

    if measurement.output_measure != dp.max_divergence():
        raise ValueError(
            "measurement must satisfy pure differential privacy (max_divergence())"
        )

    def function(data):
        while True:
            score, *output = measurement(data)

            if score >= threshold:
                return score, *output

            biased_coin = np_csprng.binomial(n=1, p=stop_probability)
            if biased_coin:
                return

    return dp.m.make_user_measurement(
        input_domain=measurement.input_domain,
        input_metric=measurement.input_metric,
        output_measure=measurement.output_measure,
        function=function,
        privacy_map=lambda d_in: measurement.map(d_in) * 2,
    )
