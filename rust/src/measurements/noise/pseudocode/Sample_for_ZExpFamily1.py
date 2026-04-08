# type: ignore
class ZExpFamily1: # analogous to impl Sample for ZExpFamily<1> in Rust
    def sample(self, shift):
        sample = shift + sample_discrete_laplace(self.scale) # |\label{line:add}|
        if self.divisor is not None: # |\label{line:mod}|
            sample %= self.divisor
        return sample
