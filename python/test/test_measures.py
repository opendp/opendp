import opendp.prelude as dp


def test_inequality():
    measure = dp.max_divergence()
    assert measure != str(measure)
