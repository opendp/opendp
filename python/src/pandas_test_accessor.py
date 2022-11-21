import opendp
import pandas as pd


@pd.api.extensions.register_dataframe_accessor("ezodp")
class EZODPAccessor:

    def __init__(self, pandas_obj):
        self._obj = pandas_obj
        self._operation = None

    def sum(self):
        lat = self._obj.latitude
        lon = self._obj.longitude
        return (float(lon.mean()), float(lat.mean()))


def test_accessor():
    pass
