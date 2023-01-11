import apache_beam
from opendp.beam import *


enable_features("floating-point", "contrib", "honest-but-curious")

def test_make_mul():
    data = [1.0, 2.0, 3.0]
    with apache_beam.Pipeline() as p:
        pcollection = p | "Create" >> apache_beam.Create(data)
        mul = make_mul(2.0)
        mul_pcollection = mul(pcollection)
        out = take_method(mul_pcollection, "i32")
    assert out == [x * 2 for x in data]


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


if __name__ == "__main__":
    test_make_mul()
