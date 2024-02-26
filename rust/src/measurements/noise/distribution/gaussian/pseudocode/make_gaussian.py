# type: ignore
def make_gaussian(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    k: Option[i32],
) -> Measurement[DI, DI_Carrier, MI, MO]:
    return DiscreteGaussian(scale, k).make_noise((input_domain, input_metric))
