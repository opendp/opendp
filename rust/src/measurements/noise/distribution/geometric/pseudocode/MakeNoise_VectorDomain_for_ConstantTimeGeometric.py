# type: ignore
# analogous to impl MakeNoise<VectorDomain<AtomDomain<T>>, L1Distance<T>, MO> for ConstantTimeGeometric<T> in Rust
class ConstantTimeGeometric:
    def make_noise(
        self, input_space: tuple[DI, MI]
    ) -> Measurement[DI, DI_Carrier, MI, MO]:
        input_domain, input_metric = input_space
        scale, (lower, upper) = self.scale, self.bounds
        if lower > upper: # |\label{line:check-bounds}|
            raise "lower may not be greater than upper"

        distribution = ZExpFamily(scale=RBig.from_f64(scale))
        output_measure = MO.default()

        privacy_map = distribution.noise_privacy_map(
            L1Distance.default(), output_measure
        )

        p = (1.0).neg_inf_sub((-scale.recip()).inf_exp()) # |\label{line:prob-check}|
        if not (0.0 < p <= 1.0):
            raise f"Probability of termination p ({p}) must be in (0, 1]. This is likely because the noise scale is so large that conservative arithmetic causes p to go negative"

        def function(arg: Vec[T]) -> Vec[T]:
            return [
                sample_discrete_laplace_linear(v, scale, (lower, upper)) for v in arg
            ]

        return Measurement.new(
            input_domain,
            Function.new_fallible(function),
            input_metric,
            output_measure,
            PrivacyMap.new_fallible(lambda d_in: privacy_map.eval(RBig.try_from(d_in))),
        )
