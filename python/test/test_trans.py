from opendp.v1.typing import HammingDistance, L1Sensitivity, SymmetricDistance

INT_DATA = list(range(1, 10))
FLOAT_DATA = list(map(float, INT_DATA))


def test_identity():
    from opendp.v1.trans import make_identity
    # test int
    transformation = make_identity(HammingDistance, int)
    arg = 123
    ret = transformation(arg)
    assert ret == arg

    transformation = make_identity(HammingDistance, float)
    arg = 123.123
    ret = transformation(arg)
    assert ret == arg

    transformation = make_identity(HammingDistance, str)
    arg = "hello, world"
    ret = transformation(arg)
    assert ret == arg

    transformation = make_identity(HammingDistance, "Vec<i32>")
    arg = [1, 2, 3]
    ret = transformation(arg)
    assert ret == arg


# TODO: cannot test independently until Vec<String> data loader implemented
def test_split_lines__parse_series():
    from opendp.v1.trans import make_split_lines, make_parse_series
    query = make_split_lines(M=HammingDistance) >> make_parse_series(impute=True, M=HammingDistance, TO=int)
    assert query("1\n2\n3") == [1, 2, 3]
    assert query.check(1, 1)


def test_dataframe_pipeline():
    from opendp.v1.trans import make_split_lines, make_split_records, \
        make_create_dataframe, make_parse_column, make_select_column

    query = (
        make_split_lines(M=HammingDistance) >>
        make_split_records(separator=",", M=HammingDistance) >>
        make_create_dataframe(col_names=[1, 2], M=HammingDistance) >>
        make_parse_column(key=1, impute=True, M=HammingDistance, T=int) >>
        make_select_column(key=1, M=HammingDistance, T=int)
    )
    assert query("1,1.\n2,2.\n3,3.") == [1, 2, 3]
    assert query.check(1, 1)


def test_split_dataframe():
    from opendp.v1.trans import make_split_dataframe, make_parse_column, make_select_column

    query = (
        make_split_dataframe(separator=",", col_names=[23, 17], M=HammingDistance) >>
        make_parse_column(key=23, impute=True, M=HammingDistance, T=int) >>
        make_select_column(key=23, M=HammingDistance, T=int)
    )
    assert query("1,1.\n2,2.\n3,3.") == [1, 2, 3]
    assert query.check(1, 1)


def test_clamp_vec():
    from opendp.v1.trans import make_clamp_vec
    query = make_clamp_vec(lower=-1, upper=1, M=HammingDistance)
    assert query([-10, 0, 10]) == [-1, 0, 1]
    assert query.check(1, 1)


def test_clamp_sensitivity():
    from opendp.v1.trans import make_clamp_sensitivity
    query = make_clamp_sensitivity(lower=-1, upper=1, M=L1Sensitivity[int])
    assert query(20) == 1
    assert query.check(20, 2)


def test_bounded_mean():
    from opendp.v1.trans import make_bounded_mean
    query = make_bounded_mean(lower=0., upper=10., n=9, MI=HammingDistance, MO=L1Sensitivity[float])
    assert query(FLOAT_DATA) == 5.
    assert query.check(1, 10. / 9.)


def test_bounded_sum():
    from opendp.v1.trans import make_bounded_sum
    query = make_bounded_sum(lower=0., upper=10., MI=HammingDistance, MO=L1Sensitivity[float])
    assert query(FLOAT_DATA) == 45.
    # TODO: tighten the check
    assert query.check(1, 20.)

    query = make_bounded_sum(lower=0, upper=10, MI=HammingDistance, MO=L1Sensitivity[int])
    assert query(INT_DATA) == 45
    # TODO: tighten the check
    assert query.check(1, 20)

    try:
        query(FLOAT_DATA)
        raise ValueError("should not accept float data")
    except AssertionError:
        pass


def test_bounded_sum_n():
    from opendp.v1.trans import make_bounded_sum_n
    query = make_bounded_sum_n(lower=0., upper=10., n=9, MO=L1Sensitivity[float])
    assert query(FLOAT_DATA) == 45.
    # TODO: tighten this check
    assert query.check(1, 20.)


def test_bounded_variance():
    from opendp.v1.trans import make_bounded_variance
    query = make_bounded_variance(lower=0., upper=10., n=9, MI=HammingDistance, MO=L1Sensitivity[float])
    assert query(FLOAT_DATA) == 7.5
    assert query.check(1, 20.)


def test_count():
    from opendp.v1.trans import make_count
    transformation = make_count(SymmetricDistance, L1Sensitivity["i32"], int)
    arg = [1, 2, 3]
    ret = transformation(arg)
    assert ret == 3
    assert transformation.check(1, 1)


def test_count_by():
    from opendp.v1.trans import make_count_by
    query = make_count_by(n=9, MI=HammingDistance, MO=L1Sensitivity[float], TI=int)
    # TODO: cannot test until hashmap data unloader is added
    # assert query(INT_DATA) == {i + 1: 1 for i in range(9)}
    print('first')
    assert query.check(1, 2.)


def test_count_by_categories():
    from opendp.v1.trans import make_count_by_categories
    query = make_count_by_categories(categories=[1, 3, 4], MI=HammingDistance, MO=L1Sensitivity[float])
    assert query(INT_DATA) == [1, 1, 1, 6]
    assert query.check(1, 2.)
