import apache_beam as beam
import ctypes
from opendp.beam import *

enable_features("floating-point", "contrib", "honest-but-curious")


def test_make_mul():
    data = list(range(10))
    with beam.Pipeline() as p:
        pcollection = p | "Create" >> beam.Create(data)
        print("Just made", ctypes.py_object(pcollection))
        # collection = make_collection(pcollection, "i32")
        # print("After make_collection", ctypes.py_object(pcollection))
        mul = make_mul(2, "i32")
        mul_pcollection = mul(pcollection)
        print(mul_pcollection)
        mul_pcollection | "Combine" >> beam.combiners.ToList() | "Print" >> beam.Map(lambda x: print(x))

"""
test makes pcollection
test calls make_collection()
    make_collection() calls opendp_beam__new_collection_methods()
        opendp_beam__new_collection_methods() allocates and returns AnyObject0
    make_collection() calls unwrap(AnyObject0)
        unwrap() extracts AnyObject0, but it's not freed -- WHY?
    make_collection() returns AnyObject0
test makes py_transformation
test calls py_transformation(AnyObject0)
    py_transformation() calls rust_transformation(AnyObject0)
        rust_transformation() takes reference to AnyObject0
        rust_transformation() calls external.map(data)
            external.map() calls runtime.map()
                runtime.map() calls make_collection()
                    make_collection() calls opendp_beam__new_collection_methods()
                        opendp_beam__new_collection_methods() allocates and returns AnyObject1
                    make_collection() calls unwrap(AnyObject1)
                        unwrap() extracts AnyObject1
                    make_collection() returns AnyObject1
                runtime.map() returns AnyObject1
            external.map() consumes AnyObject1
            external.map() returns Collection
        rust_transformation() allocates and returns AnyObject2
    py_transformation frees AnyObject1
"""




def make_mul_beam(x, map_impl):
    def f(arg, ctx):
        return arg * ctx
    def function(arg):
        return map_impl(arg, f, x)
    return function


def test_make_sum_beam():
    def map_impl(arg, f, ctx):
        return arg | "Mul" >> beam.Map(lambda a: f(a, ctx))

    with beam.Pipeline() as p:
        mul2 = make_mul_beam(2, map_impl)
        arg = p | "Create" >> beam.Create(list(range(10)))
        res = mul2(arg)
        res | "Combine" >> beam.combiners.ToList() | "Print" >> beam.Map(lambda x: print(x))


if __name__ == "__main__":
    # test_make_sum_beam()
    test_make_mul()
