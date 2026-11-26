# type: ignore
# analogous to impl MakeNoise<VectorDomain<AtomDomain<T>>, L1Distance<QI>, MO> for ConstantTimeGeometric<T> in Rust
class ConstantTimeGeometric:
    def make_noise(
        self, input_space: tuple[DI, MI]
    ) -> Measurement[DI, DI_Carrier, MI, MO]:
        input_domain, input_metric = input_space
        scale, bounds = self.scale, self.bounds
        width, threshold = discrete_laplace_linear_parameters(scale, bounds)

        distribution = ZExpFamily(scale=RBig.from_f64(scale))
        output_measure = MO.default()

        privacy_map = distribution.noise_privacy_map(
            L1Distance.default(), output_measure
        )

        def function(arg: Vec[T]) -> Vec[T]:
            return [
                sample_bounded_discrete_laplace(v, bounds, width, threshold)
                for v in arg
            ]

        return Measurement.new(
            input_domain,
            Function.new_fallible(function),
            input_metric,
            output_measure,
            PrivacyMap.new_fallible(lambda d_in: privacy_map.eval(RBig.try_from(d_in))),
        )
