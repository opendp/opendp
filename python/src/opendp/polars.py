from opendp.mod import assert_features


class OnceFrame(object):
    def __init__(self, value):
        self.value = value

    def collect(self):
        """Collects a DataFrame from a OnceFrame, exhausting the OnceFrame."""
        from opendp._data import onceframe_collect
        return onceframe_collect(self.value)

    def lazy(self):
        r"""Extracts a LazyFrame from a OnceFrame,
        circumventing protections against multiple evaluations.

        Each collection consumes the entire allocated privacy budget.
        To remain DP at the advertised privacy level, only collect the LazyFrame once.

        **Features:**

        * `honest-but-curious` - LazyFrames can be collected an unlimited number of times.
        """
        from opendp._data import onceframe_lazy
        assert_features("honest-but-curious")
        return onceframe_lazy(self.value)
