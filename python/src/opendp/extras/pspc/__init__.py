import opendp.prelude as dp
from opendp._lib import get_np_csprng
from opendp.mod import Measurement


def make_private_selection_threshold(meas: Measurement,
                                     threshold: float,
                                     stop_probability: float) -> Measurement:

    """Measurement for private selection with known threshold.

    This executes an ε-DP mechanism M that returns a tuple (q, x) repeatedly
    until the score q is at least `threshold`.
    Only the last output of M is released.

    Algorithm 1 in `Private selection from private candidates <https://arxiv.org/pdf/1811.07971.pdf#page=7>`_ (Liu and Talwar, STOC 2019).

    :param meas: A measurement function that returns a 2-tuple when invoked,
                 where the first element is a score and the second element is the output of interest.
    :param threshold: The threshold score. Return immediately if the score is above this threshold.
    :param stop_probability: The probability of stopping early at any iteration.
    :returns: A tuple from `meas` with the first element being the score.
    """

    dp.assert_features("contrib", "floating-point")

    np_csprng = get_np_csprng()

    # If stop_probability is 1, the measurement is executed only once with double of the privacy budget,
    # so we prevent this inefficient case.
    if not 0 <= stop_probability < 1:
        raise ValueError("stop_probability must be between 0 and 1")

    if meas.output_measure != dp.max_divergence(T=float):
        raise ValueError("meas must satisfy pure differential privacy (max_divergence(T=float))")

    def function(data):

        while True:

            score, *output = meas(data)

            if score >= threshold:
                return score, output

            biased_coin = np_csprng.binomial(n=1, p=stop_probability)
            if biased_coin:
                return

    return dp.m.make_user_measurement(
        input_domain=meas.input_domain,
        input_metric=meas.input_metric,
        output_measure=meas.output_measure,
        function=function,
        privacy_map=lambda d_in: meas.map(d_in) * 2)
