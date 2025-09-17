# type: ignore
class ZExpFamily1: # analogous to impl Sample for ZExpFamily<1> in Rust
    def sample(self, shift):
        return shift + sample_discrete_laplace(self.scale) # |\label{line:add}|
