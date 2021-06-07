from opendp.v1.typing import HammingDistance, L1Sensitivity, SymmetricDistance

INT_DATA = list(range(1, 10))
FLOAT_DATA = list(map(float, INT_DATA))


def test_bounded_mean():
    from opendp.v1.trans import make_bounded_mean
    query = make_bounded_mean(lower=0., upper=10., n=9, MI=HammingDistance, MO=L1Sensitivity[float])
    assert query(FLOAT_DATA) == 5.
    assert query.check(1, 10. / 9.)


def test_bounded_sum_float():
    from opendp.v1.trans import make_bounded_sum
    query = make_bounded_sum(lower=0., upper=10., MI=HammingDistance, MO=L1Sensitivity[float])
    assert query(FLOAT_DATA) == 45.
    # TODO: tighten the check
    assert query.check(1, 20.)


def test_bounded_sum_int():
    from opendp.v1.trans import make_bounded_sum
    query = make_bounded_sum(lower=0, upper=10, MI=HammingDistance, MO=L1Sensitivity[int])
    assert query(INT_DATA) == 45
    # TODO: tighten the check
    assert query.check(1, 20)

    try:
        query(FLOAT_DATA)
        raise ValueError("should not accept float data")
    except AssertionError:
        pass



def test_identity_int():
    from opendp.v1.trans import make_identity
    transformation = make_identity(HammingDistance, int)
    arg = 123
    ret = transformation(arg)
    assert ret == arg


def test_identity_float():
    from opendp.v1.trans import make_identity
    transformation = make_identity(HammingDistance, float)
    arg = 123.123
    ret = transformation(arg)
    assert ret == arg


def test_identity_str():
    from opendp.v1.trans import make_identity
    transformation = make_identity(HammingDistance, str)
    arg = "hello, world"
    ret = transformation(arg)
    assert ret == arg


def test_identity_list():
    from opendp.v1.trans import make_identity
    transformation = make_identity(HammingDistance, "Vec<i32>")
    arg = [1, 2, 3]
    ret = transformation(arg)
    assert ret == arg


def test_count():
    from opendp.v1.trans import make_count
    transformation = make_count(SymmetricDistance, L1Sensitivity["i32"], int)
    arg = [1, 2, 3]
    ret = transformation(arg)
    assert ret == 3