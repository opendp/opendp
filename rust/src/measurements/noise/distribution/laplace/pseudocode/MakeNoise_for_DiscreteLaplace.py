# type: ignore
# analogous to impl MakeNoise<DI, MI, MO> for DiscreteLaplace in Rust
class DiscreteLaplace:
    def make_noise(self, input_space: tuple[DI, MI]) -> Measurement[DI, DI_Carrier, MI, MO]:
        # an equivalent random variable specific to the atom dtype
        rv_nature = DI_Atom.new_distribution(self.scale, self.k) # |\label{line:rv-nature}|
        # build a measurement sampling from this equivalent distribution
        return rv_nature.make_noise(input_space) # |\label{line:make-noise}|
