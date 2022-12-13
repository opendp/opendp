import apache_beam as beam
from opendp.mod import enable_features

import ctypes

import ctypes

ctypes.c_int32(1)

def make_mul_beam(x, map_impl):
    def fn(arg, ctx):
        return arg * ctx
    def function(arg):
        return map_impl(arg, fn, x)
    return function


def map_impl(arg, fn, ctx):
    return arg | "Mul" >> beam.Map(lambda a: fn(a, ctx))

map_impl_c_type = ctypes.CFUNCTYPE(ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p)
map_impl_c = map_impl_c_type(map_impl)

def test_make_sum_beam():
    enable_features("contrib")
    # import os
    # out_path = "/tmp/beam_test_out.txt"
    # out_path_part = f"{out_path}-00000-of-00001"
    # if os.path.isfile(out_path_part):
    #     os.remove(out_path_part)

    with beam.Pipeline() as p:
        # args = (
        #     p | 'Read' >> beam.io.ReadFromText("/Users/av/Repositories/opendp/README.md")
        #     | 'PairWithOne' >> beam.Map(lambda x: (x, 1))
        #     | 'GroupAndSum' >> beam.CombinePerKey(sum)
        #     | 'Format' >> beam.MapTuple(lambda w, c: '%s: %d' % (w, c))
        #     | 'Write' >> beam.io.WriteToText("/tmp/beam_test_out.txt")
        # )

        mul2 = make_mul_beam(2, map_impl)
        arg = p | "Create" >> beam.Create(list(range(10)))
        res = mul2(arg)

        # res | "Write" >> beam.io.WriteToText(out_path)
        res | "Combine" >> beam.combiners.ToList() | "Print" >> beam.Map(lambda x: print(x))

if __name__ == "__main__":
    test_make_sum_beam()
