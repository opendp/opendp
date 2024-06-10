from opendp.mod import assert_features


class OnceFrame(object):
    def __init__(self, queryable):
        """OnceFrame is a Polars LazyFrame that may only be collected into a DataFrame once.

        The APIs on this class mimic those that can be found in Polars.

        Differentially private guarantees on a given LazyFrame require the LazyFrame to be evaluated at most once.
        The purpose of this class is to protect against repeatedly evaluating the LazyFrame.
        """
        self.queryable = queryable

    def collect(self):
        """Collects a DataFrame from a OnceFrame, exhausting the OnceFrame."""
        from opendp._data import onceframe_collect
        return onceframe_collect(self.queryable)

    def lazy(self):
        """Extracts a LazyFrame from a OnceFrame,
        circumventing protections against multiple evaluations.

        Each collection consumes the entire allocated privacy budget.
        To remain DP at the advertised privacy level, only collect the LazyFrame once.

        **Features:**

        * `honest-but-curious` - LazyFrames can be collected an unlimited number of times.
        """
        from opendp._data import onceframe_lazy
        assert_features("honest-but-curious")
        return onceframe_lazy(self.queryable)
