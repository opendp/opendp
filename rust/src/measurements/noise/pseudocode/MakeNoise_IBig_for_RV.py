# type: ignore
# analogous to impl MakeNoise<VectorDomain<AtomDomain<IBig>>, MI, MO> for RV in Rust
class RV:
    def make_noise(self, input_space) -> Measurement[VectorDomain[AtomDomain[IBig]], Vec[IBig], MI, MO]:
        input_domain, input_metric = input_space
        return Measurement.new(
            input_domain,
            Function.new_fallible(
                lambda x: [self.sample(x_i) for x_i in x]), # |\label{line:sample}|
            input_metric,
            MO.default(),
            self.noise_privacy_map(),  # |\label{line:privacy-map}|
        ) 
