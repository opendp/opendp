from typing import Optional

import opendp.prelude as dp
from opendp._lib import import_optional_dependency
from opendp.mod import Measurement


def make_private_selection_threshold(meas: Measurement,
                                     threshold: float,
                                     stop_probability: float,
                                     epsilon_selection: float,
                                     steps: Optional[int] = None):

    """Measurement for private selection with known threshold.

    This executes an ε-DP mechanism M that returns a tuple (q, x) repeatedly
    until the score q is at least T, a predefined threshold.
    Only the last output of M is released.

    Algorithm 1 in `Private selection from private candidates <https://arxiv.org/pdf/1811.07971.pdf#page=7>`_ (Liu and Talwar, STOC 2019).

    :param meas: A measurement function that returns a 2-tuple when invoked,
                 where the first element is a score and the second element is the output of intreset.
    :param threshold: The threshold score. Return immediately if the score is above this threshold.
    :param stop_probability: The probability of stopping early at any iteration.
    :param epsilon_selection: The epsilon allocated to the private selection.
    :param steps: Optional. The number of steps to run. If not specified, will run the minimum number of steps.
    :returns: A tuple from `meas` with the first element being the score.
    """

    np = import_optional_dependency("numpy")

    dp.assert_features("contrib", "floating-point")

    if not 0 <= stop_probability <= 1:
        raise ValueError("stop_probability must be between 0 and 1")

    if not 0 <= epsilon_selection <= 1:
        raise ValueError("epsilon must be between 0 and 1")

    assert (stop_probability == 0) == (epsilon_selection == 0), "either both stop_probability and epsilon_selection should be 0, or neither should be 0."

    # From proof for (b), budget consumption
    # https://arxiv.org/pdf/1811.07971.pdf#page=25
    if stop_probability == 0 and epsilon_selection == 0:
        min_steps = None
        epsilon_selection_contribution = 0.
    else:
        min_steps = int(
            np.ceil(max(np.log(2 / epsilon_selection) / stop_probability, 1 + 1 / (np.exp(1) * stop_probability)))
        )
        epsilon_selection_contribution = epsilon_selection

    max_steps = steps or min_steps

    if min_steps is not None and max_steps is not None and max_steps < min_steps:
        raise ValueError(f"given the parameters, must run at least {min_steps} steps")

    if meas.output_measure != dp.max_divergence(T=float):
        raise ValueError("meas must be pure differential privacy (max_divergence(T=float))")

    def function(data):
        step = 0

        while True:

            score, *output = meas(data)

            if score >= threshold:
                return score, output

            biased_coin = np.random.binomial(n=1, p=stop_probability)
            if biased_coin:
                return

            if max_steps is not None and step >= max_steps:
                return

            step += 1

    return dp.m.make_user_measurement(
        input_domain=meas.input_domain,
        input_metric=meas.input_metric,
        output_measure=dp.max_divergence(T=float),
        function=function,
        privacy_map=lambda d_in: meas.map(d_in) * 2 + epsilon_selection_contribution)
