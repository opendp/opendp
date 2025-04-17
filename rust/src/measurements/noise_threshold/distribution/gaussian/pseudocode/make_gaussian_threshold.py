# type: ignore
def make_gaussian_threshold(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    threshold: DI_Atom,
    k: Option[i32],
) -> Measurement[DI, DI_Carrier, MI, MO]:
    return DiscreteGaussian(scale, k).make_noise_threshold(
        (input_domain, input_metric), threshold
    )
