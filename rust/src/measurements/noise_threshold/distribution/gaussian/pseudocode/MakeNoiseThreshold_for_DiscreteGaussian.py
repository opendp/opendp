# type: ignore
# analogous to impl MakeNoiseThreshold<DI, MI, MO> for DiscreteGaussian in Rust
class DiscreteGaussian:
    def make_noise_threshold(self, input_space, threshold) -> Measurement[DI, DI_Carrier, MI, MO]:
        # an equivalent random variable specific to the atom dtype
        rv_nature = DI_Atom.new_distribution(self.scale, self.k) # |\label{line:rv-nature}|
        # build a measurement sampling from this equivalent distribution
        return rv_nature.make_noise_threshold(input_space, threshold) # |\label{line:make-noise}|
