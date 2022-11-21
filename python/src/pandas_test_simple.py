import opendp.combinators as comb
import opendp.measurements as meas
import opendp.mod as mod
import opendp.transformations as trans

import pandas as pd


class OpenDPWrapper:

    def __init__(self, data, make_chain=None):
        self.data = data
        self.make_chain = make_chain

    def clamp(self):
        pass

    def sum(self):
        new_make_operation = comb.make_chain_tt()

    def query(self, d_in, d_out):
        chain = mod.binary_search_chain(self.make_chain, d_in=d_in, d_out=d_out)
        return chain(self.data)


def test_simple():
    data = [1, 2, 3]
    wrapper = OpenDPWrapper(data)
    answer = (
        wrapper
            .clamp((0, 5))
            .sum().base_laplace().query(1, 1.0)
              )
    print(answer)
