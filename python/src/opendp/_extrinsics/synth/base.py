from __future__ import annotations
from typing import Union

import polars as pl

from opendp._lib import import_optional_dependency


SYNTH_MAP = {
    "MWEM": "opendp._extrinsics.synth.mwem.MWEMSynthesizer",
}


class Synthesizer:
    @classmethod
    def list_synthesizers(cls):
        """
        List the available synthesizers.

        :return: List of available synthesizer names.
        :rtype: list[str]
        """
        return list(SYNTH_MAP.keys())

    # from Smart Noise
    # https://github.com/opendp/smartnoise-sdk/blob/main/synth/snsynth/base.py
    @classmethod
    def create(cls, synth: Union[None, str, Synthesizer],
               epsilon: float, *args, **kwargs) -> Synthesizer:
        """
        Create a differentially private synthesizer.

        :param synth: The name of the synthesizer to create.  If called from an instance of a Synthesizer subclass, creates
            an instance of the specified synthesizer.  Allowed synthesizers are available from
            the list_synthesizers() method.
        :type synth: str or Synthesizer class, required
        :param epsilon: The privacy budget to be allocated to the synthesizer.  This budget will be
            used when the synthesizer is fit to the data.
        :type epsilon: float, required
        :param args: Positional arguments to pass to the synthesizer constructor.
        :type args: list, optional
        :param kwargs: Keyword arguments to pass to the synthesizer constructor.  At a minimum,
            the epsilon value must be provided.  Any other hyperparameters can be provided
            here.  See the documentation for each specific synthesizer for details about available
            hyperparameter.
        :type kwargs: dict, optional

        """

        if synth is None or (isinstance(synth, type) and issubclass(synth, Synthesizer)):
            clsname = cls.__module__ + '.' + cls.__name__ if synth is None else synth.__module__ + '.' + synth.__name__
            if clsname == '.base.Synthesizer':
                raise ValueError("Must specify a synthesizer to use.")
            matching_keys = [k for k, v in SYNTH_MAP.items() if v == clsname]
            if len(matching_keys) == 0:
                raise ValueError(f"Synthesizer {clsname} not found in map.")
            elif len(matching_keys) > 1:
                raise ValueError(f"Synthesizer {clsname} found multiple times in map.")
            else:
                synth = matching_keys[0]
        if isinstance(synth, str):
            synth = synth.upper()
            if synth not in SYNTH_MAP:
                raise ValueError('Synthesizer {} not found'.format(synth))
            synth_class_name = SYNTH_MAP[synth]
            synth_module_name, synth_class_name = synth_class_name.rsplit('.', 1)
            synth_module = __import__(synth_module_name, fromlist=[synth_class_name])
            synth_class = getattr(synth_module, synth_class_name)
            return synth_class(epsilon=epsilon, *args, **kwargs)
        else:
            raise ValueError('Synthesizer must be a string or a class')

    def __init__(self):
        self._is_fitted = False

    def fit(self, data: pl.LazyFrame):
        assert not self._is_fitted, "Synthesizer is already fitted"
        self._is_fitted = True

    def sample(self, num_samples: int) -> pl.DataFrame:
        assert self._is_fitted, "Synthesizer must be fitted first"

    def releasable(self):
        assert self._is_fitted, "Synthesizer must be fitted first"
