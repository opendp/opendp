from opendp.mod import enable_features
import opendp.prelude as dp

enable_features("floating-point", "contrib")


def test_private_quantile():
    input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()
    
    m_median = input_space >> dp.m.then_private_quantile(
        candidates=[0, 25, 50, 75, 100],
        alpha=0.5,
        scale=1.0,
    )

    assert m_median(list(range(100))) == 50
    assert m_median.map(1) == 1.0
