# type: ignore
def make_geometric(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    bounds: Option[tuple[DI_Atom, DI_Atom]],
) -> Measurement[DI, DI_Carrier, MI, MO]:
    input_space = input_domain, input_metric
    if bounds is None:
        return DiscreteLaplace(scale, k=None).make_noise(input_space)
    else:
        return ConstantTimeGeometric(scale, bounds).make_noise(input_space)
