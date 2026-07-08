# type: ignore
class ZExpFamily1: # analogous to impl Sample for ZExpFamily<1> in Rust
    def sample(self, shift):
        return [x_i + sample_discrete_laplace(self.scale) for x_i in shift] # |\label{line:add}|
