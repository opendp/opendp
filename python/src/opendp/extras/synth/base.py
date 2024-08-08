from __future__ import annotations
from abc import abstractmethod
from typing import Callable

import opendp.prelude as dp
from opendp.mod import Metric, Domain, Measurement, PartialConstructor, assert_features
from opendp.extras._utilities import to_then


class SynthesizerTrainer:
    """Abstract base class for a synthesizer trainer."""

    # OpenDP style make_private_... function
    @classmethod
    def make(cls, input_domain: Domain, input_metric: Metric, epsilon: float,
             *args, **kwargs) -> Measurement:
        raise NotImplementedError

    # OpenDP style then_private_... function
    @classmethod
    def then(cls, epsilon: float,
             *args, **kwargs) -> Callable[..., PartialConstructor]:
        """then constructor."""

        return to_then(cls.make)

    def __init__(self, input_domain: Domain, input_metric: Metric, epsilon: float):
        assert_features("contrib", "floating-point")

        if "LazyFrameDomain" not in str(input_domain.type):
            raise ValueError("input_domain must be a LazyFrame domain")

        if input_metric != dp.symmetric_distance():
            raise ValueError("input metric must be symmetric distance")

        self.input_domain = input_domain
        self.input_metric = input_metric
        self.epsilon = epsilon

    def fit(self, data):
        """Fit the synthesizer to the data.

        :param data: The data to fit a synthesizer to.
        :type data: pl.LazyFrame
        return: A synthesizer that can generate records.
        :rtype: ReleasedSynthesizer
        """
        raise NotImplementedError


class ReleasedSynthesizer:
    """Abstract base class for a released synthesizer generating records."""

    @abstractmethod
    def __init__(self, releasble, configuration):
        pass

    @abstractmethod
    def sample(self, num_samples: int):
        """Generate a sample of records from a released synthesizer.

        :param num_samples: The number of records to generate.
        :type num_samples: int
        :return: A sample of records.
        :rtype: pl.DataFrame
        """
        pass
