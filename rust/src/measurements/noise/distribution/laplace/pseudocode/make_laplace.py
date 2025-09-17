# type: ignore
def make_laplace(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    k: Option[i32],
) -> Measurement[DI, DI_Carrier, MI, MO]:
    return DiscreteLaplace(scale, k).make_noise((input_domain, input_metric))
