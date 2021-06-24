from opendp.v1.typing import HammingDistance, L1Distance, SymmetricDistance, AbsoluteDistance

INT_DATA = list(range(1, 10))
FLOAT_DATA = list(map(float, INT_DATA))


def test_cast_impute():
    from opendp.v1.trans import make_cast, make_impute_constant
    caster = make_cast(M=HammingDistance, TI=float, TO=int) >> make_impute_constant(-1, M=HammingDistance)
    assert caster([1., 2., 3.]) == [1, 2, 3]

    caster = make_cast(M=HammingDistance, TI=float, TO=int) \
             >> make_impute_constant(1, M=HammingDistance)
    assert caster([float('nan'), 2.]) == [1, 2]


def test_cast_inherent():
    from opendp.v1.trans import make_cast_inherent
    caster = make_cast_inherent(M=HammingDistance, TI=int, TO=float)

    assert caster([1, 2]) == [1., 2.]


def test_impute_constant_inherent():
    from opendp.v1.trans import make_impute_constant_inherent
    imputer = make_impute_constant_inherent(-1., M=HammingDistance)
    assert imputer([float('nan'), 1.]) == [-1., 1.]


def test_cast_default():
    from opendp.v1.trans import make_cast_default
    caster = make_cast_default(M=HammingDistance, TI=float, TO=int)
    assert caster([float('nan'), 2.]) == [0, 2]


def test_impute_uniform():
    from opendp.v1.trans import make_impute_uniform_float
    caster = make_impute_uniform_float(-1., 2., M=HammingDistance)
    assert -1. <= caster([float('nan')])[0] <= 2.


def test_cast_metric():
    from opendp.v1.trans import make_cast_metric
    caster = make_cast_metric(HammingDistance, SymmetricDistance, T=float)
    assert caster([1., 2.]) == [1., 2.]
    assert not caster.check(1, 1)


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
def test_split_lines__cast__impute():
    from opendp.v1.trans import make_split_lines, make_cast, make_impute_constant
    query = (
            make_split_lines(M=HammingDistance) >>
            make_cast(M=HammingDistance, TI=str, TO=int) >>
            make_impute_constant(constant=2, M=HammingDistance)
    )
    assert query("1\n2\n3") == [1, 2, 3]
    assert query.check(1, 1)


def test_inherent_cast__impute():
    from opendp.v1.trans import make_split_lines, make_cast_inherent, make_impute_constant_inherent
    cast = make_split_lines(M=HammingDistance) >> make_cast_inherent(M=HammingDistance, TI=str, TO=float)
    constant = cast >> make_impute_constant_inherent(constant=9., M=HammingDistance)

    assert constant("a\n23.23\n12") == [9., 23.23, 12.]
    assert constant.check(1, 1)


def test_inherent_cast__impute_uniform():
    from opendp.v1.trans import make_split_lines, make_cast_inherent, make_impute_uniform_float
    cast = make_split_lines(M=HammingDistance) >> make_cast_inherent(M=HammingDistance, TI=str, TO=float)
    constant = cast >> make_impute_uniform_float(lower=23., upper=32.5, M=HammingDistance)

    res = constant("a\n23.23\n12")
    assert res[1:] == [23.23, 12.]
    assert 23. <= res[0] <= 32.5
    assert constant.check(1, 1)



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


def test_vector_clamp():
    from opendp.v1.trans import make_clamp
    query = make_clamp(lower=-1, upper=1, M=HammingDistance)
    assert query([-10, 0, 10]) == [-1, 0, 1]
    assert query.check(1, 1)


def test_clamp_sensitivity():
    from opendp.v1.trans import make_clamp
    query = make_clamp(lower=-1, upper=1, M=AbsoluteDistance[int])
    assert query(20) == 1
    assert query.check(20, 2)


def test_bounded_mean():
    from opendp.v1.trans import make_bounded_mean
    query = make_bounded_mean(lower=0., upper=10., n=9, MI=HammingDistance)
    assert query(FLOAT_DATA) == 5.
    assert query.check(1, 10. / 9.)


def test_bounded_sum():
    from opendp.v1.trans import make_bounded_sum
    query = make_bounded_sum(lower=0., upper=10., MI=HammingDistance)
    assert query(FLOAT_DATA) == 45.
    # TODO: tighten the check
    assert query.check(1, 20.)

    query = make_bounded_sum(lower=0, upper=10, MI=HammingDistance)
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
    query = make_bounded_sum_n(lower=0., upper=10., n=9)
    assert query(FLOAT_DATA) == 45.
    # TODO: tighten this check
    assert query.check(1, 20.)


def test_bounded_variance():
    from opendp.v1.trans import make_bounded_variance
    query = make_bounded_variance(lower=0., upper=10., n=9, MI=HammingDistance)
    assert query(FLOAT_DATA) == 7.5
    assert query.check(1, 20.)


def test_count():
    from opendp.v1.trans import make_count
    transformation = make_count(SymmetricDistance, TI=int, TO=int)
    arg = [1, 2, 3]
    ret = transformation(arg)
    assert ret == 3
    assert transformation.check(1, 1)


def test_count_distinct():
    from opendp.v1.trans import make_count_distinct
    transformation = make_count_distinct(SymmetricDistance, L1Distance["i32"], int)
    arg = [1, 2, 3, 2, 7, 3, 4]
    ret = transformation(arg)
    assert ret == 5
    assert transformation.check(1, 1)


def test_count_by():
    from opendp.v1.trans import make_count_by
    query = make_count_by(n=9, MI=HammingDistance, MO=L1Distance[float], TI=int)
    # TODO: cannot test until hashmap data unloader is added
    # assert query(INT_DATA) == {i + 1: 1 for i in range(9)}
    print('first')
    assert query.check(1, 2.)


def test_count_by_categories():
    from opendp.v1.trans import make_count_by_categories
    query = make_count_by_categories(categories=[1, 3, 4], MI=HammingDistance, MO=L1Distance[float])
    assert query(INT_DATA) == [1, 1, 1, 6]
    assert query.check(1, 2.)
