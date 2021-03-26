import opendp


def test_init():
    odp = opendp.OpenDP()
    assert odp

def test_identity():
    odp = opendp.OpenDP()

    ### HELLO WORLD
    identity = odp.trans.make_identity(b"<String>")
    arg = odp.data.from_string(b"hello, world!")
    res = odp.core.transformation_invoke(identity, arg)
    # TODO: Fix extra quotes
    assert odp.to_str(res) == '"hello, world!"'
    odp.data.data_free(arg)
    odp.data.data_free(res)
    odp.core.transformation_free(identity)
